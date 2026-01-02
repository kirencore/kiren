const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const net = std.net;
const posix = std.posix;
const websocket = @import("websocket.zig");

const Allocator = std.mem.Allocator;

// HTTP Request structure
pub const HttpRequest = struct {
    method: []const u8,
    path: []const u8,
    headers: std.StringHashMap([]const u8),
    body: []const u8,
    raw_url: []const u8,

    pub fn deinit(self: *HttpRequest, allocator: Allocator) void {
        allocator.free(self.method);
        allocator.free(self.path);
        allocator.free(self.raw_url);
        if (self.body.len > 0) {
            allocator.free(self.body);
        }
        var iter = self.headers.iterator();
        while (iter.next()) |entry| {
            allocator.free(entry.key_ptr.*);
            allocator.free(entry.value_ptr.*);
        }
        self.headers.deinit();
    }
};

// Simple HTTP parser
pub fn parseHttpRequest(allocator: Allocator, data: []const u8) !HttpRequest {
    var headers = std.StringHashMap([]const u8).init(allocator);
    errdefer headers.deinit();

    var lines = std.mem.splitSequence(u8, data, "\r\n");

    // Parse request line
    const request_line = lines.next() orelse return error.InvalidRequest;
    var parts = std.mem.splitScalar(u8, request_line, ' ');

    const method = parts.next() orelse return error.InvalidRequest;
    const path = parts.next() orelse return error.InvalidRequest;

    // Parse headers
    while (lines.next()) |line| {
        if (line.len == 0) break; // Empty line = end of headers

        if (std.mem.indexOf(u8, line, ": ")) |sep_idx| {
            const key = try allocator.dupe(u8, line[0..sep_idx]);
            const value = try allocator.dupe(u8, line[sep_idx + 2 ..]);
            try headers.put(key, value);
        }
    }

    // Get body (rest after empty line)
    const header_end = std.mem.indexOf(u8, data, "\r\n\r\n");
    const body = if (header_end) |idx|
        if (idx + 4 < data.len) try allocator.dupe(u8, data[idx + 4 ..]) else ""
    else
        "";

    return HttpRequest{
        .method = try allocator.dupe(u8, method),
        .path = try allocator.dupe(u8, path),
        .headers = headers,
        .body = body,
        .raw_url = try allocator.dupe(u8, path),
    };
}

// Format HTTP response
pub fn formatHttpResponse(allocator: Allocator, status: u16, status_text: []const u8, headers: []const u8, body: []const u8) ![]u8 {
    return std.fmt.allocPrint(allocator, "HTTP/1.1 {d} {s}\r\nContent-Length: {d}\r\n{s}\r\n\r\n{s}", .{
        status,
        status_text,
        body.len,
        headers,
        body,
    });
}

// Global server state
var global_context: ?*c.JSContext = null;
var global_fetch_callback: c.JSValue = undefined;
var global_websocket_callback: ?c.JSValue = null;
var server_running: bool = false;

// Kiren.serve() implementation
fn kirenServe(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "serve() requires an options object");
    }

    const options = argv[0];
    if (c.JS_IsObject(options) == 0) {
        return c.JS_ThrowTypeError(ctx, "serve() requires an options object");
    }

    // Get port
    const port_val = c.JS_GetPropertyStr(ctx, options, "port");
    defer c.JS_FreeValue(ctx, port_val);

    var port: i32 = 3000;
    if (c.JS_IsNumber(port_val) != 0) {
        _ = c.JS_ToInt32(ctx, &port, port_val);
    }

    // Get fetch callback
    const fetch_val = c.JS_GetPropertyStr(ctx, options, "fetch");
    if (c.JS_IsFunction(ctx, fetch_val) == 0) {
        c.JS_FreeValue(ctx, fetch_val);
        return c.JS_ThrowTypeError(ctx, "serve() requires a fetch function");
    }

    // Get optional websocket callback
    const ws_val = c.JS_GetPropertyStr(ctx, options, "websocket");
    const has_websocket = c.JS_IsObject(ws_val) != 0;

    // Store globals
    global_context = context;
    global_fetch_callback = c.JS_DupValue(ctx, fetch_val);
    c.JS_FreeValue(ctx, fetch_val);

    if (has_websocket) {
        global_websocket_callback = c.JS_DupValue(ctx, ws_val);
        // Initialize WebSocket state
        websocket.initGlobalState(context, ws_val);
    }
    c.JS_FreeValue(ctx, ws_val);

    // Start server in a separate function
    startServer(@intCast(port)) catch |err| {
        std.debug.print("Server error: {}\n", .{err});
        return c.JS_ThrowInternalError(ctx, "Failed to start server");
    };

    return engine.makeUndefined();
}

