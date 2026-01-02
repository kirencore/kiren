const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const allocator = std.heap.page_allocator;

// URLSearchParams class
var urlsearchparams_class_id: c.JSClassID = 0;
var urlsearchparams_proto: c.JSValue = undefined;

const SearchParamsData = struct {
    params: std.StringHashMapUnmanaged(std.ArrayListUnmanaged([]const u8)),
};

fn getSearchParamsData(ctx: ?*c.JSContext, this: c.JSValue) ?*SearchParamsData {
    const ptr = c.JS_GetOpaque(this, urlsearchparams_class_id);
    if (ptr == null) {
        _ = c.JS_ThrowTypeError(ctx, "Not a URLSearchParams");
        return null;
    }
    return @ptrCast(@alignCast(ptr));
}

fn searchParamsFinalizer(_: ?*c.JSRuntime, val: c.JSValue) callconv(.c) void {
    const ptr = c.JS_GetOpaque(val, urlsearchparams_class_id);
    if (ptr != null) {
        const data: *SearchParamsData = @ptrCast(@alignCast(ptr));
        var it = data.params.iterator();
        while (it.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            for (entry.value_ptr.items) |v| {
                allocator.free(v);
            }
            entry.value_ptr.deinit(allocator);
        }
        data.params.deinit(allocator);
        allocator.destroy(data);
    }
}

fn parseQueryString(query: []const u8) SearchParamsData {
    var data = SearchParamsData{
        .params = .{},
    };

    // Split by &
    var pairs = std.mem.splitScalar(u8, query, '&');
    while (pairs.next()) |pair| {
        if (pair.len == 0) continue;

        // Split by =
        var parts = std.mem.splitScalar(u8, pair, '=');
        const key = parts.next() orelse continue;
        const value = parts.next() orelse "";

        // URL decode and store
        const key_copy = allocator.dupe(u8, key) catch continue;
        const value_copy = allocator.dupe(u8, value) catch {
            allocator.free(key_copy);
            continue;
        };

        if (data.params.getPtr(key_copy)) |list| {
            list.append(allocator, value_copy) catch {};
            allocator.free(key_copy);
        } else {
            var list: std.ArrayListUnmanaged([]const u8) = .{};
            list.append(allocator, value_copy) catch {
                allocator.free(key_copy);
                allocator.free(value_copy);
                continue;
            };
            data.params.put(allocator, key_copy, list) catch {
                allocator.free(key_copy);
                allocator.free(value_copy);
                list.deinit(allocator);
            };
        }
    }

    return data;
}

fn createSearchParams(ctx: ?*c.JSContext, query: []const u8) c.JSValue {
    const data = allocator.create(SearchParamsData) catch {
        return c.JS_ThrowOutOfMemory(ctx);
    };
    data.* = parseQueryString(query);

    const obj = c.JS_NewObjectClass(ctx, @intCast(urlsearchparams_class_id));
    if (c.JS_IsException(obj) != 0) {
        allocator.destroy(data);
        return obj;
    }

    c.JS_SetOpaque(obj, data);
    _ = c.JS_SetPrototype(ctx, obj, urlsearchparams_proto);

    return obj;
}

// URLSearchParams constructor
fn urlSearchParamsConstructor(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    var query: []const u8 = "";

    if (argc >= 1 and c.JS_IsString(argv[0]) != 0) {
        const str = c.JS_ToCString(ctx, argv[0]);
        if (str != null) {
            const str_slice = str[0..std.mem.len(str)];
            // Remove leading ? if present
            if (str_slice.len > 0 and str_slice[0] == '?') {
                query = str_slice[1..];
            } else {
                query = str_slice;
            }
            const result = createSearchParams(ctx, query);
            c.JS_FreeCString(ctx, str);
            return result;
        }
    }

    return createSearchParams(ctx, query);
}

// searchParams.get(name)
fn searchParamsGet(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return engine.makeNull();

    if (argc < 1) return engine.makeNull();

    const key = c.JS_ToCString(ctx, argv[0]);
    if (key == null) return engine.makeNull();
    defer c.JS_FreeCString(ctx, key);

    const key_slice = key[0..std.mem.len(key)];

    if (data.params.get(key_slice)) |list| {
        if (list.items.len > 0) {
            return c.JS_NewStringLen(ctx, @ptrCast(list.items[0].ptr), list.items[0].len);
        }
    }

    return engine.makeNull();
}

