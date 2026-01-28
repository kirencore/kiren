const std = @import("std");
const engine = @import("../engine.zig");
const config_mod = @import("config.zig");
const router_mod = @import("router.zig");
const transform = @import("transform.zig");
const http = @import("../api/http.zig");
const c = engine.c;

// API imports for registration
const console = @import("../api/console.zig");
const process = @import("../api/process.zig");
const path_mod = @import("../api/path.zig");
const fs = @import("../api/fs.zig");
const buffer = @import("../api/buffer.zig");
const url = @import("../api/url.zig");
const encoding = @import("../api/encoding.zig");
const crypto = @import("../api/crypto.zig");
const fetch = @import("../api/fetch.zig");
const module = @import("../api/module.zig");
const websocket = @import("../api/websocket.zig");
const sqlite = @import("../api/sqlite.zig");
const event_loop = @import("../event_loop.zig");

const Worker = struct {
    name: []const u8,
    fetch_callback: c.JSValue,
    module_exports: c.JSValue,
};

pub const EdgeRuntime = struct {
    allocator: std.mem.Allocator,
    eng: engine.Engine,
    loop: event_loop.EventLoop,
    config: ?config_mod.EdgeConfig,
    router: router_mod.Router,
    workers: std.ArrayListUnmanaged(Worker),

    pub fn init(allocator: std.mem.Allocator) !EdgeRuntime {
        var eng = try engine.Engine.init();

        var loop = try event_loop.EventLoop.init(allocator, eng.context);
        event_loop.setGlobalEventLoop(&loop);

        // Register all APIs
        console.register(&eng);
        process.register(&eng);
        path_mod.register(&eng);
        fs.register(&eng);
        buffer.register(&eng);
        url.register(&eng);
        encoding.register(&eng);
        crypto.register(&eng);
        fetch.register(&eng);
        module.register(&eng);
        http.register(&eng);
        websocket.register(&eng);
        sqlite.register(&eng);
        event_loop.register(&eng);

        return EdgeRuntime{
            .allocator = allocator,
            .eng = eng,
            .loop = loop,
            .config = null,
            .router = router_mod.Router.init(allocator),
            .workers = .{},
        };
    }

    pub fn deinit(self: *EdgeRuntime) void {
        // Free worker callbacks
        for (self.workers.items) |worker| {
            c.JS_FreeValue(self.eng.context, worker.fetch_callback);
            c.JS_FreeValue(self.eng.context, worker.module_exports);
        }
        self.workers.deinit(self.allocator);

        self.router.deinit();

        if (self.config) |*cfg| {
            cfg.deinit();
        }

        self.loop.deinit();
        self.eng.deinit();
    }

    pub fn loadConfig(self: *EdgeRuntime, config_path: []const u8) !void {
        self.config = try config_mod.EdgeConfig.parse(self.allocator, config_path);

        const cfg = self.config.?;

        // Load each worker
        for (cfg.workers, 0..) |worker_cfg, idx| {
            try self.loadWorker(worker_cfg, idx);
        }
    }

    fn loadWorker(self: *EdgeRuntime, worker_cfg: config_mod.WorkerConfig, index: usize) !void {
        if (worker_cfg.path.len == 0) {
            std.debug.print("Worker '{s}' has no path, skipping\n", .{worker_cfg.name});
            return;
        }

        // Read file
        const file = std.fs.cwd().openFile(worker_cfg.path, .{}) catch |err| {
            std.debug.print("Failed to open worker file '{s}': {}\n", .{ worker_cfg.path, err });
            return error.WorkerLoadFailed;
        };
        defer file.close();

        const stat = try file.stat();
        const source = try self.allocator.alloc(u8, stat.size);
        defer self.allocator.free(source);
        _ = try file.readAll(source);

        // Check if ES module and transform
        var code: []const u8 = source;
        var transformed: ?[]u8 = null;

        if (transform.isEsModule(source)) {
            transformed = try transform.transformEsModule(self.allocator, source);
            code = transformed.?;
        }
        defer if (transformed) |t| self.allocator.free(t);

        // Wrap in module pattern
        const wrapper_prefix = "(function(exports, require, module, __filename, __dirname) {\n";
        const wrapper_suffix = "\nreturn module.exports;\n})({}, require, {exports:{}}, '', '')";

        const full_code = try std.fmt.allocPrint(self.allocator, "{s}{s}{s}", .{ wrapper_prefix, code, wrapper_suffix });
        defer self.allocator.free(full_code);

        // Create null-terminated path
        const path_z = try self.allocator.allocSentinel(u8, worker_cfg.path.len, 0);
        defer self.allocator.free(path_z);
        @memcpy(path_z, worker_cfg.path);

        // Evaluate
        const result = c.JS_Eval(
            self.eng.context,
            full_code.ptr,
            full_code.len,
            path_z.ptr,
            c.JS_EVAL_TYPE_GLOBAL,
        );

        if (c.JS_IsException(result) != 0) {
            self.eng.dumpError();
            return error.WorkerLoadFailed;
        }

        // Get fetch callback
        var fetch_cb: c.JSValue = engine.makeUndefined();

        if (c.JS_IsObject(result) != 0) {
            // Try direct fetch property
            fetch_cb = c.JS_GetPropertyStr(self.eng.context, result, "fetch");

            if (c.JS_IsUndefined(fetch_cb) != 0) {
                c.JS_FreeValue(self.eng.context, fetch_cb);

                // Try default.fetch
                const default_obj = c.JS_GetPropertyStr(self.eng.context, result, "default");
                if (c.JS_IsObject(default_obj) != 0) {
                    fetch_cb = c.JS_GetPropertyStr(self.eng.context, default_obj, "fetch");
                }
                c.JS_FreeValue(self.eng.context, default_obj);
            }
        }

        if (c.JS_IsFunction(self.eng.context, fetch_cb) == 0) {
            std.debug.print("Worker '{s}' has no fetch handler\n", .{worker_cfg.name});
            c.JS_FreeValue(self.eng.context, result);
            c.JS_FreeValue(self.eng.context, fetch_cb);
            return error.NoFetchHandler;
        }

        // Store worker
        try self.workers.append(self.allocator, .{
            .name = worker_cfg.name,
            .fetch_callback = fetch_cb,
            .module_exports = result,
        });

        // Add routes
        for (worker_cfg.routes) |route| {
            try self.router.addRoute(route, index);
        }

        std.debug.print("Loaded worker: {s} ({d} routes)\n", .{ worker_cfg.name, worker_cfg.routes.len });
    }

    pub fn start(self: *EdgeRuntime) !void {
        const cfg = self.config orelse return error.NoConfig;

        std.debug.print("\n", .{});
        std.debug.print("  Kiren Edge Runtime\n", .{});
        std.debug.print("  -------------------\n", .{});
        std.debug.print("  Port: {d}\n", .{cfg.port});
        std.debug.print("  Workers: {d}\n", .{self.workers.items.len});
        std.debug.print("\n", .{});

        self.router.printRoutes();
        std.debug.print("\n", .{});

        try self.runServer(cfg.port);
    }

    fn runServer(self: *EdgeRuntime, port: u16) !void {
        const address = std.net.Address.initIp4(.{ 0, 0, 0, 0 }, port);
        var server = try address.listen(.{ .reuse_address = true });
        defer server.deinit();

        std.debug.print("Listening on http://localhost:{d}\n\n", .{port});

        while (true) {
            const connection = server.accept() catch |err| {
                std.debug.print("Accept error: {}\n", .{err});
                continue;
            };

            self.handleConnection(connection) catch |err| {
                std.debug.print("Connection error: {}\n", .{err});
            };
        }
    }

    fn handleConnection(self: *EdgeRuntime, connection: std.net.Server.Connection) !void {
        defer connection.stream.close();

        // Read request
        var buf: [16384]u8 = undefined;
        const bytes_read = connection.stream.read(&buf) catch |err| {
            std.debug.print("Read error: {}\n", .{err});
            return;
        };

        if (bytes_read == 0) return;

        const request_data = buf[0..bytes_read];

        // Parse request
        var request = http.parseHttpRequest(self.allocator, request_data) catch |err| {
            std.debug.print("Parse error: {}\n", .{err});
            const response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 11\r\n\r\nBad Request";
            _ = connection.stream.write(response) catch {};
            return;
        };
        defer request.deinit(self.allocator);

        // Log request
        std.debug.print("{s} {s}", .{ request.method, request.path });

        // Route match
        const worker_idx = self.router.match(request.path);

        if (worker_idx) |idx| {
            if (idx < self.workers.items.len) {
                const worker = self.workers.items[idx];
                const response = self.callWorkerFetch(worker, &request) catch |err| {
                    std.debug.print(" -> Error: {}\n", .{err});
                    const error_response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 21\r\n\r\nInternal Server Error";
                    _ = connection.stream.write(error_response) catch {};
                    return;
                };
                defer self.allocator.free(response);

                std.debug.print(" -> {s}\n", .{worker.name});
                _ = connection.stream.write(response) catch {};
                return;
            }
        }

        // 404
        std.debug.print(" -> 404\n", .{});
        const not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        _ = connection.stream.write(not_found) catch {};
    }

    fn callWorkerFetch(self: *EdgeRuntime, worker: Worker, request: *http.HttpRequest) ![]u8 {
        const ctx = self.eng.context;

        // Create Request object
        const js_request = c.JS_NewObject(ctx);

        // Set method
        const method_str = c.JS_NewStringLen(ctx, request.method.ptr, request.method.len);
        _ = c.JS_SetPropertyStr(ctx, js_request, "method", method_str);

        // Set url
        const url_str = c.JS_NewStringLen(ctx, request.raw_url.ptr, request.raw_url.len);
        _ = c.JS_SetPropertyStr(ctx, js_request, "url", url_str);

        // Set headers
        const js_headers = c.JS_NewObject(ctx);
        var iter = request.headers.iterator();
        while (iter.next()) |entry| {
            const key_z = try self.allocator.allocSentinel(u8, entry.key_ptr.*.len, 0);
            defer self.allocator.free(key_z);
            @memcpy(key_z, entry.key_ptr.*);

            const val = c.JS_NewStringLen(ctx, entry.value_ptr.*.ptr, entry.value_ptr.*.len);
            _ = c.JS_SetPropertyStr(ctx, js_headers, key_z.ptr, val);
        }
        _ = c.JS_SetPropertyStr(ctx, js_request, "headers", js_headers);

        // Set body
        if (request.body.len > 0) {
            const body_str = c.JS_NewStringLen(ctx, request.body.ptr, request.body.len);
            _ = c.JS_SetPropertyStr(ctx, js_request, "body", body_str);
        }

        // Call fetch handler
        var args = [_]c.JSValue{js_request};
        const result = c.JS_Call(
            ctx,
            worker.fetch_callback,
            worker.module_exports,
            1,
            &args,
        );

        // Cleanup request object
        c.JS_FreeValue(ctx, js_request);

        if (c.JS_IsException(result) != 0) {
            self.eng.dumpError();
            return error.FetchError;
        }
        defer c.JS_FreeValue(ctx, result);

        // Parse response
        return try parseJsResponse(self.allocator, ctx, result);
    }
};