fn startServer(port: u16) !void {
    const allocator = std.heap.page_allocator;

    const address = net.Address.initIp4(.{ 0, 0, 0, 0 }, port);
    var server = try address.listen(.{
        .reuse_address = true,
    });
    defer server.deinit();

    std.debug.print("Kiren server listening on http://localhost:{d}\n", .{port});
    server_running = true;

    while (server_running) {
        const connection = server.accept() catch |err| {
            std.debug.print("Accept error: {}\n", .{err});
            continue;
        };

        handleConnection(allocator, connection) catch |err| {
            std.debug.print("Connection error: {}\n", .{err});
        };
    }
}

// Thread wrapper for WebSocket handling
fn handleWebSocketThread(allocator: Allocator, stream: net.Stream, request_data_ptr: [*]const u8, request_data_len: usize) void {
    const request_data = request_data_ptr[0..request_data_len];
    defer allocator.free(request_data);

    websocket.handleWebSocket(allocator, stream, request_data) catch |err| {
        std.debug.print("WebSocket error: {}\n", .{err});
        stream.close();
    };
}

fn handleConnection(allocator: Allocator, connection: net.Server.Connection) !void {
    var buf: [4096]u8 = undefined;
    const bytes_read = try connection.stream.read(&buf);

    if (bytes_read == 0) {
        connection.stream.close();
        return;
    }

    const request_data = buf[0..bytes_read];

    // Parse request
    var request = try parseHttpRequest(allocator, request_data);

    // Check for WebSocket upgrade
    if (global_websocket_callback != null and websocket.isWebSocketUpgrade(request.headers)) {
        request.deinit(allocator);

        // Copy request data to heap for thread safety
        const heap_data = allocator.alloc(u8, request_data.len) catch {
            connection.stream.close();
            return;
        };
        @memcpy(heap_data, request_data);

        // Handle WebSocket in a separate thread so HTTP can continue
        const ws_thread = std.Thread.spawn(.{}, handleWebSocketThread, .{ allocator, connection.stream, heap_data.ptr, heap_data.len }) catch |err| {
            std.debug.print("Failed to spawn WebSocket thread: {}\n", .{err});
            allocator.free(heap_data);
            connection.stream.close();
            return;
        };
        ws_thread.detach();
        return;
    }

    defer request.deinit(allocator);
    defer connection.stream.close();

    // Call JavaScript fetch handler
    const response = callFetchHandler(allocator, &request) catch |err| {
        std.debug.print("Fetch handler error: {}\n", .{err});
        const error_response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 21\r\n\r\nInternal Server Error";
        _ = connection.stream.write(error_response) catch {};
        return;
    };
    defer allocator.free(response);

    _ = try connection.stream.write(response);
}

fn callFetchHandler(allocator: Allocator, request: *HttpRequest) ![]u8 {
    const ctx = global_context orelse return error.NoContext;

    // Create Request object for JavaScript
    const js_request = c.JS_NewObject(ctx);

    // Set request properties
    const method_str = c.JS_NewStringLen(ctx, request.method.ptr, request.method.len);
    _ = c.JS_SetPropertyStr(ctx, js_request, "method", method_str);

    const url_str = c.JS_NewStringLen(ctx, request.raw_url.ptr, request.raw_url.len);
    _ = c.JS_SetPropertyStr(ctx, js_request, "url", url_str);

    // Create full URL
    var full_url_buf: [256]u8 = undefined;
    const full_url = std.fmt.bufPrint(&full_url_buf, "http://localhost{s}", .{request.path}) catch request.path;
    const full_url_str = c.JS_NewStringLen(ctx, full_url.ptr, full_url.len);
    _ = c.JS_SetPropertyStr(ctx, js_request, "fullUrl", full_url_str);

    // Set body
    if (request.body.len > 0) {
        const body_str = c.JS_NewStringLen(ctx, request.body.ptr, request.body.len);
        _ = c.JS_SetPropertyStr(ctx, js_request, "body", body_str);
    }

    // Set headers as object
    const js_headers = c.JS_NewObject(ctx);
    var iter = request.headers.iterator();
    while (iter.next()) |entry| {
        const header_val = c.JS_NewStringLen(ctx, entry.value_ptr.*.ptr, entry.value_ptr.*.len);
        _ = c.JS_SetPropertyStr(ctx, js_headers, @ptrCast(entry.key_ptr.*.ptr), header_val);
    }
    _ = c.JS_SetPropertyStr(ctx, js_request, "headers", js_headers);

    // Call fetch function
    var args = [_]c.JSValue{js_request};
    const result = c.JS_Call(ctx, global_fetch_callback, engine.makeUndefined(), 1, &args);
    defer c.JS_FreeValue(ctx, result);
    defer c.JS_FreeValue(ctx, js_request);

    if (c.JS_IsException(result) != 0) {
        // Get and print exception
        const exception = c.JS_GetException(ctx);
        defer c.JS_FreeValue(ctx, exception);
        const str = c.JS_ToCString(ctx, exception);
        if (str != null) {
            std.debug.print("JS Error: {s}\n", .{str});
            c.JS_FreeCString(ctx, str);
        }
        return error.JSException;
    }

    // Parse Response object
    return parseJsResponse(allocator, ctx, result);
}