// searchParams.getAll(name)
fn searchParamsGetAll(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return engine.makeNull();

    if (argc < 1) {
        return c.JS_NewArray(ctx);
    }

    const key = c.JS_ToCString(ctx, argv[0]);
    if (key == null) return c.JS_NewArray(ctx);
    defer c.JS_FreeCString(ctx, key);

    const key_slice = key[0..std.mem.len(key)];

    const arr = c.JS_NewArray(ctx);

    if (data.params.get(key_slice)) |list| {
        var i: u32 = 0;
        for (list.items) |value| {
            _ = c.JS_SetPropertyUint32(ctx, arr, i, c.JS_NewStringLen(ctx, @ptrCast(value.ptr), value.len));
            i += 1;
        }
    }

    return arr;
}

// searchParams.has(name)
fn searchParamsHas(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return engine.makeBool(false);

    if (argc < 1) return engine.makeBool(false);

    const key = c.JS_ToCString(ctx, argv[0]);
    if (key == null) return engine.makeBool(false);
    defer c.JS_FreeCString(ctx, key);

    const key_slice = key[0..std.mem.len(key)];
    return engine.makeBool(data.params.get(key_slice) != null);
}

// searchParams.set(name, value)
fn searchParamsSet(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return engine.makeUndefined();

    if (argc < 2) return engine.makeUndefined();

    const key = c.JS_ToCString(ctx, argv[0]);
    if (key == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, key);

    const value = c.JS_ToCString(ctx, argv[1]);
    if (value == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, value);

    const key_slice = key[0..std.mem.len(key)];
    const key_copy = allocator.dupe(u8, key_slice) catch return engine.makeUndefined();
    const value_copy = allocator.dupe(u8, value[0..std.mem.len(value)]) catch {
        allocator.free(key_copy);
        return engine.makeUndefined();
    };

    // Remove existing
    if (data.params.getPtr(key_slice)) |list| {
        for (list.items) |v| {
            allocator.free(v);
        }
        list.clearRetainingCapacity();
        list.append(allocator, value_copy) catch {};
        allocator.free(key_copy);
    } else {
        var list: std.ArrayListUnmanaged([]const u8) = .{};
        list.append(allocator, value_copy) catch {
            allocator.free(key_copy);
            allocator.free(value_copy);
            return engine.makeUndefined();
        };
        data.params.put(allocator, key_copy, list) catch {};
    }

    return engine.makeUndefined();
}

// searchParams.append(name, value)
fn searchParamsAppend(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return engine.makeUndefined();

    if (argc < 2) return engine.makeUndefined();

    const key = c.JS_ToCString(ctx, argv[0]);
    if (key == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, key);

    const value = c.JS_ToCString(ctx, argv[1]);
    if (value == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, value);

    const key_slice = key[0..std.mem.len(key)];
    const value_copy = allocator.dupe(u8, value[0..std.mem.len(value)]) catch return engine.makeUndefined();

    if (data.params.getPtr(key_slice)) |list| {
        list.append(allocator, value_copy) catch {
            allocator.free(value_copy);
        };
    } else {
        const key_copy = allocator.dupe(u8, key_slice) catch {
            allocator.free(value_copy);
            return engine.makeUndefined();
        };
        var list: std.ArrayListUnmanaged([]const u8) = .{};
        list.append(allocator, value_copy) catch {
            allocator.free(key_copy);
            allocator.free(value_copy);
            return engine.makeUndefined();
        };
        data.params.put(allocator, key_copy, list) catch {};
    }

    return engine.makeUndefined();
}

// searchParams.delete(name)
fn searchParamsDelete(ctx: ?*c.JSContext, this: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return engine.makeUndefined();

    if (argc < 1) return engine.makeUndefined();

    const key = c.JS_ToCString(ctx, argv[0]);
    if (key == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, key);

    const key_slice = key[0..std.mem.len(key)];

    if (data.params.fetchRemove(key_slice)) |kv| {
        allocator.free(kv.key);
        for (kv.value.items) |v| {
            allocator.free(v);
        }
        var list = kv.value;
        list.deinit(allocator);
    }

    return engine.makeUndefined();
}

// searchParams.toString()
fn searchParamsToString(ctx: ?*c.JSContext, this: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getSearchParamsData(ctx, this) orelse return c.JS_NewString(ctx, "");

    var result = std.ArrayListUnmanaged(u8){};
    defer result.deinit(allocator);

    var first = true;
    var it = data.params.iterator();
    while (it.next()) |entry| {
        for (entry.value_ptr.items) |value| {
            if (!first) {
                result.append(allocator, '&') catch {};
            }
            first = false;
            result.appendSlice(allocator, entry.key_ptr.*) catch {};
            result.append(allocator, '=') catch {};
            result.appendSlice(allocator, value) catch {};
        }
    }

    return c.JS_NewStringLen(ctx, @ptrCast(result.items.ptr), result.items.len);
}

