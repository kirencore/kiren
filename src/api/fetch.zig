const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const allocator = std.heap.page_allocator;

// Response class for fetch
var response_class_id: c.JSClassID = 0;
var response_proto: c.JSValue = undefined;

const ResponseData = struct {
    status: u16,
    status_text: []const u8,
    headers: std.StringHashMapUnmanaged([]const u8),
    body: []const u8,
    ok: bool,
    url: []const u8,
};

fn getResponseData(ctx: ?*c.JSContext, this: c.JSValue) ?*ResponseData {
    const ptr = c.JS_GetOpaque(this, response_class_id);
    if (ptr == null) {
        _ = c.JS_ThrowTypeError(ctx, "Not a Response");
        return null;
    }
    return @ptrCast(@alignCast(ptr));
}

fn responseFinalizer(_: ?*c.JSRuntime, val: c.JSValue) callconv(.c) void {
    const ptr = c.JS_GetOpaque(val, response_class_id);
    if (ptr != null) {
        const data: *ResponseData = @ptrCast(@alignCast(ptr));
        allocator.free(data.body);
        allocator.free(data.url);
        allocator.free(data.status_text);
        var it = data.headers.iterator();
        while (it.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        data.headers.deinit(allocator);
        allocator.destroy(data);
    }
}

fn createResponse(ctx: ?*c.JSContext, status: u16, body: []const u8, url: []const u8, headers: std.StringHashMapUnmanaged([]const u8)) c.JSValue {
    const data = allocator.create(ResponseData) catch {
        return c.JS_ThrowOutOfMemory(ctx);
    };

    data.* = .{
        .status = status,
        .status_text = allocator.dupe(u8, getStatusText(status)) catch "",
        .body = allocator.dupe(u8, body) catch "",
        .url = allocator.dupe(u8, url) catch "",
        .headers = headers,
        .ok = status >= 200 and status < 300,
    };

    const obj = c.JS_NewObjectClass(ctx, @intCast(response_class_id));
    if (c.JS_IsException(obj) != 0) {
        allocator.destroy(data);
        return obj;
    }

    c.JS_SetOpaque(obj, data);
    _ = c.JS_SetPrototype(ctx, obj, response_proto);

    // Set properties
    _ = c.JS_SetPropertyStr(ctx, obj, "status", c.JS_NewInt32(ctx, status));
    _ = c.JS_SetPropertyStr(ctx, obj, "statusText", c.JS_NewString(ctx, data.status_text.ptr));
    _ = c.JS_SetPropertyStr(ctx, obj, "ok", engine.makeBool(data.ok));
    _ = c.JS_SetPropertyStr(ctx, obj, "url", c.JS_NewStringLen(ctx, @ptrCast(data.url.ptr), data.url.len));

    // Create headers object
    const headers_obj = c.JS_NewObject(ctx);
    var it = data.headers.iterator();
    while (it.next()) |entry| {
        _ = c.JS_SetPropertyStr(ctx, headers_obj, @ptrCast(entry.key_ptr.*.ptr), c.JS_NewStringLen(ctx, @ptrCast(entry.value_ptr.*.ptr), entry.value_ptr.*.len));
    }
    _ = c.JS_SetPropertyStr(ctx, obj, "headers", headers_obj);

    return obj;
}

fn getStatusText(status: u16) []const u8 {
    return switch (status) {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        else => "Unknown",
    };
}

// response.text()
fn responseText(ctx: ?*c.JSContext, this: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getResponseData(ctx, this) orelse return engine.makeNull();
    return c.JS_NewStringLen(ctx, @ptrCast(data.body.ptr), data.body.len);
}

// response.json()
fn responseJson(ctx: ?*c.JSContext, this: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getResponseData(ctx, this) orelse return engine.makeNull();
    return c.JS_ParseJSON(ctx, @ptrCast(data.body.ptr), data.body.len, "<json>");
}

// response.arrayBuffer()
fn responseArrayBuffer(ctx: ?*c.JSContext, this: c.JSValue, _: c_int, _: [*c]c.JSValue) callconv(.c) c.JSValue {
    const data = getResponseData(ctx, this) orelse return engine.makeNull();
    return c.JS_NewArrayBufferCopy(ctx, @ptrCast(data.body.ptr), data.body.len);
}

// Parse URL into host, port, path
fn parseUrl(url_str: []const u8) ?struct { host: []const u8, port: u16, path: []const u8, is_https: bool } {
    var remaining = url_str;
    var is_https = false;

    // Parse protocol
    if (std.mem.startsWith(u8, remaining, "https://")) {
        is_https = true;
        remaining = remaining[8..];
    } else if (std.mem.startsWith(u8, remaining, "http://")) {
        remaining = remaining[7..];
    } else {
        return null;
    }

    // Find path start
    const path_start = std.mem.indexOfScalar(u8, remaining, '/') orelse remaining.len;
    const host_port = remaining[0..path_start];
    const path = if (path_start < remaining.len) remaining[path_start..] else "/";

    // Parse host:port
    if (std.mem.lastIndexOfScalar(u8, host_port, ':')) |port_sep| {
        const port_str = host_port[port_sep + 1 ..];
        const port = std.fmt.parseInt(u16, port_str, 10) catch (if (is_https) @as(u16, 443) else @as(u16, 80));
        return .{
            .host = host_port[0..port_sep],
            .port = port,
            .path = path,
            .is_https = is_https,
        };
    }

    return .{
        .host = host_port,
        .port = if (is_https) 443 else 80,
        .path = path,
        .is_https = is_https,
    };
}

// Main fetch function
fn fetchImpl(ctx: ?*c.JSContext, _: c.JSValue, argc: c_int, argv: [*c]c.JSValue) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "fetch requires URL argument");
    }

    const url_cstr = c.JS_ToCString(ctx, argv[0]);
    if (url_cstr == null) {
        return c.JS_ThrowTypeError(ctx, "URL must be a string");
    }
    defer c.JS_FreeCString(ctx, url_cstr);

    const url_str = url_cstr[0..std.mem.len(url_cstr)];

    // Parse options
    var method: []const u8 = "GET";
    var body: ?[]const u8 = null;
    var content_type: ?[]const u8 = null;

    if (argc >= 2 and c.JS_IsObject(argv[1]) != 0) {
        // Get method
        const method_val = c.JS_GetPropertyStr(ctx, argv[1], "method");
        if (c.JS_IsString(method_val) != 0) {
            const m = c.JS_ToCString(ctx, method_val);
            if (m != null) {
                method = m[0..std.mem.len(m)];
                // Note: we don't free this as it's used later
            }
        }
        c.JS_FreeValue(ctx, method_val);

        // Get body
        const body_val = c.JS_GetPropertyStr(ctx, argv[1], "body");
        if (c.JS_IsString(body_val) != 0) {
            const b = c.JS_ToCString(ctx, body_val);
            if (b != null) {
                body = b[0..std.mem.len(b)];
            }
        }
        c.JS_FreeValue(ctx, body_val);

        // Get headers
        const headers_val = c.JS_GetPropertyStr(ctx, argv[1], "headers");
        if (c.JS_IsObject(headers_val) != 0) {
            const ct = c.JS_GetPropertyStr(ctx, headers_val, "Content-Type");
            if (c.JS_IsString(ct) != 0) {
                const ct_str = c.JS_ToCString(ctx, ct);
                if (ct_str != null) {
                    content_type = ct_str[0..std.mem.len(ct_str)];
                }
            }
            c.JS_FreeValue(ctx, ct);
        }
        c.JS_FreeValue(ctx, headers_val);
    }

    // Parse URL
    const parsed = parseUrl(url_str) orelse {
        return c.JS_ThrowTypeError(ctx, "Invalid URL");
    };

    // HTTPS not supported yet
    if (parsed.is_https) {
        return c.JS_ThrowTypeError(ctx, "HTTPS not supported yet, use HTTP");
    }

    // Make HTTP request
    const response_body = makeHttpRequest(parsed.host, parsed.port, parsed.path, method, body, content_type) catch |err| {
        var buf: [256]u8 = undefined;
        const msg = std.fmt.bufPrint(&buf, "Network error: {}", .{err}) catch "Network error";
        return c.JS_ThrowTypeError(ctx, @ptrCast(msg.ptr));
    };
    defer if (response_body.body) |b| allocator.free(b);

    // Create response
    var headers: std.StringHashMapUnmanaged([]const u8) = .{};

    // Parse response headers
    if (response_body.headers) |h| {
        var lines = std.mem.splitScalar(u8, h, '\n');
        while (lines.next()) |line| {
            const trimmed = std.mem.trim(u8, line, "\r\n ");
            if (std.mem.indexOf(u8, trimmed, ": ")) |sep| {
                const key = allocator.dupe(u8, trimmed[0..sep]) catch continue;
                const val = allocator.dupe(u8, trimmed[sep + 2 ..]) catch {
                    allocator.free(key);
                    continue;
                };
                headers.put(allocator, key, val) catch {
                    allocator.free(key);
                    allocator.free(val);
                };
            }
        }
        allocator.free(h);
    }

    return createResponse(ctx, response_body.status, response_body.body orelse "", url_str, headers);
}

