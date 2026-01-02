const std = @import("std");

pub const c = @cImport({
    @cInclude("quickjs.h");
    @cInclude("quickjs-libc.h");
});

pub const JSValue = c.JSValue;
pub const JSContext = c.JSContext;
pub const JSRuntime = c.JSRuntime;

pub const Engine = struct {
    runtime: *c.JSRuntime,
    context: *c.JSContext,

    pub fn init() !Engine {
        const runtime = c.JS_NewRuntime() orelse {
            return error.RuntimeInitFailed;
        };
        const context = c.JS_NewContext(runtime) orelse {
            c.JS_FreeRuntime(runtime);
            return error.ContextInitFailed;
        };

        // Load standard libraries (Date, JSON, etc.)
        c.js_std_add_helpers(context, 0, null);

        return Engine{
            .runtime = runtime,
            .context = context,
        };
    }

    pub fn deinit(self: *Engine) void {
        c.JS_FreeContext(self.context);
        c.JS_FreeRuntime(self.runtime);
    }

    pub fn eval(self: *Engine, code: []const u8, filename: [:0]const u8) !JSValue {
        const result = c.JS_Eval(
            self.context,
            code.ptr,
            code.len,
            filename.ptr,
            c.JS_EVAL_TYPE_GLOBAL,
        );

        if (c.JS_IsException(result) != 0) {
            self.dumpError();
            return error.EvalFailed;
        }

        return result;
    }

    pub fn evalFile(self: *Engine, filepath: []const u8) !JSValue {
        const allocator = std.heap.page_allocator;

        // Create null-terminated filepath
        const filepath_z = allocator.allocSentinel(u8, filepath.len, 0) catch {
            return error.OutOfMemory;
        };
        defer allocator.free(filepath_z);
        @memcpy(filepath_z, filepath);

        const file = std.fs.cwd().openFile(filepath_z, .{}) catch |err| {
            std.debug.print("Failed to open file: {s}\n", .{filepath});
            return err;
        };
        defer file.close();

        const stat = file.stat() catch |err| {
            std.debug.print("Failed to get file size: {s}\n", .{filepath});
            return err;
        };
        const file_size = stat.size;

        const code = allocator.alloc(u8, file_size) catch {
            return error.OutOfMemory;
        };
        defer allocator.free(code);

        const bytes_read = file.readAll(code) catch |err| {
            std.debug.print("Failed to read file: {s}\n", .{filepath});
            return err;
        };
        _ = bytes_read;

        return self.eval(code, filepath_z);
    }

    pub fn dumpError(self: *Engine) void {
        const exception = c.JS_GetException(self.context);
        defer c.JS_FreeValue(self.context, exception);

        const str = c.JS_ToCString(self.context, exception);
        if (str != null) {
            std.debug.print("Error: {s}\n", .{str});
            c.JS_FreeCString(self.context, str);
        }

        // Stack trace
        if (c.JS_IsError(self.context, exception) != 0) {
            const stack = c.JS_GetPropertyStr(self.context, exception, "stack");
            if (c.JS_IsUndefined(stack) == 0) {
                const stack_str = c.JS_ToCString(self.context, stack);
                if (stack_str != null) {
                    std.debug.print("{s}\n", .{stack_str});
                    c.JS_FreeCString(self.context, stack_str);
                }
            }
            c.JS_FreeValue(self.context, stack);
        }
    }

    pub fn freeValue(self: *Engine, value: JSValue) void {
        c.JS_FreeValue(self.context, value);
    }

    pub fn getGlobalObject(self: *Engine) JSValue {
        return c.JS_GetGlobalObject(self.context);
    }

    pub fn setProperty(self: *Engine, obj: JSValue, name: [*:0]const u8, value: JSValue) void {
        _ = c.JS_SetPropertyStr(self.context, obj, name, value);
    }

    pub fn newCFunction(self: *Engine, func: c.JSCFunction, name: [*:0]const u8, arg_count: c_int) JSValue {
        return c.JS_NewCFunction(self.context, func, name, arg_count);
    }

    pub fn newObject(self: *Engine) JSValue {
        return c.JS_NewObject(self.context);
    }

    pub fn newString(self: *Engine, str: []const u8) JSValue {
        return c.JS_NewStringLen(self.context, str.ptr, str.len);
    }

    pub fn toString(self: *Engine, value: JSValue) ?[]const u8 {
        var len: usize = 0;
        const str = c.JS_ToCStringLen(self.context, &len, value);
        if (str == null) return null;
        return str[0..len];
    }

    pub fn freeCString(self: *Engine, str: [*:0]const u8) void {
        c.JS_FreeCString(self.context, str);
    }
};

// Utility functions
pub fn isUndefined(value: JSValue) bool {
    return c.JS_IsUndefined(value) != 0;
}

pub fn isNull(value: JSValue) bool {
    return c.JS_IsNull(value) != 0;
}

pub fn isException(value: JSValue) bool {
    return c.JS_IsException(value) != 0;
}

pub fn isString(value: JSValue) bool {
    return c.JS_IsString(value) != 0;
}

pub fn isNumber(value: JSValue) bool {
    return c.JS_IsNumber(value) != 0;
}

pub fn isBool(value: JSValue) bool {
    return c.JS_IsBool(value) != 0;
}

pub fn isObject(value: JSValue) bool {
    return c.JS_IsObject(value) != 0;
}

pub fn isArray(ctx: *JSContext, value: JSValue) bool {
    return c.JS_IsArray(ctx, value) != 0;
}

pub fn isFunction(ctx: *JSContext, value: JSValue) bool {
    return c.JS_IsFunction(ctx, value) != 0;
}

// Manual JS_UNDEFINED and JS_NULL definition (for comptime macro issues)
pub const JS_TAG_UNDEFINED: i64 = 3;
pub const JS_TAG_NULL: i64 = 2;

pub fn makeUndefined() JSValue {
    return JSValue{
        .u = .{ .int32 = 0 },
        .tag = JS_TAG_UNDEFINED,
    };
}

pub fn makeNull() JSValue {
    return JSValue{
        .u = .{ .int32 = 0 },
        .tag = JS_TAG_NULL,
    };
}

pub const JS_TAG_BOOL: i64 = 1;

pub fn makeBool(val: bool) JSValue {
    return JSValue{
        .u = .{ .int32 = if (val) 1 else 0 },
        .tag = JS_TAG_BOOL,
    };
}
