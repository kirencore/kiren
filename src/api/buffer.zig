const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const allocator = std.heap.page_allocator;

// Buffer class prototype
var buffer_class_id: c.JSClassID = 0;
var buffer_proto: c.JSValue = undefined;

const BufferData = struct {
    data: []u8,
    len: usize,
};

fn getBufferData(ctx: ?*c.JSContext, this: c.JSValue) ?*BufferData {
    const ptr = c.JS_GetOpaque(this, buffer_class_id);
    if (ptr == null) {
        _ = c.JS_ThrowTypeError(ctx, "Not a Buffer");
        return null;
    }
    return @ptrCast(@alignCast(ptr));
}

fn bufferFinalizer(rt: ?*c.JSRuntime, val: c.JSValue) callconv(.c) void {
    _ = rt;
    const ptr = c.JS_GetOpaque(val, buffer_class_id);
    if (ptr != null) {
        const buf_data: *BufferData = @ptrCast(@alignCast(ptr));
        allocator.free(buf_data.data);
        allocator.destroy(buf_data);
    }
}

fn createBuffer(ctx: ?*c.JSContext, size: usize) c.JSValue {
    const data = allocator.alloc(u8, size) catch {
        return c.JS_ThrowOutOfMemory(ctx);
    };
    @memset(data, 0);

    const buf_data = allocator.create(BufferData) catch {
        allocator.free(data);
        return c.JS_ThrowOutOfMemory(ctx);
    };
    buf_data.* = .{
        .data = data,
        .len = size,
    };

    const obj = c.JS_NewObjectClass(ctx, @intCast(buffer_class_id));
    if (c.JS_IsException(obj) != 0) {
        allocator.free(data);
        allocator.destroy(buf_data);
        return obj;
    }

    c.JS_SetOpaque(obj, buf_data);
    _ = c.JS_SetPrototype(ctx, obj, buffer_proto);

    // Set length property
    _ = c.JS_SetPropertyStr(ctx, obj, "length", c.JS_NewInt32(ctx, @intCast(size)));

    return obj;
}

fn createBufferFromData(ctx: ?*c.JSContext, src_data: []const u8) c.JSValue {
    const size = src_data.len;
    const data = allocator.alloc(u8, size) catch {
        return c.JS_ThrowOutOfMemory(ctx);
    };
    @memcpy(data, src_data);

    const buf_data = allocator.create(BufferData) catch {
        allocator.free(data);
        return c.JS_ThrowOutOfMemory(ctx);
    };
    buf_data.* = .{
        .data = data,
        .len = size,
    };

    const obj = c.JS_NewObjectClass(ctx, @intCast(buffer_class_id));
    if (c.JS_IsException(obj) != 0) {
        allocator.free(data);
        allocator.destroy(buf_data);
        return obj;
    }

    c.JS_SetOpaque(obj, buf_data);
    _ = c.JS_SetPrototype(ctx, obj, buffer_proto);

    _ = c.JS_SetPropertyStr(ctx, obj, "length", c.JS_NewInt32(ctx, @intCast(size)));

    return obj;
}

// Buffer.alloc(size)
fn bufferAlloc(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "Buffer.alloc requires size argument");
    }

    var size: i32 = 0;
    if (c.JS_ToInt32(ctx, &size, argv[0]) < 0) {
        return c.JS_ThrowTypeError(ctx, "Size must be a number");
    }

    if (size < 0) {
        return c.JS_ThrowRangeError(ctx, "Buffer size must be non-negative");
    }

    return createBuffer(ctx, @intCast(size));
}

