const std = @import("std");
const Engine = @import("engine.zig").Engine;
const engine = @import("engine.zig");
const console = @import("api/console.zig");
const process = @import("api/process.zig");
const path_mod = @import("api/path.zig");
const fs = @import("api/fs.zig");
const buffer = @import("api/buffer.zig");
const url = @import("api/url.zig");
const encoding = @import("api/encoding.zig");
const crypto = @import("api/crypto.zig");
const fetch = @import("api/fetch.zig");
const module = @import("api/module.zig");
const http = @import("api/http.zig");
const websocket = @import("api/websocket.zig");
const sqlite = @import("api/sqlite.zig");
const event_loop = @import("event_loop.zig");
const bundler = @import("bundler.zig");

const VERSION = "0.1.0";

fn print(comptime fmt: []const u8, args: anytype) void {
    std.debug.print(fmt, args);
}

fn printUsage() void {
    print(
        \\Kiren - JavaScript Runtime
        \\
        \\Usage:
        \\  kiren <file.js>           Run a JavaScript file
        \\  kiren -e <code>           Evaluate inline JavaScript
        \\  kiren bundle <file.js>    Create standalone executable
        \\  kiren --version           Show version info
        \\  kiren --help              Show this help message
        \\
        \\Bundle options:
        \\  kiren bundle app.js                Create ./app executable
        \\  kiren bundle app.js -o myapp       Create ./myapp executable
        \\  kiren bundle app.js --js-only      Create bundled JS file only
        \\
        \\Examples:
        \\  kiren script.js
        \\  kiren -e "console.log('Hello!')"
        \\  kiren bundle server.js -o server
        \\
    , .{});
}

fn printVersion() void {
    print("kiren v{s}\n", .{VERSION});
}

fn fatal(comptime fmt: []const u8, args: anytype) noreturn {
    print(fmt, args);
    std.process.exit(1);
}

pub fn main() u8 {
    // Check for embedded code first (for bundled executables)
    if (bundler.getEmbeddedCode()) |embedded_code| {
        return runEmbeddedCode(embedded_code);
    }

    // Collect all arguments for process.argv
    var args = std.process.args();
    var argv_list: std.ArrayListUnmanaged([:0]const u8) = .{};
    defer argv_list.deinit(std.heap.page_allocator);

    while (args.next()) |arg| {
        argv_list.append(std.heap.page_allocator, arg) catch {};
    }
    process.setArgv(argv_list.items);

    // Get first argument (skip executable name)
    const first_arg = if (argv_list.items.len > 1) argv_list.items[1] else null;

    if (first_arg == null) {
        printUsage();
        return 0;
    }

    const arg = first_arg.?;

    // Handle bundle command
    if (std.mem.eql(u8, arg, "bundle")) {
        return handleBundle(argv_list.items);
    }

    if (std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h")) {
        printUsage();
        return 0;
    }

    if (std.mem.eql(u8, arg, "--version") or std.mem.eql(u8, arg, "-v")) {
        printVersion();
        return 0;
    }

    // Initialize engine
    var eng = Engine.init() catch |err| {
        fatal("Failed to initialize engine: {}\n", .{err});
    };
    defer eng.deinit();

    // Initialize event loop (now with I/O poller)
    var loop = event_loop.EventLoop.init(std.heap.page_allocator, eng.context) catch |err| {
        fatal("Failed to initialize event loop: {}\n", .{err});
    };
    defer loop.deinit();
    event_loop.setGlobalEventLoop(&loop);

    // Register APIs
    console.register(&eng);
    process.register(&eng);
    path_mod.register(&eng);
    fs.register(&eng);
    buffer.register(&eng);
    url.register(&eng);
    encoding.register(&eng);
    crypto.register(&eng);
    fetch.register(&eng);
    module.register(&eng);
    http.register(&eng);
    websocket.register(&eng);
    sqlite.register(&eng);
    event_loop.register(&eng);

    if (std.mem.eql(u8, arg, "-e")) {
        // Execute inline code
        const code = if (argv_list.items.len > 2) argv_list.items[2] else {
            fatal("Error: code expected after -e\n", .{});
        };

        const result = eng.eval(code, "<inline>");
        if (result) |val| {
            // Show result (if not undefined)
            if (engine.c.JS_IsUndefined(val) == 0) {
                printResult(&eng, val);
            }
            eng.freeValue(val);

            // Run event loop if there are pending timers
            loop.run();

            return 0;
        } else |_| {
            return 1;
        }
    } else {
        // Execute file
        // Set module directory based on script location
        if (std.fs.path.dirname(arg)) |dir| {
            module.setModuleDir(dir);
        }

        const result = eng.evalFile(arg);
        if (result) |val| {
            eng.freeValue(val);

            // Run event loop if there are pending timers
            loop.run();

            return 0;
        } else |_| {
            return 1;
        }
    }
}

