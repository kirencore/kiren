const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;

const Allocator = std.mem.Allocator;

// fs.readFileSync(path, options?)
fn fsReadFileSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "readFileSync requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    // Check encoding option
    var encoding: ?[]const u8 = null;
    if (argc >= 2) {
        if (c.JS_IsString(argv[1]) != 0) {
            const enc_str = c.JS_ToCString(ctx, argv[1]);
            if (enc_str != null) {
                encoding = std.mem.span(enc_str);
                c.JS_FreeCString(ctx, enc_str);
            }
        } else if (c.JS_IsObject(argv[1]) != 0) {
            const enc_val = c.JS_GetPropertyStr(ctx, argv[1], "encoding");
            if (c.JS_IsString(enc_val) != 0) {
                const enc_str = c.JS_ToCString(ctx, enc_val);
                if (enc_str != null) {
                    encoding = std.mem.span(enc_str);
                    c.JS_FreeCString(ctx, enc_str);
                }
            }
            c.JS_FreeValue(ctx, enc_val);
        }
    }

    const allocator = std.heap.page_allocator;

    // Open and read file
    const file = std.fs.cwd().openFile(path, .{}) catch |err| {
        return throwFsError(ctx, "readFileSync", path, err);
    };
    defer file.close();

    const stat = file.stat() catch |err| {
        return throwFsError(ctx, "readFileSync", path, err);
    };

    const content = allocator.alloc(u8, stat.size) catch {
        return c.JS_ThrowInternalError(ctx, "Out of memory");
    };
    defer allocator.free(content);

    _ = file.readAll(content) catch |err| {
        return throwFsError(ctx, "readFileSync", path, err);
    };

    // Return as string if encoding specified, otherwise as string (default utf8)
    // TODO: Support Buffer return when encoding is null
    if (encoding) |_| {
        // With encoding, return string
    }
    return c.JS_NewStringLen(context, content.ptr, content.len);
}

// fs.writeFileSync(path, data, options?)
fn fsWriteFileSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 2) {
        return c.JS_ThrowTypeError(ctx, "writeFileSync requires path and data arguments");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    // Get data to write
    const data_str = c.JS_ToCString(ctx, argv[1]);
    if (data_str == null) {
        return c.JS_ThrowTypeError(ctx, "Data must be a string");
    }
    defer c.JS_FreeCString(ctx, data_str);

    const data = std.mem.span(data_str);

    // Write file
    const file = std.fs.cwd().createFile(path, .{}) catch |err| {
        return throwFsError(ctx, "writeFileSync", path, err);
    };
    defer file.close();

    file.writeAll(data) catch |err| {
        return throwFsError(ctx, "writeFileSync", path, err);
    };

    return engine.makeUndefined();
}

// fs.existsSync(path)
fn fsExistsSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return engine.makeBool(false);
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return engine.makeBool(false);
    }
    defer c.JS_FreeCString(ctx, path_str);

    const filepath = std.mem.span(path_str);

    // Check if path exists
    std.fs.cwd().access(filepath, .{}) catch {
        return engine.makeBool(false);
    };

    return engine.makeBool(true);
}

// fs.mkdirSync(path, options?)
fn fsMkdirSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "mkdirSync requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    // Check for recursive option
    var recursive = false;
    if (argc >= 2 and c.JS_IsObject(argv[1]) != 0) {
        const rec_val = c.JS_GetPropertyStr(ctx, argv[1], "recursive");
        if (c.JS_IsBool(rec_val) != 0) {
            recursive = c.JS_ToBool(ctx, rec_val) != 0;
        }
        c.JS_FreeValue(ctx, rec_val);
    }

    if (recursive) {
        std.fs.cwd().makePath(path) catch |err| {
            return throwFsError(ctx, "mkdirSync", path, err);
        };
    } else {
        std.fs.cwd().makeDir(path) catch |err| {
            return throwFsError(ctx, "mkdirSync", path, err);
        };
    }

    return engine.makeUndefined();
}

