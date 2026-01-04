const std = @import("std");
const Allocator = std.mem.Allocator;

pub const BundleError = error{
    FileNotFound,
    ReadError,
    WriteError,
    CircularDependency,
    InvalidPath,
    OutOfMemory,
};

const Module = struct {
    id: u32,
    path: []const u8,
    code: []const u8,
    requires: std.ArrayListUnmanaged([]const u8),
};

pub const Bundler = struct {
    allocator: Allocator,
    modules: std.ArrayListUnmanaged(Module),
    path_to_id: std.StringHashMapUnmanaged(u32),
    next_id: u32,
    base_dir: []const u8,

    pub fn init(allocator: Allocator) Bundler {
        return .{
            .allocator = allocator,
            .modules = .{},
            .path_to_id = .{},
            .next_id = 0,
            .base_dir = "",
        };
    }

    pub fn deinit(self: *Bundler) void {
        for (self.modules.items) |*mod| {
            mod.requires.deinit(self.allocator);
            self.allocator.free(mod.code);
        }
        self.modules.deinit(self.allocator);
        self.path_to_id.deinit(self.allocator);
    }

    pub fn bundle(self: *Bundler, entry_path: []const u8) ![]const u8 {
        // Set base directory
        if (std.fs.path.dirname(entry_path)) |dir| {
            self.base_dir = dir;
        } else {
            self.base_dir = ".";
        }

        // Resolve entry to absolute path
        const abs_entry = try self.resolvePath(entry_path, ".");

        // Process entry module and all its dependencies
        _ = try self.processModule(abs_entry);

        // Generate bundled output
        return try self.generateBundle();
    }

    fn processModule(self: *Bundler, path: []const u8) !u32 {
        // Check if already processed
        if (self.path_to_id.get(path)) |id| {
            return id;
        }

        // Read file
        const code = self.readFile(path) catch |err| {
            std.debug.print("Error reading {s}: {}\n", .{ path, err });
            return BundleError.FileNotFound;
        };

        // Assign ID
        const id = self.next_id;
        self.next_id += 1;

        // Store path -> id mapping
        try self.path_to_id.put(self.allocator, path, id);

        // Find all require() calls
        var requires: std.ArrayListUnmanaged([]const u8) = .{};
        try self.findRequires(code, &requires);

        // Add module
        try self.modules.append(self.allocator, .{
            .id = id,
            .path = path,
            .code = code,
            .requires = requires,
        });

        // Process dependencies
        const dir = std.fs.path.dirname(path) orelse ".";
        for (requires.items) |req| {
            const dep_path = try self.resolvePath(req, dir);
            _ = try self.processModule(dep_path);
        }

        return id;
    }

    fn findRequires(self: *Bundler, code: []const u8, requires: *std.ArrayListUnmanaged([]const u8)) !void {
        // Simple regex-like search for require("...") or require('...')
        var i: usize = 0;
        while (i < code.len) {
            // Look for "require("
            if (i + 8 < code.len and std.mem.eql(u8, code[i .. i + 8], "require(")) {
                i += 8;
                // Skip whitespace
                while (i < code.len and (code[i] == ' ' or code[i] == '\t')) {
                    i += 1;
                }
                // Check for quote
                if (i < code.len and (code[i] == '"' or code[i] == '\'')) {
                    const quote = code[i];
                    i += 1;
                    const start = i;
                    // Find closing quote
                    while (i < code.len and code[i] != quote) {
                        i += 1;
                    }
                    if (i > start) {
                        const req_path = code[start..i];
                        // Skip built-in modules (no ./ or ../)
                        if (req_path.len > 0 and (req_path[0] == '.' or req_path[0] == '/')) {
                            const duped = try self.allocator.dupe(u8, req_path);
                            try requires.append(self.allocator, duped);
                        }
                    }
                }
            }
            i += 1;
        }
    }

    fn resolvePath(self: *Bundler, req_path: []const u8, from_dir: []const u8) ![]const u8 {
        var path_buf: [4096]u8 = undefined;

        // Handle relative paths
        if (req_path.len > 0 and req_path[0] == '.') {
            // Join with from_dir
            const joined = try std.fmt.bufPrint(&path_buf, "{s}/{s}", .{ from_dir, req_path });

            // Add .js extension if needed
            if (!std.mem.endsWith(u8, joined, ".js") and !std.mem.endsWith(u8, joined, ".json")) {
                const with_ext = try std.fmt.bufPrint(&path_buf, "{s}/{s}.js", .{ from_dir, req_path });
                return try self.allocator.dupe(u8, with_ext);
            }
            return try self.allocator.dupe(u8, joined);
        }

        // Handle absolute paths
        if (req_path.len > 0 and req_path[0] == '/') {
            return try self.allocator.dupe(u8, req_path);
        }

        // For non-relative paths, assume it's in the same directory
        const joined = try std.fmt.bufPrint(&path_buf, "{s}/{s}", .{ from_dir, req_path });
        if (!std.mem.endsWith(u8, joined, ".js") and !std.mem.endsWith(u8, joined, ".json")) {
            const with_ext = try std.fmt.bufPrint(&path_buf, "{s}/{s}.js", .{ from_dir, req_path });
            return try self.allocator.dupe(u8, with_ext);
        }
        return try self.allocator.dupe(u8, joined);
    }

    fn readFile(self: *Bundler, path: []const u8) ![]const u8 {
        _ = self;
        const file = try std.fs.cwd().openFile(path, .{});
        defer file.close();

        const stat = try file.stat();
        const content = try std.heap.page_allocator.alloc(u8, stat.size);
        const bytes_read = try file.readAll(content);
        if (bytes_read != stat.size) {
            return error.ReadError;
        }
        return content;
    }

    fn generateBundle(self: *Bundler) ![]const u8 {
        var output: std.ArrayListUnmanaged(u8) = .{};
        const writer = output.writer(self.allocator);

        // Write bundle header
        try writer.writeAll(
            \\// Kiren Bundle - Generated automatically
            \\(function() {
            \\  var __modules = {};
            \\  var __cache = {};
            \\
            \\  function __require(id) {
            \\    if (__cache[id]) return __cache[id].exports;
            \\    var module = __cache[id] = { exports: {} };
            \\    __modules[id](module.exports, function(path) {
            \\      return __require(__resolve(id, path));
            \\    }, module);
            \\    return module.exports;
            \\  }
            \\
            \\  var __paths = {
        );

        // Write path -> id mapping
        var first = true;
        for (self.modules.items) |mod| {
            if (!first) try writer.writeAll(",");
            first = false;
            try writer.print("\n    \"{s}\": {d}", .{ mod.path, mod.id });
        }

        try writer.writeAll(
            \\
            \\  };
            \\
            \\  function __resolve(fromId, path) {
            \\    // Simple resolution - in bundled code, paths are already resolved
            \\    for (var p in __paths) {
            \\      if (p.indexOf(path) !== -1 || path.indexOf(p) !== -1) {
            \\        return __paths[p];
            \\      }
            \\    }
            \\    // Try with .js extension
            \\    var withJs = path + ".js";
            \\    for (var p in __paths) {
            \\      if (p.indexOf(withJs) !== -1) {
            \\        return __paths[p];
            \\      }
            \\    }
            \\    throw new Error("Module not found: " + path);
            \\  }
            \\
        );

        // Write each module
        for (self.modules.items) |mod| {
            try writer.print(
                \\
                \\  // Module {d}: {s}
                \\  __modules[{d}] = function(exports, require, module) {{
                \\
            , .{ mod.id, mod.path, mod.id });

            // Write module code (indent each line)
            var lines = std.mem.splitScalar(u8, mod.code, '\n');
            while (lines.next()) |line| {
                try writer.writeAll("    ");
                try writer.writeAll(line);
                try writer.writeAll("\n");
            }

            try writer.writeAll("  };\n");
        }

        // Write entry point
        try writer.writeAll(
            \\
            \\  // Start execution
            \\  __require(0);
            \\})();
            \\
        );

        return output.toOwnedSlice(self.allocator);
    }
};

// Bundle and write to file
pub fn bundleToFile(entry_path: []const u8, output_path: []const u8) !void {
    var bundler = Bundler.init(std.heap.page_allocator);
    defer bundler.deinit();

    const code = try bundler.bundle(entry_path);
    defer std.heap.page_allocator.free(code);

    const file = try std.fs.cwd().createFile(output_path, .{});
    defer file.close();

    try file.writeAll(code);
}

// Create self-contained executable
pub fn bundleToExecutable(entry_path: []const u8, output_path: []const u8) !void {
    var bundler = Bundler.init(std.heap.page_allocator);
    defer bundler.deinit();

    const js_code = try bundler.bundle(entry_path);
    defer std.heap.page_allocator.free(js_code);

    // Get path to current executable (kiren)
    var exe_path_buf: [4096]u8 = undefined;
    const exe_path = try std.fs.selfExePath(&exe_path_buf);

    // Read kiren binary
    const exe_file = try std.fs.openFileAbsolute(exe_path, .{});
    defer exe_file.close();

    const exe_stat = try exe_file.stat();
    const exe_content = try std.heap.page_allocator.alloc(u8, exe_stat.size);
    defer std.heap.page_allocator.free(exe_content);
    _ = try exe_file.readAll(exe_content);

    // Create output file
    const out_file = try std.fs.cwd().createFile(output_path, .{ .mode = 0o755 });
    defer out_file.close();

    // Write: [kiren binary] [js code] [js length: 8 bytes] [magic: "KIRENJS\0"]
    try out_file.writeAll(exe_content);
    try out_file.writeAll(js_code);

    // Write JS code length as 8-byte little-endian
    const js_len: u64 = @intCast(js_code.len);
    var len_bytes: [8]u8 = undefined;
    std.mem.writeInt(u64, &len_bytes, js_len, .little);
    try out_file.writeAll(&len_bytes);

    // Write magic marker
    try out_file.writeAll("KIRENJS\x00");
}

// Check if current executable has embedded JS
pub fn getEmbeddedCode() ?[]const u8 {
    var exe_path_buf: [4096]u8 = undefined;
    const exe_path = std.fs.selfExePath(&exe_path_buf) catch return null;

    const file = std.fs.openFileAbsolute(exe_path, .{}) catch return null;
    defer file.close();

    const stat = file.stat() catch return null;
    const size = stat.size;

    // Check if file is large enough to have embedded code
    // Minimum: magic(8) + length(8) = 16 bytes
    if (size < 16) return null;

    // Seek to end - 8 bytes to read magic
    file.seekTo(size - 8) catch return null;
    var magic_buf: [8]u8 = undefined;
    _ = file.read(&magic_buf) catch return null;

    // Check magic
    if (!std.mem.eql(u8, &magic_buf, "KIRENJS\x00")) {
        return null;
    }

    // Read JS length
    file.seekTo(size - 16) catch return null;
    var len_buf: [8]u8 = undefined;
    _ = file.read(&len_buf) catch return null;
    const js_len = std.mem.readInt(u64, &len_buf, .little);

    // Validate length
    if (js_len == 0 or js_len > size - 16) return null;

    // Read JS code
    const js_start = size - 16 - js_len;
    file.seekTo(js_start) catch return null;

    const js_code = std.heap.page_allocator.alloc(u8, js_len) catch return null;
    const bytes_read = file.read(js_code) catch {
        std.heap.page_allocator.free(js_code);
        return null;
    };

    if (bytes_read != js_len) {
        std.heap.page_allocator.free(js_code);
        return null;
    }

    return js_code;
}