// Buffer.from(data, encoding)
fn bufferFrom(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "Buffer.from requires data argument");
    }

    const first_arg = argv[0];

    // Check if it's a string
    if (c.JS_IsString(first_arg) != 0) {
        const str = c.JS_ToCString(ctx, first_arg);
        if (str == null) {
            return c.JS_ThrowTypeError(ctx, "Failed to convert to string");
        }
        defer c.JS_FreeCString(ctx, str);

        const len = std.mem.len(str);
        return createBufferFromData(ctx, str[0..len]);
    }

    // Check if it's an array
    if (c.JS_IsArray(ctx, first_arg) != 0) {
        var arr_len: i64 = 0;
        const len_val = c.JS_GetPropertyStr(ctx, first_arg, "length");
        _ = c.JS_ToInt64(ctx, &arr_len, len_val);
        c.JS_FreeValue(ctx, len_val);

        const data = allocator.alloc(u8, @intCast(arr_len)) catch {
            return c.JS_ThrowOutOfMemory(ctx);
        };

        var i: u32 = 0;
        while (i < @as(u32, @intCast(arr_len))) : (i += 1) {
            const elem = c.JS_GetPropertyUint32(ctx, first_arg, i);
            var val: i32 = 0;
            _ = c.JS_ToInt32(ctx, &val, elem);
            c.JS_FreeValue(ctx, elem);
            data[i] = @intCast(val & 0xFF);
        }

        const buf_data = allocator.create(BufferData) catch {
            allocator.free(data);
            return c.JS_ThrowOutOfMemory(ctx);
        };
        buf_data.* = .{
            .data = data,
            .len = @intCast(arr_len),
        };

        const obj = c.JS_NewObjectClass(ctx, @intCast(buffer_class_id));
        if (c.JS_IsException(obj) != 0) {
            allocator.free(data);
            allocator.destroy(buf_data);
            return obj;
        }

        c.JS_SetOpaque(obj, buf_data);
        _ = c.JS_SetPrototype(ctx, obj, buffer_proto);
        _ = c.JS_SetPropertyStr(ctx, obj, "length", c.JS_NewInt32(ctx, @intCast(arr_len)));

        return obj;
    }

    // Check if it's another Buffer
    const ptr = c.JS_GetOpaque(first_arg, buffer_class_id);
    if (ptr != null) {
        const src_buf: *BufferData = @ptrCast(@alignCast(ptr));
        return createBufferFromData(ctx, src_buf.data[0..src_buf.len]);
    }

    return c.JS_ThrowTypeError(ctx, "Buffer.from argument must be string, array or Buffer");
}

// buffer.toString(encoding)
fn bufferToString(ctx: ?*c.JSContext, this: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    // For now, just return UTF-8 string
    return c.JS_NewStringLen(ctx, @ptrCast(buf_data.data.ptr), buf_data.len);
}

// buffer.slice(start, end)
fn bufferSlice(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    var start: i32 = 0;
    var end: i32 = @intCast(buf_data.len);

    if (argc >= 1) {
        _ = c.JS_ToInt32(ctx, &start, argv[0]);
    }
    if (argc >= 2) {
        _ = c.JS_ToInt32(ctx, &end, argv[1]);
    }

    // Handle negative indices
    if (start < 0) start = @intCast(@max(0, @as(i32, @intCast(buf_data.len)) + start));
    if (end < 0) end = @intCast(@max(0, @as(i32, @intCast(buf_data.len)) + end));

    // Clamp to buffer bounds
    start = @min(start, @as(i32, @intCast(buf_data.len)));
    end = @min(end, @as(i32, @intCast(buf_data.len)));

    if (start >= end) {
        return createBuffer(ctx, 0);
    }

    const ustart: usize = @intCast(start);
    const uend: usize = @intCast(end);

    return createBufferFromData(ctx, buf_data.data[ustart..uend]);
}