// URL class
var url_class_id: c.JSClassID = 0;
var url_proto: c.JSValue = undefined;

const UrlData = struct {
    href: []const u8,
    protocol: []const u8,
    hostname: []const u8,
    port: []const u8,
    pathname: []const u8,
    search: []const u8,
    hash: []const u8,
    search_params: c.JSValue,
};

fn getUrlData(ctx: ?*c.JSContext, this: c.JSValue) ?*UrlData {
    const ptr = c.JS_GetOpaque(this, url_class_id);
    if (ptr == null) {
        _ = c.JS_ThrowTypeError(ctx, "Not a URL");
        return null;
    }
    return @ptrCast(@alignCast(ptr));
}

fn urlFinalizer(rt: ?*c.JSRuntime, val: c.JSValue) callconv(.c) void {
    const ptr = c.JS_GetOpaque(val, url_class_id);
    if (ptr != null) {
        const data: *UrlData = @ptrCast(@alignCast(ptr));
        allocator.free(data.href);
        allocator.free(data.protocol);
        allocator.free(data.hostname);
        allocator.free(data.port);
        allocator.free(data.pathname);
        allocator.free(data.search);
        allocator.free(data.hash);
        // Free search_params
        c.JS_FreeValueRT(rt, data.search_params);
        allocator.destroy(data);
    }
}

fn parseUrl(ctx: ?*c.JSContext, url_str: []const u8) ?*UrlData {
    const data = allocator.create(UrlData) catch return null;

    // Store full href
    data.href = allocator.dupe(u8, url_str) catch {
        allocator.destroy(data);
        return null;
    };

    var remaining = url_str;

    // Parse protocol (scheme://)
    if (std.mem.indexOf(u8, remaining, "://")) |proto_end| {
        data.protocol = allocator.dupe(u8, remaining[0 .. proto_end + 1]) catch {
            allocator.free(data.href);
            allocator.destroy(data);
            return null;
        };
        remaining = remaining[proto_end + 3 ..];
    } else {
        data.protocol = allocator.dupe(u8, "") catch {
            allocator.free(data.href);
            allocator.destroy(data);
            return null;
        };
    }

    // Find path start
    const path_start = std.mem.indexOfScalar(u8, remaining, '/') orelse remaining.len;
    const host_part = remaining[0..path_start];

    // Parse hostname:port
    if (std.mem.lastIndexOfScalar(u8, host_part, ':')) |port_sep| {
        data.hostname = allocator.dupe(u8, host_part[0..port_sep]) catch "";
        data.port = allocator.dupe(u8, host_part[port_sep + 1 ..]) catch "";
    } else {
        data.hostname = allocator.dupe(u8, host_part) catch "";
        data.port = allocator.dupe(u8, "") catch "";
    }

    remaining = if (path_start < remaining.len) remaining[path_start..] else "";

    // Parse hash
    if (std.mem.indexOfScalar(u8, remaining, '#')) |hash_start| {
        data.hash = allocator.dupe(u8, remaining[hash_start..]) catch "";
        remaining = remaining[0..hash_start];
    } else {
        data.hash = allocator.dupe(u8, "") catch "";
    }

    // Parse search/query
    if (std.mem.indexOfScalar(u8, remaining, '?')) |query_start| {
        data.search = allocator.dupe(u8, remaining[query_start..]) catch "";
        data.pathname = allocator.dupe(u8, remaining[0..query_start]) catch "/";

        // Create searchParams
        data.search_params = createSearchParams(ctx, remaining[query_start + 1 ..]);
    } else {
        data.search = allocator.dupe(u8, "") catch "";
        data.pathname = allocator.dupe(u8, if (remaining.len > 0) remaining else "/") catch "/";
        data.search_params = createSearchParams(ctx, "");
    }

    return data;
}

