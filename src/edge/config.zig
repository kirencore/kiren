const std = @import("std");

pub const WorkerConfig = struct {
    name: []const u8,
    path: []const u8,
    routes: []const []const u8,
};

pub const EdgeConfig = struct {
    allocator: std.mem.Allocator,
    workers: []WorkerConfig,
    port: u16,

    pub fn parse(allocator: std.mem.Allocator, json_path: []const u8) !EdgeConfig {
        const file = std.fs.cwd().openFile(json_path, .{}) catch |err| {
            std.debug.print("Failed to open config file '{s}': {}\n", .{ json_path, err });
            return error.ConfigNotFound;
        };
        defer file.close();

        const stat = try file.stat();
        const content = try allocator.alloc(u8, stat.size);
        defer allocator.free(content);
        _ = try file.readAll(content);

        return parseJson(allocator, content);
    }

    fn parseJson(allocator: std.mem.Allocator, content: []const u8) !EdgeConfig {
        var config = EdgeConfig{
            .allocator = allocator,
            .workers = &[_]WorkerConfig{},
            .port = 3000,
        };

        // Parse port
        if (findJsonValue(content, "\"port\"")) |port_str| {
            config.port = std.fmt.parseInt(u16, std.mem.trim(u8, port_str, " \t\n\r,"), 10) catch 3000;
        }

        // Parse workers
        config.workers = try parseWorkers(allocator, content);

        return config;
    }

    fn parseWorkers(allocator: std.mem.Allocator, content: []const u8) ![]WorkerConfig {
        var workers: std.ArrayListUnmanaged(WorkerConfig) = .{};
        errdefer {
            for (workers.items) |w| {
                allocator.free(w.name);
                allocator.free(w.path);
                for (w.routes) |r| allocator.free(r);
                allocator.free(w.routes);
            }
            workers.deinit(allocator);
        }

        // Find "workers": { ... }
        const workers_start = std.mem.indexOf(u8, content, "\"workers\"") orelse return workers.toOwnedSlice(allocator);

        // Find the opening brace after "workers":
        var i = workers_start + 9;
        while (i < content.len and content[i] != '{') : (i += 1) {}
        if (i >= content.len) return workers.toOwnedSlice(allocator);
        i += 1; // Skip '{'

        // Find matching closing brace
        var brace_depth: usize = 1;
        const workers_content_start = i;
        var workers_content_end = i;

        while (i < content.len and brace_depth > 0) {
            if (content[i] == '{') brace_depth += 1;
            if (content[i] == '}') {
                brace_depth -= 1;
                if (brace_depth == 0) workers_content_end = i;
            }
            i += 1;
        }

        const workers_block = content[workers_content_start..workers_content_end];

        // Parse each worker: "name": { "path": "...", "routes": [...] }
        var pos: usize = 0;
        while (pos < workers_block.len) {
            // Find worker name (in quotes)
            const name_start = std.mem.indexOfScalarPos(u8, workers_block, pos, '"') orelse break;
            const name_end = std.mem.indexOfScalarPos(u8, workers_block, name_start + 1, '"') orelse break;
            const worker_name = workers_block[name_start + 1 .. name_end];

            // Find opening brace for this worker's config
            var j = name_end + 1;
            while (j < workers_block.len and workers_block[j] != '{') : (j += 1) {}
            if (j >= workers_block.len) break;
            j += 1;

            // Find matching closing brace
            var depth: usize = 1;
            const worker_start = j;
            while (j < workers_block.len and depth > 0) {
                if (workers_block[j] == '{') depth += 1;
                if (workers_block[j] == '}') depth -= 1;
                j += 1;
            }
            const worker_config = workers_block[worker_start .. j - 1];

            // Parse path
            var worker_path: []const u8 = "";
            if (findJsonString(worker_config, "\"path\"")) |p| {
                worker_path = try allocator.dupe(u8, p);
            }

            // Parse routes array
            var routes_list: std.ArrayListUnmanaged([]const u8) = .{};
            if (findJsonArray(worker_config, "\"routes\"")) |routes_str| {
                var route_pos: usize = 0;
                while (route_pos < routes_str.len) {
                    const rs = std.mem.indexOfScalarPos(u8, routes_str, route_pos, '"') orelse break;
                    const re = std.mem.indexOfScalarPos(u8, routes_str, rs + 1, '"') orelse break;
                    const route = routes_str[rs + 1 .. re];
                    try routes_list.append(allocator, try allocator.dupe(u8, route));
                    route_pos = re + 1;
                }
            }

            try workers.append(allocator, WorkerConfig{
                .name = try allocator.dupe(u8, worker_name),
                .path = worker_path,
                .routes = try routes_list.toOwnedSlice(allocator),
            });

            pos = name_end + j;
        }

        return workers.toOwnedSlice(allocator);
    }

    pub fn deinit(self: *EdgeConfig) void {
        for (self.workers) |worker| {
            self.allocator.free(worker.name);
            if (worker.path.len > 0) self.allocator.free(worker.path);
            for (worker.routes) |route| {
                self.allocator.free(route);
            }
            self.allocator.free(worker.routes);
        }
        self.allocator.free(self.workers);
    }
};

fn findJsonValue(content: []const u8, key: []const u8) ?[]const u8 {
    const key_pos = std.mem.indexOf(u8, content, key) orelse return null;
    var i = key_pos + key.len;

    // Skip to colon
    while (i < content.len and content[i] != ':') : (i += 1) {}
    if (i >= content.len) return null;
    i += 1;

    // Skip whitespace
    while (i < content.len and (content[i] == ' ' or content[i] == '\t' or content[i] == '\n' or content[i] == '\r')) : (i += 1) {}
    if (i >= content.len) return null;

    // Find end of value
    var end = i;
    while (end < content.len and content[end] != ',' and content[end] != '}' and content[end] != '\n') : (end += 1) {}

    return content[i..end];
}

fn findJsonString(content: []const u8, key: []const u8) ?[]const u8 {
    const key_pos = std.mem.indexOf(u8, content, key) orelse return null;
    var i = key_pos + key.len;

    // Skip to colon
    while (i < content.len and content[i] != ':') : (i += 1) {}
    if (i >= content.len) return null;
    i += 1;

    // Skip whitespace
    while (i < content.len and (content[i] == ' ' or content[i] == '\t' or content[i] == '\n' or content[i] == '\r')) : (i += 1) {}
    if (i >= content.len) return null;

    // Expect opening quote
    if (content[i] != '"') return null;
    i += 1;

    const start = i;
    while (i < content.len and content[i] != '"') : (i += 1) {}

    return content[start..i];
}

fn findJsonArray(content: []const u8, key: []const u8) ?[]const u8 {
    const key_pos = std.mem.indexOf(u8, content, key) orelse return null;
    var i = key_pos + key.len;

    // Skip to colon
    while (i < content.len and content[i] != ':') : (i += 1) {}
    if (i >= content.len) return null;
    i += 1;

    // Skip whitespace
    while (i < content.len and (content[i] == ' ' or content[i] == '\t' or content[i] == '\n' or content[i] == '\r')) : (i += 1) {}
    if (i >= content.len) return null;

    // Expect opening bracket
    if (content[i] != '[') return null;
    i += 1;

    const start = i;

    // Find matching closing bracket
    var depth: usize = 1;
    while (i < content.len and depth > 0) {
        if (content[i] == '[') depth += 1;
        if (content[i] == ']') depth -= 1;
        i += 1;
    }

    return content[start .. i - 1];
}
