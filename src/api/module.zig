const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;

const Allocator = std.mem.Allocator;

// Module cache - stores already loaded modules
var module_cache: std.StringHashMapUnmanaged(c.JSValue) = .{};
var cache_initialized: bool = false;
var global_context: ?*c.JSContext = null;
var global_allocator: Allocator = undefined;

// Current module directory stack for resolving relative paths
var module_dir_stack: std.ArrayListUnmanaged([]const u8) = .{};
var stack_initialized: bool = false;

fn initCache(allocator: Allocator) void {
    if (!cache_initialized) {
        global_allocator = allocator;
        cache_initialized = true;
    }
    if (!stack_initialized) {
        stack_initialized = true;
    }
}

fn getCurrentDir() []const u8 {
    if (module_dir_stack.items.len > 0) {
        return module_dir_stack.items[module_dir_stack.items.len - 1];
    }
    return ".";
}

// Resolve module path
fn resolvePath(allocator: Allocator, request: []const u8) ![]u8 {
    // If starts with ./ or ../ it's relative
    if (std.mem.startsWith(u8, request, "./") or std.mem.startsWith(u8, request, "../")) {
        const current_dir = getCurrentDir();
        const full_path = try std.fs.path.join(allocator, &[_][]const u8{ current_dir, request });

        // Try with .js extension if not present
        if (!std.mem.endsWith(u8, full_path, ".js") and !std.mem.endsWith(u8, full_path, ".json")) {
            const with_js = try std.fmt.allocPrint(allocator, "{s}.js", .{full_path});
            allocator.free(full_path);
            return with_js;
        }
        return full_path;
    }

    // Absolute path
    if (std.mem.startsWith(u8, request, "/")) {
        return try allocator.dupe(u8, request);
    }

    // Node module (from node_modules) - not implemented yet
    // For now, treat as relative to current dir
    const full_path = try std.fs.path.join(allocator, &[_][]const u8{ getCurrentDir(), request });
    if (!std.mem.endsWith(u8, full_path, ".js") and !std.mem.endsWith(u8, full_path, ".json")) {
        const with_js = try std.fmt.allocPrint(allocator, "{s}.js", .{full_path});
        allocator.free(full_path);
        return with_js;
    }
    return full_path;
}

