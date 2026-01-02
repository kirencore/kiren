const std = @import("std");
const Engine = @import("engine.zig").Engine;
const engine = @import("engine.zig");
const console = @import("api/console.zig");

const VERSION = "0.1.0";

fn print(comptime fmt: []const u8, args: anytype) void {
    std.debug.print(fmt, args);
}

fn printUsage() void {
    print(
        \\Kiren - JavaScript Runtime
        \\
        \\Usage:
        \\  kiren <file.js>       Run a JavaScript file
        \\  kiren -e <code>       Evaluate inline JavaScript
        \\  kiren --version       Show version info
        \\  kiren --help          Show this help message
        \\
        \\Examples:
        \\  kiren script.js
        \\  kiren -e "console.log('Hello!')"
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
    var args = std.process.args();
    _ = args.next(); // skip executable name

    const first_arg = args.next();

    if (first_arg == null) {
        printUsage();
        return 0;
    }

    const arg = first_arg.?;

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

    // Register APIs
    console.register(&eng);

    if (std.mem.eql(u8, arg, "-e")) {
        // Execute inline code
        const code = args.next() orelse {
            fatal("Error: code expected after -e\n", .{});
        };

        const result = eng.eval(code, "<inline>");
        if (result) |val| {
            // Show result (if not undefined)
            if (engine.c.JS_IsUndefined(val) == 0) {
                printResult(&eng, val);
            }
            eng.freeValue(val);
            return 0;
        } else |_| {
            return 1;
        }
    } else {
        // Execute file
        const result = eng.evalFile(arg);
        if (result) |val| {
            eng.freeValue(val);
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