// URL constructor
fn urlConstructor(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "URL constructor requires a URL string");
    }

    const url_str = c.JS_ToCString(ctx, argv[0]);
    if (url_str == null) {
        return c.JS_ThrowTypeError(ctx, "URL must be a string");
    }
    defer c.JS_FreeCString(ctx, url_str);

    const url_slice = url_str[0..std.mem.len(url_str)];

    const data = parseUrl(ctx, url_slice) orelse {
        return c.JS_ThrowTypeError(ctx, "Invalid URL");
    };

    const obj = c.JS_NewObjectClass(ctx, @intCast(url_class_id));
    if (c.JS_IsException(obj) != 0) {
        allocator.destroy(data);
        return obj;
    }

    c.JS_SetOpaque(obj, data);
    _ = c.JS_SetPrototype(ctx, obj, url_proto);

    // Set properties
    _ = c.JS_SetPropertyStr(ctx, obj, "href", c.JS_NewStringLen(ctx, @ptrCast(data.href.ptr), data.href.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "protocol", c.JS_NewStringLen(ctx, @ptrCast(data.protocol.ptr), data.protocol.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "hostname", c.JS_NewStringLen(ctx, @ptrCast(data.hostname.ptr), data.hostname.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "host", c.JS_NewStringLen(ctx, @ptrCast(data.hostname.ptr), data.hostname.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "port", c.JS_NewStringLen(ctx, @ptrCast(data.port.ptr), data.port.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "pathname", c.JS_NewStringLen(ctx, @ptrCast(data.pathname.ptr), data.pathname.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "search", c.JS_NewStringLen(ctx, @ptrCast(data.search.ptr), data.search.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "hash", c.JS_NewStringLen(ctx, @ptrCast(data.hash.ptr), data.hash.len));
    _ = c.JS_SetPropertyStr(ctx, obj, "searchParams", data.search_params);

    // origin = protocol + // + host
    var origin_buf: [512]u8 = undefined;
    const origin = std.fmt.bufPrint(&origin_buf, "{s}//{s}", .{ data.protocol, data.hostname }) catch "";
    _ = c.JS_SetPropertyStr(ctx, obj, "origin", c.JS_NewStringLen(ctx, @ptrCast(origin.ptr), origin.len));

    return obj;
}

// url.toString()
fn urlToString(ctx: ?*c.JSContext, this: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getUrlData(ctx, this) orelse return c.JS_NewString(ctx, "");
    return c.JS_NewStringLen(ctx, @ptrCast(data.href.ptr), data.href.len);
}

pub fn register(eng: *engine.Engine) void {
    const ctx = eng.context;

    // Create URLSearchParams class
    urlsearchparams_class_id = c.JS_NewClassID(&urlsearchparams_class_id);

    const searchparams_class_def = c.JSClassDef{
        .class_name = "URLSearchParams",
        .finalizer = searchParamsFinalizer,
        .gc_mark = null,
        .call = null,
        .exotic = null,
    };
    _ = c.JS_NewClass(c.JS_GetRuntime(ctx), urlsearchparams_class_id, &searchparams_class_def);

    urlsearchparams_proto = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "get", c.JS_NewCFunction(ctx, searchParamsGet, "get", 1));
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "getAll", c.JS_NewCFunction(ctx, searchParamsGetAll, "getAll", 1));
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "has", c.JS_NewCFunction(ctx, searchParamsHas, "has", 1));
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "set", c.JS_NewCFunction(ctx, searchParamsSet, "set", 2));
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "append", c.JS_NewCFunction(ctx, searchParamsAppend, "append", 2));
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "delete", c.JS_NewCFunction(ctx, searchParamsDelete, "delete", 1));
    _ = c.JS_SetPropertyStr(ctx, urlsearchparams_proto, "toString", c.JS_NewCFunction(ctx, searchParamsToString, "toString", 0));

    // Create URL class
    url_class_id = c.JS_NewClassID(&url_class_id);

    const url_class_def = c.JSClassDef{
        .class_name = "URL",
        .finalizer = urlFinalizer,
        .gc_mark = null,
        .call = null,
        .exotic = null,
    };
    _ = c.JS_NewClass(c.JS_GetRuntime(ctx), url_class_id, &url_class_def);

    url_proto = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, url_proto, "toString", c.JS_NewCFunction(ctx, urlToString, "toString", 0));

    // Register constructors globally (JS_CFUNC_constructor_or_func = 5)
    const global = c.JS_GetGlobalObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, global, "URL", c.JS_NewCFunction2(ctx, urlConstructor, "URL", 1, 5, 0));
    _ = c.JS_SetPropertyStr(ctx, global, "URLSearchParams", c.JS_NewCFunction2(ctx, urlSearchParamsConstructor, "URLSearchParams", 1, 5, 0));
    c.JS_FreeValue(ctx, global);
}