fn parseJsResponse(allocator: Allocator, ctx: *c.JSContext, response: c.JSValue) ![]u8 {
    var status: i32 = 200;
    var body: []const u8 = "";
    var content_type: []const u8 = "text/plain";
    var has_cors: bool = false;

    // Check if it's a Response object or plain object
    if (c.JS_IsObject(response) != 0) {
        // Get status
        const status_val = c.JS_GetPropertyStr(ctx, response, "status");
        if (c.JS_IsNumber(status_val) != 0) {
            _ = c.JS_ToInt32(ctx, &status, status_val);
        }
        c.JS_FreeValue(ctx, status_val);

        // Get body (check _body first for Response objects, then body)
        const body_val = c.JS_GetPropertyStr(ctx, response, "_body");
        if (c.JS_IsString(body_val) != 0) {
            const str = c.JS_ToCString(ctx, body_val);
            if (str != null) {
                body = std.mem.span(str);
            }
        }
        c.JS_FreeValue(ctx, body_val);

        // Get all headers from Response object
        const headers_val = c.JS_GetPropertyStr(ctx, response, "_headers");
        if (c.JS_IsObject(headers_val) != 0) {
            // Get content-type (try both cases)
            var ct_val = c.JS_GetPropertyStr(ctx, headers_val, "content-type");
            if (c.JS_IsUndefined(ct_val) != 0) {
                c.JS_FreeValue(ctx, ct_val);
                ct_val = c.JS_GetPropertyStr(ctx, headers_val, "Content-Type");
            }
            if (c.JS_IsString(ct_val) != 0) {
                const ct_str = c.JS_ToCString(ctx, ct_val);
                if (ct_str != null) {
                    content_type = std.mem.span(ct_str);
                }
            }
            c.JS_FreeValue(ctx, ct_val);

            // Get CORS headers
            const cors_origin = c.JS_GetPropertyStr(ctx, headers_val, "Access-Control-Allow-Origin");
            if (c.JS_IsString(cors_origin) != 0) {
                has_cors = true;
            }
            c.JS_FreeValue(ctx, cors_origin);
        }
        c.JS_FreeValue(ctx, headers_val);
    } else if (c.JS_IsString(response) != 0) {
        // Plain string response
        const str = c.JS_ToCString(ctx, response);
        if (str != null) {
            body = std.mem.span(str);
        }
    }

    const status_text = getStatusText(@intCast(status));
    var header_buf: [512]u8 = undefined;
    const headers = if (has_cors)
        std.fmt.bufPrint(&header_buf, "Content-Type: {s}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization", .{content_type}) catch "Content-Type: text/plain\r\nConnection: close"
    else
        std.fmt.bufPrint(&header_buf, "Content-Type: {s}\r\nConnection: close", .{content_type}) catch "Content-Type: text/plain\r\nConnection: close";

    return formatHttpResponse(allocator, @intCast(status), status_text, headers, body);
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
        else => "Unknown",
    };
}

// Response class implementation in JS
const response_js =
    \\class Response {
    \\  constructor(body, options = {}) {
    \\    this._body = body || "";
    \\    this.status = options.status || 200;
    \\    this.statusText = options.statusText || "OK";
    \\    this._headers = options.headers || {};
    \\  }
    \\
    \\  static json(data, options = {}) {
    \\    const body = JSON.stringify(data);
    \\    const headers = { "content-type": "application/json", ...(options.headers || {}) };
    \\    return new Response(body, { ...options, headers });
    \\  }
    \\
    \\  static text(text, options = {}) {
    \\    return new Response(text, options);
    \\  }
    \\}
    \\globalThis.Response = Response;
;

pub fn register(eng: *engine.Engine) void {
    // Register Response class
    const result = c.JS_Eval(
        eng.context,
        response_js.ptr,
        response_js.len,
        "<response>",
        c.JS_EVAL_TYPE_GLOBAL,
    );
    c.JS_FreeValue(eng.context, result);

    // Register Kiren namespace
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    const kiren = eng.newObject();
    eng.setProperty(kiren, "serve", eng.newCFunction(kirenServe, "serve", 1));
    eng.setProperty(kiren, "version", eng.newString("0.1.0"));

    eng.setProperty(global, "Kiren", kiren);
}
