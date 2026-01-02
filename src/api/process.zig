const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const builtin = @import("builtin");

// Store argv globally for access from JS
var global_argv: ?[]const [:0]const u8 = null;

pub fn setArgv(argv: []const [:0]const u8) void {
    global_argv = argv;
}

// process.exit(code)
fn processExit(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv_js: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    var code: i32 = 0;
    if (argc >= 1) {
        _ = c.JS_ToInt32(ctx, &code, argv_js[0]);
    }
    std.process.exit(@intCast(if (code < 0) 1 else @as(u32, @intCast(code))));
}

// process.cwd()
fn processCwd(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    _: c_int,
    _: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();

    var buf: [std.fs.max_path_bytes]u8 = undefined;
    const cwd = std.process.getCwd(&buf) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to get current working directory");
    };

    return c.JS_NewStringLen(context, cwd.ptr, cwd.len);
}

// process.chdir(path)
fn processChdir(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv_js: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "chdir requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv_js[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    var dir = std.fs.cwd().openDir(path, .{}) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to change directory");
    };
    dir.close();

    // Actually change the directory
    std.process.changeCurDir(path) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to change directory");
    };

    return engine.makeUndefined();
}

// Build process.env object
fn buildEnvObject(ctx: *c.JSContext) c.JSValue {
    const env_obj = c.JS_NewObject(ctx);

    var env_map = std.process.getEnvMap(std.heap.page_allocator) catch {
        return env_obj;
    };
    defer env_map.deinit();

    var iter = env_map.iterator();
    while (iter.next()) |entry| {
        const key_z = std.heap.page_allocator.dupeZ(u8, entry.key_ptr.*) catch continue;
        defer std.heap.page_allocator.free(key_z);

        const val = c.JS_NewStringLen(ctx, entry.value_ptr.ptr, entry.value_ptr.len);
        _ = c.JS_SetPropertyStr(ctx, env_obj, key_z.ptr, val);
    }

    return env_obj;
}

// Build process.argv array
fn buildArgvArray(ctx: *c.JSContext) c.JSValue {
    const arr = c.JS_NewArray(ctx);

    if (global_argv) |argv| {
        for (argv, 0..) |arg, i| {
            const str = c.JS_NewString(ctx, arg.ptr);
            _ = c.JS_SetPropertyUint32(ctx, arr, @intCast(i), str);
        }
    }

    return arr;
}

// Get platform string
fn getPlatformString() [:0]const u8 {
    return switch (builtin.os.tag) {
        .macos => "darwin",
        .linux => "linux",
        .windows => "win32",
        .freebsd => "freebsd",
        .openbsd => "openbsd",
        else => "unknown",
    };
}

// Get arch string
fn getArchString() [:0]const u8 {
    return switch (builtin.cpu.arch) {
        .x86_64 => "x64",
        .aarch64 => "arm64",
        .x86 => "ia32",
        .arm => "arm",
        .wasm32 => "wasm32",
        else => "unknown",
    };
}

pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    const process = eng.newObject();

    // Version info
    eng.setProperty(process, "version", eng.newString("v0.1.0"));
    eng.setProperty(process, "versions", createVersionsObject(eng));

    // Platform info
    const platform_str = getPlatformString();
    eng.setProperty(process, "platform", c.JS_NewString(eng.context, platform_str.ptr));

    const arch_str = getArchString();
    eng.setProperty(process, "arch", c.JS_NewString(eng.context, arch_str.ptr));

    // PID (using libc)
    const pid = std.c.getpid();
    eng.setProperty(process, "pid", c.JS_NewInt32(eng.context, @intCast(pid)));

    // Environment variables
    eng.setProperty(process, "env", buildEnvObject(eng.context));

    // Command line arguments
    eng.setProperty(process, "argv", buildArgvArray(eng.context));

    // Functions
    eng.setProperty(process, "exit", eng.newCFunction(processExit, "exit", 1));
    eng.setProperty(process, "cwd", eng.newCFunction(processCwd, "cwd", 0));
    eng.setProperty(process, "chdir", eng.newCFunction(processChdir, "chdir", 1));

    // Register global
    eng.setProperty(global, "process", process);
}

fn createVersionsObject(eng: *engine.Engine) c.JSValue {
    const versions = eng.newObject();
    eng.setProperty(versions, "kiren", eng.newString("0.1.0"));
    eng.setProperty(versions, "quickjs", eng.newString("2024-01-13"));
    eng.setProperty(versions, "zig", eng.newString("0.15.0"));
    return versions;
}