fn printResult(eng: *Engine, val: engine.JSValue) void {
    const c = engine.c;

    if (c.JS_IsNull(val) != 0) {
        print("\x1b[1mnull\x1b[0m\n", .{});
    } else if (c.JS_IsBool(val) != 0) {
        const b = c.JS_ToBool(eng.context, val);
        if (b != 0) {
            print("\x1b[33mtrue\x1b[0m\n", .{});
        } else {
            print("\x1b[33mfalse\x1b[0m\n", .{});
        }
    } else if (c.JS_IsNumber(val) != 0) {
        var num: f64 = 0;
        _ = c.JS_ToFloat64(eng.context, &num, val);
        print("\x1b[33m{d}\x1b[0m\n", .{num});
    } else if (c.JS_IsString(val) != 0) {
        const str = c.JS_ToCString(eng.context, val);
        if (str != null) {
            print("\x1b[32m'{s}'\x1b[0m\n", .{str});
            c.JS_FreeCString(eng.context, str);
        }
    } else if (c.JS_IsFunction(eng.context, val) != 0) {
        print("\x1b[36m[Function]\x1b[0m\n", .{});
    } else if (c.JS_IsArray(eng.context, val) != 0 or c.JS_IsObject(val) != 0) {
        const json_str = c.JS_JSONStringify(eng.context, val, engine.makeUndefined(), engine.makeUndefined());
        defer c.JS_FreeValue(eng.context, json_str);

        if (c.JS_IsException(json_str) == 0) {
            const str = c.JS_ToCString(eng.context, json_str);
            if (str != null) {
                print("\x1b[90m{s}\x1b[0m\n", .{str});
                c.JS_FreeCString(eng.context, str);
            }
        }
    } else {
        const str = c.JS_ToCString(eng.context, val);
        if (str != null) {
            print("{s}\n", .{str});
            c.JS_FreeCString(eng.context, str);
        }
    }
}

fn handleBundle(argv: [][:0]const u8) u8 {
    // Parse bundle arguments: bundle <input.js> [-o <output>] [--js-only]
    if (argv.len < 3) {
        print("Error: bundle requires an input file\n", .{});
        print("Usage: kiren bundle <file.js> [-o output]\n", .{});
        return 1;
    }

    const input_file = argv[2];
    var output_file: ?[]const u8 = null;
    var js_only = false;

    // Parse options
    var i: usize = 3;
    while (i < argv.len) : (i += 1) {
        if (std.mem.eql(u8, argv[i], "-o") and i + 1 < argv.len) {
            output_file = argv[i + 1];
            i += 1;
        } else if (std.mem.eql(u8, argv[i], "--js-only")) {
            js_only = true;
        }
    }

    // Default output name: remove .js extension from input
    var default_output_buf: [256]u8 = undefined;
    const default_output = if (output_file) |out| out else blk: {
        const basename = std.fs.path.basename(input_file);
        if (std.mem.endsWith(u8, basename, ".js")) {
            const name_without_ext = basename[0 .. basename.len - 3];
            break :blk std.fmt.bufPrint(&default_output_buf, "{s}", .{name_without_ext}) catch input_file;
        }
        break :blk input_file;
    };

    print("Bundling {s}...\n", .{input_file});

    if (js_only) {
        // Create JS bundle only
        var js_output_buf: [260]u8 = undefined;
        const js_output = std.fmt.bufPrint(&js_output_buf, "{s}.bundle.js", .{default_output}) catch {
            print("Error: output path too long\n", .{});
            return 1;
        };

        bundler.bundleToFile(input_file, js_output) catch |err| {
            print("Error: Failed to bundle: {}\n", .{err});
            return 1;
        };

        print("Created {s}\n", .{js_output});
    } else {
        // Create standalone executable
        bundler.bundleToExecutable(input_file, default_output) catch |err| {
            print("Error: Failed to create executable: {}\n", .{err});
            return 1;
        };

        print("Created {s}\n", .{default_output});
        print("Run with: ./{s}\n", .{default_output});
    }

    return 0;
}

fn runEmbeddedCode(code: []const u8) u8 {
    // Initialize engine
    var eng = Engine.init() catch |err| {
        print("Failed to initialize engine: {}\n", .{err});
        return 1;
    };
    defer eng.deinit();

    // Initialize event loop
    var loop = event_loop.EventLoop.init(std.heap.page_allocator, eng.context) catch |err| {
        print("Failed to initialize event loop: {}\n", .{err});
        return 1;
    };
    defer loop.deinit();
    event_loop.setGlobalEventLoop(&loop);

    // Set process.argv from actual args
    var args = std.process.args();
    var argv_list: std.ArrayListUnmanaged([:0]const u8) = .{};
    defer argv_list.deinit(std.heap.page_allocator);

    while (args.next()) |arg| {
        argv_list.append(std.heap.page_allocator, arg) catch {};
    }
    process.setArgv(argv_list.items);

    // Register APIs
    console.register(&eng);
    process.register(&eng);
    path_mod.register(&eng);
    fs.register(&eng);
    buffer.register(&eng);
    url.register(&eng);
    encoding.register(&eng);
    crypto.register(&eng);
    fetch.register(&eng);
    module.register(&eng);
    http.register(&eng);
    websocket.register(&eng);
    sqlite.register(&eng);
    event_loop.register(&eng);

    // Execute embedded code
    const result = eng.eval(code, "<embedded>");
    if (result) |val| {
        eng.freeValue(val);
        loop.run();
        return 0;
    } else |_| {
        return 1;
    }
}