// require() implementation
fn jsRequire(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();
    global_context = context;

    const allocator = std.heap.page_allocator;
    initCache(allocator);

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "require() needs a module path");
    }

    const path_str = c.JS_ToCString(ctx, argv[0]);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Module path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const request = std.mem.span(path_str);

    // Resolve the full path
    const full_path = resolvePath(allocator, request) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to resolve module path");
    };
    defer allocator.free(full_path);

    // Check cache
    if (module_cache.get(full_path)) |cached| {
        return c.JS_DupValue(ctx, cached);
    }

    // Read the file
    const file = std.fs.cwd().openFile(full_path, .{}) catch {
        var err_buf: [256]u8 = undefined;
        const err_msg = std.fmt.bufPrint(&err_buf, "Cannot find module '{s}'", .{request}) catch "Cannot find module";
        _ = err_msg;
        return c.JS_ThrowInternalError(ctx, "Cannot find module");
    };
    defer file.close();

    const stat = file.stat() catch {
        return c.JS_ThrowInternalError(ctx, "Failed to stat module file");
    };

    const content = allocator.alloc(u8, stat.size) catch {
        return c.JS_ThrowInternalError(ctx, "Out of memory");
    };
    defer allocator.free(content);

    _ = file.readAll(content) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to read module file");
    };

    // Handle JSON files
    if (std.mem.endsWith(u8, full_path, ".json")) {
        const result = c.JS_ParseJSON(context, content.ptr, content.len, full_path.ptr);
        if (c.JS_IsException(result) != 0) {
            return result;
        }
        // Cache and return
        const cache_key = allocator.dupe(u8, full_path) catch {
            return result;
        };
        module_cache.put(allocator, cache_key, c.JS_DupValue(ctx, result)) catch {};
        return result;
    }

    // Get directory of current module for nested requires
    const module_dir = std.fs.path.dirname(full_path) orelse ".";
    const dir_copy = allocator.dupe(u8, module_dir) catch {
        return c.JS_ThrowInternalError(ctx, "Out of memory");
    };
    module_dir_stack.append(allocator, dir_copy) catch {};
    defer {
        if (module_dir_stack.pop()) |popped| {
            allocator.free(popped);
        }
    }

    // Create module wrapper
    // (function(exports, require, module, __filename, __dirname) { ... })
    const wrapper_prefix = "(function(exports, require, module, __filename, __dirname) {\n";
    const wrapper_suffix = "\n})";

    const wrapped = std.fmt.allocPrint(allocator, "{s}{s}{s}", .{
        wrapper_prefix,
        content,
        wrapper_suffix,
    }) catch {
        return c.JS_ThrowInternalError(ctx, "Out of memory");
    };
    defer allocator.free(wrapped);

    // Create null-terminated path for eval
    const path_z = allocator.allocSentinel(u8, full_path.len, 0) catch {
        return c.JS_ThrowInternalError(ctx, "Out of memory");
    };
    defer allocator.free(path_z);
    @memcpy(path_z, full_path);

    // Evaluate the wrapper function
    const func = c.JS_Eval(context, wrapped.ptr, wrapped.len, path_z.ptr, c.JS_EVAL_TYPE_GLOBAL);
    if (c.JS_IsException(func) != 0) {
        return func;
    }
    defer c.JS_FreeValue(context, func);

    // Create module object
    const module_obj = c.JS_NewObject(context);
    const exports_obj = c.JS_NewObject(context);
    _ = c.JS_SetPropertyStr(context, module_obj, "exports", c.JS_DupValue(ctx, exports_obj));

    // Create __filename and __dirname strings
    const filename_str = c.JS_NewStringLen(context, full_path.ptr, full_path.len);
    const dirname_str = c.JS_NewStringLen(context, module_dir.ptr, module_dir.len);

    // Get require function from global
    const global = c.JS_GetGlobalObject(context);
    defer c.JS_FreeValue(context, global);
    const require_func = c.JS_GetPropertyStr(context, global, "require");

    // Call the wrapper function
    var args = [_]c.JSValue{
        exports_obj,
        require_func,
        module_obj,
        filename_str,
        dirname_str,
    };

    const call_result = c.JS_Call(context, func, engine.makeUndefined(), 5, &args);

    // Clean up args (except exports and module which we still need)
    c.JS_FreeValue(context, require_func);
    c.JS_FreeValue(context, filename_str);
    c.JS_FreeValue(context, dirname_str);

    if (c.JS_IsException(call_result) != 0) {
        c.JS_FreeValue(context, exports_obj);
        c.JS_FreeValue(context, module_obj);
        return call_result;
    }
    c.JS_FreeValue(context, call_result);

    // Get module.exports (might have been replaced)
    const final_exports = c.JS_GetPropertyStr(context, module_obj, "exports");
    c.JS_FreeValue(context, exports_obj);
    c.JS_FreeValue(context, module_obj);

    // Cache the module
    const cache_key = allocator.dupe(u8, full_path) catch {
        return final_exports;
    };
    module_cache.put(allocator, cache_key, c.JS_DupValue(ctx, final_exports)) catch {};

    return final_exports;
}

pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    // Register require function
    eng.setProperty(global, "require", eng.newCFunction(jsRequire, "require", 1));

    // Set initial module directory based on cwd
    const allocator = std.heap.page_allocator;
    initCache(allocator);

    var cwd_buf: [std.fs.max_path_bytes]u8 = undefined;
    const cwd = std.process.getCwd(&cwd_buf) catch ".";
    const cwd_copy = allocator.dupe(u8, cwd) catch return;
    module_dir_stack.append(allocator, cwd_copy) catch {};
}

pub fn setModuleDir(dir: []const u8) void {
    const allocator = std.heap.page_allocator;
    initCache(allocator);

    // Clear existing and set new
    for (module_dir_stack.items) |item| {
        allocator.free(item);
    }
    module_dir_stack.clearRetainingCapacity();

    const dir_copy = allocator.dupe(u8, dir) catch return;
    module_dir_stack.append(allocator, dir_copy) catch {};
}
