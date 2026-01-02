const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;

// crypto.randomUUID() - Generate UUID v4
fn cryptoRandomUUID(ctx: ?*c.JSContext, _: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    var uuid: [36]u8 = undefined;

    // Get random bytes
    var random_bytes: [16]u8 = undefined;
    std.crypto.random.bytes(&random_bytes);

    // Set version (4) and variant bits
    random_bytes[6] = (random_bytes[6] & 0x0f) | 0x40; // Version 4
    random_bytes[8] = (random_bytes[8] & 0x3f) | 0x80; // Variant 1

    // Format as UUID string
    _ = std.fmt.bufPrint(&uuid, "{x:0>2}{x:0>2}{x:0>2}{x:0>2}-{x:0>2}{x:0>2}-{x:0>2}{x:0>2}-{x:0>2}{x:0>2}-{x:0>2}{x:0>2}{x:0>2}{x:0>2}{x:0>2}{x:0>2}", .{
        random_bytes[0],  random_bytes[1],  random_bytes[2],  random_bytes[3],
        random_bytes[4],  random_bytes[5],
        random_bytes[6],  random_bytes[7],
        random_bytes[8],  random_bytes[9],
        random_bytes[10], random_bytes[11], random_bytes[12], random_bytes[13], random_bytes[14], random_bytes[15],
    }) catch return c.JS_ThrowInternalError(ctx, "Failed to format UUID");

    return c.JS_NewStringLen(ctx, &uuid, 36);
}

// crypto.getRandomValues(typedArray) - Fill typed array with random values
fn cryptoGetRandomValues(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "getRandomValues requires a TypedArray argument");
    }

    // Get the buffer from the typed array
    const buffer_prop = c.JS_GetPropertyStr(ctx, argv[0], "buffer");
    defer c.JS_FreeValue(ctx, buffer_prop);

    var buf_size: usize = 0;
    const buf_ptr = c.JS_GetArrayBuffer(ctx, &buf_size, buffer_prop);

    if (buf_ptr == null) {
        return c.JS_ThrowTypeError(ctx, "Argument must be a TypedArray");
    }

    // Fill with random bytes
    std.crypto.random.bytes(buf_ptr[0..buf_size]);

    return argv[0];
}

// crypto.randomBytes(size) - Node.js compatible
fn cryptoRandomBytes(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "randomBytes requires size argument");
    }

    var size: i32 = 0;
    if (c.JS_ToInt32(ctx, &size, argv[0]) < 0 or size < 0) {
        return c.JS_ThrowTypeError(ctx, "Size must be a positive number");
    }

    // Create Uint8Array
    const global = c.JS_GetGlobalObject(ctx);
    defer c.JS_FreeValue(ctx, global);

    const uint8array_ctor = c.JS_GetPropertyStr(ctx, global, "Uint8Array");
    defer c.JS_FreeValue(ctx, uint8array_ctor);

    var size_arg = [_]c.JSValue{c.JS_NewInt32(ctx, size)};
    const result = c.JS_CallConstructor(ctx, uint8array_ctor, 1, &size_arg);
    if (c.JS_IsException(result) != 0) {
        return result;
    }

    // Get buffer and fill with random data
    const buffer_prop = c.JS_GetPropertyStr(ctx, result, "buffer");
    defer c.JS_FreeValue(ctx, buffer_prop);

    var buf_size: usize = 0;
    const buf_ptr = c.JS_GetArrayBuffer(ctx, &buf_size, buffer_prop);

    if (buf_ptr != null) {
        std.crypto.random.bytes(buf_ptr[0..buf_size]);
    }

    return result;
}

pub fn register(eng: *engine.Engine) void {
    const ctx = eng.context;
    const global = c.JS_GetGlobalObject(ctx);
    defer c.JS_FreeValue(ctx, global);

    // Create crypto object
    const crypto_obj = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, crypto_obj, "randomUUID", c.JS_NewCFunction(ctx, cryptoRandomUUID, "randomUUID", 0));
    _ = c.JS_SetPropertyStr(ctx, crypto_obj, "getRandomValues", c.JS_NewCFunction(ctx, cryptoGetRandomValues, "getRandomValues", 1));
    _ = c.JS_SetPropertyStr(ctx, crypto_obj, "randomBytes", c.JS_NewCFunction(ctx, cryptoRandomBytes, "randomBytes", 1));

    _ = c.JS_SetPropertyStr(ctx, global, "crypto", crypto_obj);
}