// fs.rmdirSync(path)
fn fsRmdirSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "rmdirSync requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    std.fs.cwd().deleteDir(path) catch |err| {
        return throwFsError(ctx, "rmdirSync", path, err);
    };

    return engine.makeUndefined();
}

// fs.unlinkSync(path)
fn fsUnlinkSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "unlinkSync requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    std.fs.cwd().deleteFile(path) catch |err| {
        return throwFsError(ctx, "unlinkSync", path, err);
    };

    return engine.makeUndefined();
}

// fs.readdirSync(path)
fn fsReaddirSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "readdirSync requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    var dir = std.fs.cwd().openDir(path, .{ .iterate = true }) catch |err| {
        return throwFsError(ctx, "readdirSync", path, err);
    };
    defer dir.close();

    const arr = c.JS_NewArray(context);
    var idx: u32 = 0;

    var iter = dir.iterate();
    while (iter.next() catch null) |entry| {
        const name = c.JS_NewString(context, entry.name.ptr);
        _ = c.JS_SetPropertyUint32(context, arr, idx, name);
        idx += 1;
    }

    return arr;
}

// fs.statSync(path)
fn fsStatSync(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "statSync requires a path argument");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    const stat = std.fs.cwd().statFile(path) catch |err| {
        return throwFsError(ctx, "statSync", path, err);
    };

    const obj = c.JS_NewObject(context);

    // Size
    _ = c.JS_SetPropertyStr(context, obj, "size", c.JS_NewInt64(context, @intCast(stat.size)));

    // File type checks
    const is_file = stat.kind == .file;
    const is_dir = stat.kind == .directory;
    const is_symlink = stat.kind == .sym_link;

    // Add isFile(), isDirectory(), isSymbolicLink() methods
    // For simplicity, we add boolean properties
    _ = c.JS_SetPropertyStr(context, obj, "isFile", engine.makeBool(is_file));
    _ = c.JS_SetPropertyStr(context, obj, "isDirectory", engine.makeBool(is_dir));
    _ = c.JS_SetPropertyStr(context, obj, "isSymbolicLink", engine.makeBool(is_symlink));

    return obj;
}

fn throwFsError(ctx: ?*c.JSContext, operation: []const u8, path: []const u8, err: anyerror) c.JSValue {
    _ = operation;
    _ = path;

    const msg = switch (err) {
        error.FileNotFound => "ENOENT: no such file or directory",
        error.AccessDenied => "EACCES: permission denied",
        error.PathAlreadyExists => "EEXIST: file already exists",
        error.NotDir => "ENOTDIR: not a directory",
        error.IsDir => "EISDIR: illegal operation on a directory",
        error.DirNotEmpty => "ENOTEMPTY: directory not empty",
        else => "Unknown error",
    };

    return c.JS_ThrowInternalError(ctx, msg);
}

pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    const fs = eng.newObject();

    eng.setProperty(fs, "readFileSync", eng.newCFunction(fsReadFileSync, "readFileSync", 2));
    eng.setProperty(fs, "writeFileSync", eng.newCFunction(fsWriteFileSync, "writeFileSync", 3));
    eng.setProperty(fs, "existsSync", eng.newCFunction(fsExistsSync, "existsSync", 1));
    eng.setProperty(fs, "mkdirSync", eng.newCFunction(fsMkdirSync, "mkdirSync", 2));
    eng.setProperty(fs, "rmdirSync", eng.newCFunction(fsRmdirSync, "rmdirSync", 1));
    eng.setProperty(fs, "unlinkSync", eng.newCFunction(fsUnlinkSync, "unlinkSync", 1));
    eng.setProperty(fs, "readdirSync", eng.newCFunction(fsReaddirSync, "readdirSync", 1));
    eng.setProperty(fs, "statSync", eng.newCFunction(fsStatSync, "statSync", 1));

    eng.setProperty(global, "fs", fs);
}