// buffer.copy(target, targetStart, sourceStart, sourceEnd)
fn bufferCopy(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const src_data = getBufferData(ctx, this) orelse return engine.makeException();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "copy requires target Buffer");
    }

    const target_ptr = c.JS_GetOpaque(argv[0], buffer_class_id);
    if (target_ptr == null) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a Buffer");
    }
    const target_data: *BufferData = @ptrCast(@alignCast(target_ptr));

    var target_start: i32 = 0;
    var source_start: i32 = 0;
    var source_end: i32 = @intCast(src_data.len);

    if (argc >= 2) _ = c.JS_ToInt32(ctx, &target_start, argv[1]);
    if (argc >= 3) _ = c.JS_ToInt32(ctx, &source_start, argv[2]);
    if (argc >= 4) _ = c.JS_ToInt32(ctx, &source_end, argv[3]);

    // Clamp values
    target_start = @max(0, @min(target_start, @as(i32, @intCast(target_data.len))));
    source_start = @max(0, @min(source_start, @as(i32, @intCast(src_data.len))));
    source_end = @max(source_start, @min(source_end, @as(i32, @intCast(src_data.len))));

    const utarget_start: usize = @intCast(target_start);
    const usource_start: usize = @intCast(source_start);
    const usource_end: usize = @intCast(source_end);

    const copy_len = @min(usource_end - usource_start, target_data.len - utarget_start);

    if (copy_len > 0) {
        @memcpy(
            target_data.data[utarget_start .. utarget_start + copy_len],
            src_data.data[usource_start .. usource_start + copy_len],
        );
    }

    return c.JS_NewInt32(ctx, @intCast(copy_len));
}

// buffer.write(string, offset, length, encoding)
fn bufferWrite(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "write requires string argument");
    }

    const str = c.JS_ToCString(ctx, argv[0]);
    if (str == null) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a string");
    }
    defer c.JS_FreeCString(ctx, str);

    var offset: i32 = 0;
    if (argc >= 2) {
        _ = c.JS_ToInt32(ctx, &offset, argv[1]);
    }

    if (offset < 0 or offset >= @as(i32, @intCast(buf_data.len))) {
        return c.JS_NewInt32(ctx, 0);
    }

    const str_len = std.mem.len(str);
    const uoffset: usize = @intCast(offset);
    const write_len = @min(str_len, buf_data.len - uoffset);

    if (write_len > 0) {
        @memcpy(buf_data.data[uoffset .. uoffset + write_len], str[0..write_len]);
    }

    return c.JS_NewInt32(ctx, @intCast(write_len));
}

// buffer.readUInt8(offset)
fn bufferReadUInt8(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    var offset: i32 = 0;
    if (argc >= 1) {
        _ = c.JS_ToInt32(ctx, &offset, argv[0]);
    }

    if (offset < 0 or offset >= @as(i32, @intCast(buf_data.len))) {
        return c.JS_ThrowRangeError(ctx, "Offset out of bounds");
    }

    return c.JS_NewInt32(ctx, buf_data.data[@intCast(offset)]);
}

// buffer.writeUInt8(value, offset)
fn bufferWriteUInt8(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "writeUInt8 requires value");
    }

    var value: i32 = 0;
    var offset: i32 = 0;

    _ = c.JS_ToInt32(ctx, &value, argv[0]);
    if (argc >= 2) {
        _ = c.JS_ToInt32(ctx, &offset, argv[1]);
    }

    if (offset < 0 or offset >= @as(i32, @intCast(buf_data.len))) {
        return c.JS_ThrowRangeError(ctx, "Offset out of bounds");
    }

    buf_data.data[@intCast(offset)] = @intCast(value & 0xFF);
    return c.JS_NewInt32(ctx, offset + 1);
}

// buffer.equals(otherBuffer)
fn bufferEquals(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "equals requires Buffer argument");
    }

    const other_ptr = c.JS_GetOpaque(argv[0], buffer_class_id);
    if (other_ptr == null) {
        return engine.makeBool(false);
    }

    const other_data: *BufferData = @ptrCast(@alignCast(other_ptr));

    if (buf_data.len != other_data.len) {
        return engine.makeBool(false);
    }

    const equal = std.mem.eql(u8, buf_data.data[0..buf_data.len], other_data.data[0..other_data.len]);
    return engine.makeBool(equal);
}