const HttpResponse = struct {
    status: u16,
    headers: ?[]const u8,
    body: ?[]const u8,
};

fn makeHttpRequest(host: []const u8, port: u16, path: []const u8, method: []const u8, body: ?[]const u8, content_type: ?[]const u8) !HttpResponse {
    // Connect to server
    const address = std.net.Address.resolveIp(host, port) catch blk: {
        // Try DNS resolution
        const list = try std.net.getAddressList(allocator, host, port);
        defer list.deinit();
        if (list.addrs.len == 0) return error.HostNotFound;
        break :blk list.addrs[0];
    };

    const stream = try std.net.tcpConnectToAddress(address);
    defer stream.close();

    // Build request
    var request_buf: [4096]u8 = undefined;
    var request_len: usize = 0;

    // Request line
    const req_line = std.fmt.bufPrint(request_buf[request_len..], "{s} {s} HTTP/1.1\r\n", .{ method, path }) catch return error.BufferOverflow;
    request_len += req_line.len;

    // Host header
    const host_header = std.fmt.bufPrint(request_buf[request_len..], "Host: {s}\r\n", .{host}) catch return error.BufferOverflow;
    request_len += host_header.len;

    // User-Agent
    const ua = std.fmt.bufPrint(request_buf[request_len..], "User-Agent: Kiren/0.1.0\r\n", .{}) catch return error.BufferOverflow;
    request_len += ua.len;

    // Connection
    const conn = std.fmt.bufPrint(request_buf[request_len..], "Connection: close\r\n", .{}) catch return error.BufferOverflow;
    request_len += conn.len;

    // Content-Type and Content-Length for body
    if (body) |b| {
        const ct = content_type orelse "application/json";
        const ct_header = std.fmt.bufPrint(request_buf[request_len..], "Content-Type: {s}\r\n", .{ct}) catch return error.BufferOverflow;
        request_len += ct_header.len;

        const cl_header = std.fmt.bufPrint(request_buf[request_len..], "Content-Length: {d}\r\n", .{b.len}) catch return error.BufferOverflow;
        request_len += cl_header.len;
    }

    // End headers
    request_buf[request_len] = '\r';
    request_buf[request_len + 1] = '\n';
    request_len += 2;

    // Send request
    _ = try stream.write(request_buf[0..request_len]);

    // Send body if present
    if (body) |b| {
        _ = try stream.write(b);
    }

    // Read response
    var response_data: std.ArrayListUnmanaged(u8) = .{};
    defer response_data.deinit(allocator);

    var buf: [8192]u8 = undefined;
    while (true) {
        const n = stream.read(&buf) catch |err| {
            if (err == error.WouldBlock) continue;
            break;
        };
        if (n == 0) break;
        try response_data.appendSlice(allocator, buf[0..n]);
    }

    const response = response_data.toOwnedSlice(allocator) catch return error.OutOfMemory;

    // Parse response
    if (std.mem.indexOf(u8, response, "\r\n\r\n")) |header_end| {
        const header_part = response[0..header_end];
        const body_part = response[header_end + 4 ..];

        // Parse status line
        var status: u16 = 200;
        if (std.mem.indexOf(u8, header_part, " ")) |first_space| {
            const after_space = header_part[first_space + 1 ..];
            if (std.mem.indexOf(u8, after_space, " ")) |second_space| {
                status = std.fmt.parseInt(u16, after_space[0..second_space], 10) catch 200;
            }
        }

        return HttpResponse{
            .status = status,
            .headers = try allocator.dupe(u8, header_part),
            .body = try allocator.dupe(u8, body_part),
        };
    }

    allocator.free(response);
    return HttpResponse{ .status = 0, .headers = null, .body = null };
}

pub fn register(eng: *engine.Engine) void {
    const ctx = eng.context;

    // Create Response class
    response_class_id = c.JS_NewClassID(&response_class_id);

    const response_class_def = c.JSClassDef{
        .class_name = "Response",
        .finalizer = responseFinalizer,
        .gc_mark = null,
        .call = null,
        .exotic = null,
    };
    _ = c.JS_NewClass(c.JS_GetRuntime(ctx), response_class_id, &response_class_def);

    // Create Response prototype
    response_proto = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, response_proto, "text", c.JS_NewCFunction(ctx, responseText, "text", 0));
    _ = c.JS_SetPropertyStr(ctx, response_proto, "json", c.JS_NewCFunction(ctx, responseJson, "json", 0));
    _ = c.JS_SetPropertyStr(ctx, response_proto, "arrayBuffer", c.JS_NewCFunction(ctx, responseArrayBuffer, "arrayBuffer", 0));

    // Register fetch globally
    const global = c.JS_GetGlobalObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, global, "fetch", c.JS_NewCFunction(ctx, fetchImpl, "fetch", 2));
    c.JS_FreeValue(ctx, global);
}