// Parse JavaScript Response object to HTTP response string
fn parseJsResponse(allocator: std.mem.Allocator, ctx: *c.JSContext, response: c.JSValue) ![]u8 {
    var status: i32 = 200;
    var body: []const u8 = "";
    var content_type: []const u8 = "text/plain";

    if (c.JS_IsObject(response) != 0) {
        // Get status
        const status_val = c.JS_GetPropertyStr(ctx, response, "status");
        if (c.JS_IsNumber(status_val) != 0) {
            _ = c.JS_ToInt32(ctx, &status, status_val);
        }
        c.JS_FreeValue(ctx, status_val);

        // Get body (_body for Response class, body for plain object)
        var body_val = c.JS_GetPropertyStr(ctx, response, "_body");
        if (c.JS_IsUndefined(body_val) != 0) {
            c.JS_FreeValue(ctx, body_val);
            body_val = c.JS_GetPropertyStr(ctx, response, "body");
        }
        if (c.JS_IsString(body_val) != 0) {
            const str = c.JS_ToCString(ctx, body_val);
            if (str != null) {
                body = std.mem.span(str);
            }
        }
        c.JS_FreeValue(ctx, body_val);

        // Get headers
        var headers_val = c.JS_GetPropertyStr(ctx, response, "_headers");
        if (c.JS_IsUndefined(headers_val) != 0) {
            c.JS_FreeValue(ctx, headers_val);
            headers_val = c.JS_GetPropertyStr(ctx, response, "headers");
        }
        if (c.JS_IsObject(headers_val) != 0) {
            // Get content-type
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
        }
        c.JS_FreeValue(ctx, headers_val);
    } else if (c.JS_IsString(response) != 0) {
        const str = c.JS_ToCString(ctx, response);
        if (str != null) {
            body = std.mem.span(str);
        }
    }

    const status_text = getStatusText(@intCast(status));

    return std.fmt.allocPrint(
        allocator,
        "HTTP/1.1 {d} {s}\r\nContent-Type: {s}\r\nContent-Length: {d}\r\nConnection: keep-alive\r\n\r\n{s}",
        .{ status, status_text, content_type, body.len, body },
    );
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
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        else => "Unknown",
    };
}