// buffer.fill(value, offset, end)
fn bufferFill(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    if (argc < 1) {
        return this;
    }

    var fill_val: u8 = 0;

    // Check if string or number
    if (c.JS_IsString(argv[0]) != 0) {
        const str = c.JS_ToCString(ctx, argv[0]);
        if (str != null and std.mem.len(str) > 0) {
            fill_val = str[0];
        }
        if (str != null) c.JS_FreeCString(ctx, str);
    } else {
        var val: i32 = 0;
        _ = c.JS_ToInt32(ctx, &val, argv[0]);
        fill_val = @intCast(val & 0xFF);
    }

    var offset: i32 = 0;
    var end: i32 = @intCast(buf_data.len);

    if (argc >= 2) _ = c.JS_ToInt32(ctx, &offset, argv[1]);
    if (argc >= 3) _ = c.JS_ToInt32(ctx, &end, argv[2]);

    offset = @max(0, @min(offset, @as(i32, @intCast(buf_data.len))));
    end = @max(offset, @min(end, @as(i32, @intCast(buf_data.len))));

    const uoffset: usize = @intCast(offset);
    const uend: usize = @intCast(end);

    @memset(buf_data.data[uoffset..uend], fill_val);

    return this;
}

// Buffer.concat(buffers, totalLength)
fn bufferConcat(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return createBuffer(ctx, 0);
    }

    if (c.JS_IsArray(ctx, argv[0]) == 0) {
        return c.JS_ThrowTypeError(ctx, "Buffer.concat requires array");
    }

    var arr_len: i64 = 0;
    const len_val = c.JS_GetPropertyStr(ctx, argv[0], "length");
    _ = c.JS_ToInt64(ctx, &arr_len, len_val);
    c.JS_FreeValue(ctx, len_val);

    // Calculate total length
    var total_len: usize = 0;
    var i: u32 = 0;
    while (i < @as(u32, @intCast(arr_len))) : (i += 1) {
        const elem = c.JS_GetPropertyUint32(ctx, argv[0], i);
        const ptr = c.JS_GetOpaque(elem, buffer_class_id);
        if (ptr != null) {
            const buf: *BufferData = @ptrCast(@alignCast(ptr));
            total_len += buf.len;
        }
        c.JS_FreeValue(ctx, elem);
    }

    // Create result buffer
    const data = allocator.alloc(u8, total_len) catch {
        return c.JS_ThrowOutOfMemory(ctx);
    };

    // Copy data
    var offset: usize = 0;
    i = 0;
    while (i < @as(u32, @intCast(arr_len))) : (i += 1) {
        const elem = c.JS_GetPropertyUint32(ctx, argv[0], i);
        const ptr = c.JS_GetOpaque(elem, buffer_class_id);
        if (ptr != null) {
            const buf: *BufferData = @ptrCast(@alignCast(ptr));
            @memcpy(data[offset .. offset + buf.len], buf.data[0..buf.len]);
            offset += buf.len;
        }
        c.JS_FreeValue(ctx, elem);
    }

    const buf_data = allocator.create(BufferData) catch {
        allocator.free(data);
        return c.JS_ThrowOutOfMemory(ctx);
    };
    buf_data.* = .{
        .data = data,
        .len = total_len,
    };

    const obj = c.JS_NewObjectClass(ctx, @intCast(buffer_class_id));
    if (c.JS_IsException(obj) != 0) {
        allocator.free(data);
        allocator.destroy(buf_data);
        return obj;
    }

    c.JS_SetOpaque(obj, buf_data);
    _ = c.JS_SetPrototype(ctx, obj, buffer_proto);
    _ = c.JS_SetPropertyStr(ctx, obj, "length", c.JS_NewInt32(ctx, @intCast(total_len)));

    return obj;
}

// Buffer.isBuffer(obj)
fn bufferIsBuffer(_: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return engine.makeBool(false);
    }

    const ptr = c.JS_GetOpaque(argv[0], buffer_class_id);
    return engine.makeBool(ptr != null);
}

