const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;

fn print(comptime fmt: []const u8, args: anytype) void {
    std.debug.print(fmt, args);
}

fn printValue(ctx: *c.JSContext, val: c.JSValue, depth: u8) void {
    if (depth > 5) {
        print("[...]", .{});
        return;
    }

    const tag = c.JS_VALUE_GET_TAG(val);

    // Undefined
    if (c.JS_IsUndefined(val) != 0) {
        if (depth == 0) {
            print("undefined", .{});
        } else {
            print("\x1b[90mundefined\x1b[0m", .{});
        }
        return;
    }

    // Null
    if (c.JS_IsNull(val) != 0) {
        if (depth == 0) {
            print("null", .{});
        } else {
            print("\x1b[1mnull\x1b[0m", .{});
        }
        return;
    }

    // Boolean
    if (c.JS_IsBool(val) != 0) {
        const b = c.JS_ToBool(ctx, val);
        if (depth == 0) {
            if (b != 0) {
                print("true", .{});
            } else {
                print("false", .{});
            }
        } else {
            if (b != 0) {
                print("\x1b[33mtrue\x1b[0m", .{});
            } else {
                print("\x1b[33mfalse\x1b[0m", .{});
            }
        }
        return;
    }

    // Number
    if (c.JS_IsNumber(val) != 0) {
        var num: f64 = 0;
        _ = c.JS_ToFloat64(ctx, &num, val);
        if (@floor(num) == num and num < 1e15 and num > -1e15) {
            if (depth == 0) {
                print("{d}", .{@as(i64, @intFromFloat(num))});
            } else {
                print("\x1b[33m{d}\x1b[0m", .{@as(i64, @intFromFloat(num))});
            }
        } else {
            if (depth == 0) {
                print("{d}", .{num});
            } else {
                print("\x1b[33m{d}\x1b[0m", .{num});
            }
        }
        return;
    }

    // String
    if (c.JS_IsString(val) != 0) {
        const str = c.JS_ToCString(ctx, val);
        if (str != null) {
            if (depth == 0) {
                print("{s}", .{str});
            } else {
                print("\x1b[32m\"{s}\"\x1b[0m", .{str});
            }
            c.JS_FreeCString(ctx, str);
        }
        return;
    }

    // Function
    if (c.JS_IsFunction(ctx, val) != 0) {
        print("\x1b[36m[Function]\x1b[0m", .{});
        return;
    }

    // Array
    if (c.JS_IsArray(ctx, val) != 0) {
        const len_val = c.JS_GetPropertyStr(ctx, val, "length");
        defer c.JS_FreeValue(ctx, len_val);

        var len: i64 = 0;
        _ = c.JS_ToInt64(ctx, &len, len_val);

        print("[ ", .{});

        var i: u32 = 0;
        while (i < @as(u32, @intCast(len))) : (i += 1) {
            if (i > 0) print(", ", .{});
            if (i > 99) {
                print("... +{d} more", .{len - i});
                break;
            }
            const elem = c.JS_GetPropertyUint32(ctx, val, i);
            defer c.JS_FreeValue(ctx, elem);
            printValue(ctx, elem, depth + 1);
        }

        print(" ]", .{});
        return;
    }

    // Object
    if (c.JS_IsObject(val) != 0) {
        // Check for special objects
        if (tag == c.JS_TAG_OBJECT) {
            print("{{ ", .{});

            var ptab: [*c]c.JSPropertyEnum = undefined;
            var plen: u32 = 0;

            const ret = c.JS_GetOwnPropertyNames(ctx, &ptab, &plen, val, c.JS_GPN_STRING_MASK | c.JS_GPN_ENUM_ONLY);
            if (ret == 0) {
                var count: u32 = 0;
                var idx: u32 = 0;
                while (idx < plen) : (idx += 1) {
                    if (count > 0) print(", ", .{});
                    if (count > 10) {
                        print("... +{d} more", .{plen - idx});
                        break;
                    }

                    const atom = ptab[idx].atom;
                    const key = c.JS_AtomToCString(ctx, atom);
                    if (key != null) {
                        print("{s}: ", .{key});
                        c.JS_FreeCString(ctx, key);
                    }

                    const prop = c.JS_GetProperty(ctx, val, atom);
                    defer c.JS_FreeValue(ctx, prop);
                    printValue(ctx, prop, depth + 1);

                    c.JS_FreeAtom(ctx, atom);
                    count += 1;
                }
                c.js_free(ctx, ptab);
            }

            print(" }}", .{});
            return;
        }
    }

    // Fallback: toString
    const str = c.JS_ToCString(ctx, val);
    if (str != null) {
        print("{s}", .{str});
        c.JS_FreeCString(ctx, str);
    }
}

fn formatArgs(ctx: ?*c.JSContext, argc: c_int, argv: [*c]c.JSValue) void {
    const context = ctx orelse return;
    var i: usize = 0;
    while (i < @as(usize, @intCast(argc))) : (i += 1) {
        if (i > 0) print(" ", .{});
        printValue(context, argv[i], 0);
    }
}

fn consoleLog(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    formatArgs(ctx, argc, argv);
    print("\n", .{});
    return engine.makeUndefined();
}

fn consoleWarn(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    print("\x1b[33m", .{});
    formatArgs(ctx, argc, argv);
    print("\x1b[0m\n", .{});
    return engine.makeUndefined();
}

fn consoleError(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    print("\x1b[31m", .{});
    formatArgs(ctx, argc, argv);
    print("\x1b[0m\n", .{});
    return engine.makeUndefined();
}

fn consoleInfo(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    print("\x1b[36m", .{});
    formatArgs(ctx, argc, argv);
    print("\x1b[0m\n", .{});
    return engine.makeUndefined();
}

fn consoleDebug(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    print("\x1b[90m[DEBUG] ", .{});
    formatArgs(ctx, argc, argv);
    print("\x1b[0m\n", .{});
    return engine.makeUndefined();
}

pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    const console = eng.newObject();

    eng.setProperty(console, "log", eng.newCFunction(consoleLog, "log", 1));
    eng.setProperty(console, "warn", eng.newCFunction(consoleWarn, "warn", 1));
    eng.setProperty(console, "error", eng.newCFunction(consoleError, "error", 1));
    eng.setProperty(console, "info", eng.newCFunction(consoleInfo, "info", 1));
    eng.setProperty(console, "debug", eng.newCFunction(consoleDebug, "debug", 1));

    eng.setProperty(global, "console", console);
}
