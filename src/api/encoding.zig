const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const allocator = std.heap.page_allocator;

// TextEncoder - encodes strings to UTF-8 Uint8Array
fn textEncoderEncode(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        // Return empty Uint8Array
        return c.JS_NewArrayBufferCopy(ctx, null, 0);
    }

    const str = c.JS_ToCString(ctx, argv[0]);
    if (str == null) {
        return c.JS_NewArrayBufferCopy(ctx, null, 0);
    }
    defer c.JS_FreeCString(ctx, str);

    const len = std.mem.len(str);

    // Create Uint8Array directly
    const global = c.JS_GetGlobalObject(ctx);
    defer c.JS_FreeValue(ctx, global);

    const uint8array_ctor = c.JS_GetPropertyStr(ctx, global, "Uint8Array");
    defer c.JS_FreeValue(ctx, uint8array_ctor);

    // Create Uint8Array with size
    var size_arg = [_]c.JSValue{c.JS_NewInt32(ctx, @intCast(len))};
    const result = c.JS_CallConstructor(ctx, uint8array_ctor, 1, &size_arg);
    if (c.JS_IsException(result) != 0) {
        return result;
    }

    // Copy data into the Uint8Array
    var buf_size: usize = 0;
    const buffer_prop = c.JS_GetPropertyStr(ctx, result, "buffer");
    const buf_ptr = c.JS_GetArrayBuffer(ctx, &buf_size, buffer_prop);
    c.JS_FreeValue(ctx, buffer_prop);

    if (buf_ptr != null and buf_size >= len) {
        @memcpy(buf_ptr[0..len], str[0..len]);
    }

    return result;
}

// TextEncoder.encodeInto(string, uint8array)
fn textEncoderEncodeInto(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 2) {
        return c.JS_ThrowTypeError(ctx, "encodeInto requires 2 arguments");
    }

    const str = c.JS_ToCString(ctx, argv[0]);
    if (str == null) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a string");
    }
    defer c.JS_FreeCString(ctx, str);

    const str_len = std.mem.len(str);

    // Get Uint8Array buffer
    var buf_size: usize = 0;
    const buf_ptr = c.JS_GetArrayBuffer(ctx, &buf_size, argv[1]);

    if (buf_ptr == null) {
        // Try to get from Uint8Array
        const buffer_prop = c.JS_GetPropertyStr(ctx, argv[1], "buffer");
        defer c.JS_FreeValue(ctx, buffer_prop);

        var inner_size: usize = 0;
        const inner_ptr = c.JS_GetArrayBuffer(ctx, &inner_size, buffer_prop);

        if (inner_ptr == null) {
            return c.JS_ThrowTypeError(ctx, "Second argument must be a Uint8Array");
        }

        const copy_len = @min(str_len, inner_size);
        if (copy_len > 0) {
            @memcpy(inner_ptr[0..copy_len], str[0..copy_len]);
        }

        // Return result object
        const result = c.JS_NewObject(ctx);
        _ = c.JS_SetPropertyStr(ctx, result, "read", c.JS_NewInt32(ctx, @intCast(copy_len)));
        _ = c.JS_SetPropertyStr(ctx, result, "written", c.JS_NewInt32(ctx, @intCast(copy_len)));
        return result;
    }

    const copy_len = @min(str_len, buf_size);
    if (copy_len > 0) {
        @memcpy(buf_ptr[0..copy_len], str[0..copy_len]);
    }

    // Return result object
    const result = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, result, "read", c.JS_NewInt32(ctx, @intCast(copy_len)));
    _ = c.JS_SetPropertyStr(ctx, result, "written", c.JS_NewInt32(ctx, @intCast(copy_len)));
    return result;
}

// TextDecoder - decodes UTF-8 bytes to string
fn textDecoderDecode(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_NewString(ctx, "");
    }

    // Try to get ArrayBuffer directly
    var buf_size: usize = 0;
    var buf_ptr = c.JS_GetArrayBuffer(ctx, &buf_size, argv[0]);

    if (buf_ptr == null) {
        // Try to get from TypedArray (Uint8Array, etc.)
        const buffer_prop = c.JS_GetPropertyStr(ctx, argv[0], "buffer");
        if (c.JS_IsException(buffer_prop) == 0 and c.JS_IsUndefined(buffer_prop) == 0) {
            buf_ptr = c.JS_GetArrayBuffer(ctx, &buf_size, buffer_prop);
            c.JS_FreeValue(ctx, buffer_prop);
        } else {
            c.JS_FreeValue(ctx, buffer_prop);
        }
    }

    if (buf_ptr == null) {
        // Maybe it's a Buffer from our Buffer API
        // Try to call toString on it
        const to_string = c.JS_GetPropertyStr(ctx, argv[0], "toString");
        if (c.JS_IsFunction(ctx, to_string) != 0) {
            const result = c.JS_Call(ctx, to_string, argv[0], 0, null);
            c.JS_FreeValue(ctx, to_string);
            return result;
        }
        c.JS_FreeValue(ctx, to_string);
        return c.JS_NewString(ctx, "");
    }

    return c.JS_NewStringLen(ctx, @ptrCast(buf_ptr), buf_size);
}

// TextEncoder constructor
fn textEncoderConstructor(ctx: ?*c.JSContext, _: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const obj = c.JS_NewObject(ctx);

    // Add encoding property (always "utf-8")
    _ = c.JS_SetPropertyStr(ctx, obj, "encoding", c.JS_NewString(ctx, "utf-8"));

    // Add methods
    _ = c.JS_SetPropertyStr(ctx, obj, "encode", c.JS_NewCFunction(ctx, textEncoderEncode, "encode", 1));
    _ = c.JS_SetPropertyStr(ctx, obj, "encodeInto", c.JS_NewCFunction(ctx, textEncoderEncodeInto, "encodeInto", 2));

    return obj;
}

// TextDecoder constructor
fn textDecoderConstructor(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const obj = c.JS_NewObject(ctx);

    // Get encoding parameter (default utf-8)
    const encoding: [*c]const u8 = "utf-8";
    if (argc >= 1 and c.JS_IsString(argv[0]) != 0) {
        const enc = c.JS_ToCString(ctx, argv[0]);
        if (enc != null) {
            // For now, we only support utf-8
            c.JS_FreeCString(ctx, enc);
        }
    }

    // Add properties
    _ = c.JS_SetPropertyStr(ctx, obj, "encoding", c.JS_NewString(ctx, encoding));
    _ = c.JS_SetPropertyStr(ctx, obj, "fatal", engine.makeBool(false));
    _ = c.JS_SetPropertyStr(ctx, obj, "ignoreBOM", engine.makeBool(false));

    // Add decode method
    _ = c.JS_SetPropertyStr(ctx, obj, "decode", c.JS_NewCFunction(ctx, textDecoderDecode, "decode", 1));

    return obj;
}

pub fn register(eng: *engine.Engine) void {
    const ctx = eng.context;
    const global = c.JS_GetGlobalObject(ctx);
    defer c.JS_FreeValue(ctx, global);

    // Register TextEncoder and TextDecoder as constructors
    _ = c.JS_SetPropertyStr(ctx, global, "TextEncoder", c.JS_NewCFunction2(ctx, textEncoderConstructor, "TextEncoder", 0, 5, 0));
    _ = c.JS_SetPropertyStr(ctx, global, "TextDecoder", c.JS_NewCFunction2(ctx, textDecoderConstructor, "TextDecoder", 1, 5, 0));
}