// Buffer.byteLength(string, encoding)
fn bufferByteLength(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_NewInt32(ctx, 0);
    }

    // Check if it's a Buffer
    const ptr = c.JS_GetOpaque(argv[0], buffer_class_id);
    if (ptr != null) {
        const buf: *BufferData = @ptrCast(@alignCast(ptr));
        return c.JS_NewInt32(ctx, @intCast(buf.len));
    }

    // Handle string
    if (c.JS_IsString(argv[0]) != 0) {
        const str = c.JS_ToCString(ctx, argv[0]);
        if (str == null) return c.JS_NewInt32(ctx, 0);
        defer c.JS_FreeCString(ctx, str);
        return c.JS_NewInt32(ctx, @intCast(std.mem.len(str)));
    }

    return c.JS_NewInt32(ctx, 0);
}

// Index getter for buffer[i]
fn bufferIndexGetter(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const buf_data = getBufferData(ctx, this) orelse return engine.makeException();

    if (argc < 1) {
        return engine.makeUndefined();
    }

    var index: i32 = 0;
    _ = c.JS_ToInt32(ctx, &index, argv[0]);

    if (index < 0 or index >= @as(i32, @intCast(buf_data.len))) {
        return engine.makeUndefined();
    }

    return c.JS_NewInt32(ctx, buf_data.data[@intCast(index)]);
}

pub fn register(eng: *engine.Engine) void {
    const ctx = eng.context;
    const rt = c.JS_GetRuntime(ctx);

    // Create Buffer class
    buffer_class_id = c.JS_NewClassID(&buffer_class_id);

    const class_def = c.JSClassDef{
        .class_name = "Buffer",
        .finalizer = bufferFinalizer,
        .gc_mark = null,
        .call = null,
        .exotic = null,
    };
    _ = c.JS_NewClass(rt, buffer_class_id, &class_def);

    // Create prototype with methods
    buffer_proto = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "toString", c.JS_NewCFunction(ctx, bufferToString, "toString", 1));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "slice", c.JS_NewCFunction(ctx, bufferSlice, "slice", 2));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "subarray", c.JS_NewCFunction(ctx, bufferSlice, "subarray", 2));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "copy", c.JS_NewCFunction(ctx, bufferCopy, "copy", 4));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "write", c.JS_NewCFunction(ctx, bufferWrite, "write", 4));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "readUInt8", c.JS_NewCFunction(ctx, bufferReadUInt8, "readUInt8", 1));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "writeUInt8", c.JS_NewCFunction(ctx, bufferWriteUInt8, "writeUInt8", 2));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "equals", c.JS_NewCFunction(ctx, bufferEquals, "equals", 1));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "fill", c.JS_NewCFunction(ctx, bufferFill, "fill", 3));
    _ = c.JS_SetPropertyStr(ctx, buffer_proto, "at", c.JS_NewCFunction(ctx, bufferIndexGetter, "at", 1));

    // Create Buffer constructor object
    const buffer_ctor = c.JS_NewObject(ctx);

    // Static methods
    _ = c.JS_SetPropertyStr(ctx, buffer_ctor, "alloc", c.JS_NewCFunction(ctx, bufferAlloc, "alloc", 1));
    _ = c.JS_SetPropertyStr(ctx, buffer_ctor, "from", c.JS_NewCFunction(ctx, bufferFrom, "from", 2));
    _ = c.JS_SetPropertyStr(ctx, buffer_ctor, "concat", c.JS_NewCFunction(ctx, bufferConcat, "concat", 2));
    _ = c.JS_SetPropertyStr(ctx, buffer_ctor, "isBuffer", c.JS_NewCFunction(ctx, bufferIsBuffer, "isBuffer", 1));
    _ = c.JS_SetPropertyStr(ctx, buffer_ctor, "byteLength", c.JS_NewCFunction(ctx, bufferByteLength, "byteLength", 2));

    // Set Buffer globally
    const global = c.JS_GetGlobalObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, global, "Buffer", buffer_ctor);
    c.JS_FreeValue(ctx, global);
}
