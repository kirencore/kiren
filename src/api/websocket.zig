const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const net = std.net;
const Sha1 = std.crypto.hash.Sha1;
const base64 = std.base64;

const Allocator = std.mem.Allocator;

// WebSocket magic GUID for handshake
const WS_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// WebSocket opcodes
const Opcode = enum(u4) {
    continuation = 0x0,
    text = 0x1,
    binary = 0x2,
    close = 0x8,
    ping = 0x9,
    pong = 0xA,
};

// WebSocket frame structure
const Frame = struct {
    fin: bool,
    opcode: Opcode,
    masked: bool,
    payload: []const u8,
    consumed: usize, // Total bytes consumed from input buffer
};

// WebSocket connection
pub const WebSocketConnection = struct {
    stream: net.Stream,
    id: u64,
    rooms: std.ArrayListUnmanaged([]const u8),
    allocator: Allocator,

    pub fn init(allocator: Allocator, stream: net.Stream, id: u64) WebSocketConnection {
        return .{
            .stream = stream,
            .id = id,
            .rooms = .{},
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *WebSocketConnection) void {
        self.rooms.deinit(self.allocator);
        self.stream.close();
    }

    pub fn send(self: *WebSocketConnection, data: []const u8) !void {
        const result = encodeFrame(data, .text, true);
        _ = try self.stream.write(result.frame[0..result.len]);
    }

    pub fn sendBinary(self: *WebSocketConnection, data: []const u8) !void {
        const result = encodeFrame(data, .binary, true);
        _ = try self.stream.write(result.frame[0..result.len]);
    }

    pub fn close(self: *WebSocketConnection) void {
        const close_frame = [_]u8{ 0x88, 0x00 }; // FIN + close opcode, no payload
        _ = self.stream.write(&close_frame) catch {};
        self.stream.close();
    }

    pub fn joinRoom(self: *WebSocketConnection, room: []const u8) !void {
        try self.rooms.append(self.allocator, room);
    }

    pub fn leaveRoom(self: *WebSocketConnection, room: []const u8) void {
        for (self.rooms.items, 0..) |r, i| {
            if (std.mem.eql(u8, r, room)) {
                _ = self.rooms.orderedRemove(i);
                break;
            }
        }
    }
};

// Room management
pub const RoomManager = struct {
    rooms: std.StringHashMap(std.ArrayListUnmanaged(*WebSocketConnection)),
    allocator: Allocator,

    pub fn init(allocator: Allocator) RoomManager {
        return .{
            .rooms = std.StringHashMap(std.ArrayListUnmanaged(*WebSocketConnection)).init(allocator),
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *RoomManager) void {
        var iter = self.rooms.iterator();
        while (iter.next()) |entry| {
            entry.value_ptr.deinit(self.allocator);
        }
        self.rooms.deinit();
    }

    pub fn join(self: *RoomManager, room: []const u8, conn: *WebSocketConnection) !void {
        if (self.rooms.getPtr(room)) |list| {
            try list.append(self.allocator, conn);
        } else {
            var list: std.ArrayListUnmanaged(*WebSocketConnection) = .{};
            try list.append(self.allocator, conn);
            try self.rooms.put(room, list);
        }
    }

    pub fn leave(self: *RoomManager, room: []const u8, conn: *WebSocketConnection) void {
        if (self.rooms.getPtr(room)) |list| {
            for (list.items, 0..) |item, i| {
                if (item.id == conn.id) {
                    _ = list.orderedRemove(i);
                    break;
                }
            }
        }
    }

    pub fn broadcast(self: *RoomManager, room: []const u8, data: []const u8, exclude_id: ?u64) void {
        if (self.rooms.get(room)) |list| {
            for (list.items) |conn| {
                if (exclude_id) |id| {
                    if (conn.id == id) continue;
                }
                conn.send(data) catch {};
            }
        }
    }
};

// Global WebSocket state
var ws_context: ?*c.JSContext = null;
var ws_message_callback: c.JSValue = undefined;
var ws_open_callback: c.JSValue = undefined;
var ws_close_callback: c.JSValue = undefined;
var ws_connections: std.AutoHashMap(u64, *WebSocketConnection) = undefined;
var ws_room_manager: RoomManager = undefined;
var ws_next_id: u64 = 1;
var ws_initialized: bool = false;

// Initialize global state from websocket options object
pub fn initGlobalState(ctx: *c.JSContext, ws_options: c.JSValue) void {
    if (!ws_initialized) {
        ws_connections = std.AutoHashMap(u64, *WebSocketConnection).init(std.heap.page_allocator);
        ws_room_manager = RoomManager.init(std.heap.page_allocator);
        ws_initialized = true;
    }

    ws_context = ctx;

    // Get callbacks from websocket options
    const open_val = c.JS_GetPropertyStr(ctx, ws_options, "open");
    const message_val = c.JS_GetPropertyStr(ctx, ws_options, "message");
    const close_val = c.JS_GetPropertyStr(ctx, ws_options, "close");

    ws_open_callback = c.JS_DupValue(ctx, open_val);
    ws_message_callback = c.JS_DupValue(ctx, message_val);
    ws_close_callback = c.JS_DupValue(ctx, close_val);

    c.JS_FreeValue(ctx, open_val);
    c.JS_FreeValue(ctx, message_val);
    c.JS_FreeValue(ctx, close_val);
}

// WebSocket handshake
pub fn performHandshake(allocator: Allocator, stream: net.Stream, request_data: []const u8) !void {
    // Find Sec-WebSocket-Key header
    var key: ?[]const u8 = null;
    var lines = std.mem.splitSequence(u8, request_data, "\r\n");
    while (lines.next()) |line| {
        if (std.ascii.startsWithIgnoreCase(line, "Sec-WebSocket-Key:")) {
            const value_start = std.mem.indexOf(u8, line, ":").? + 1;
            key = std.mem.trim(u8, line[value_start..], " ");
            break;
        }
    }

    const ws_key = key orelse return error.NoWebSocketKey;

    // Generate accept key: SHA1(key + GUID) -> base64
    var hasher = Sha1.init(.{});
    hasher.update(ws_key);
    hasher.update(WS_GUID);
    const hash = hasher.finalResult();

    var accept_key: [28]u8 = undefined;
    _ = base64.standard.Encoder.encode(&accept_key, &hash);

    // Send handshake response
    const response = try std.fmt.allocPrint(allocator, "HTTP/1.1 101 Switching Protocols\r\n" ++
        "Upgrade: websocket\r\n" ++
        "Connection: Upgrade\r\n" ++
        "Sec-WebSocket-Accept: {s}\r\n" ++
        "\r\n", .{accept_key});
    defer allocator.free(response);

    _ = try stream.write(response);
}

// Static buffer for unmasked payload (persists beyond function call)
var decode_buffer: [8192]u8 = undefined;

// Decode WebSocket frame
pub fn decodeFrame(data: []const u8) !Frame {
    if (data.len < 2) return error.FrameTooShort;

    const fin = (data[0] & 0x80) != 0;
    const opcode: Opcode = @enumFromInt(@as(u4, @truncate(data[0] & 0x0F)));
    const masked = (data[1] & 0x80) != 0;
    var payload_len: u64 = data[1] & 0x7F;

    var offset: usize = 2;

    // Extended payload length
    if (payload_len == 126) {
        if (data.len < 4) return error.FrameTooShort;
        payload_len = @as(u64, data[2]) << 8 | @as(u64, data[3]);
        offset = 4;
    } else if (payload_len == 127) {
        if (data.len < 10) return error.FrameTooShort;
        payload_len = 0;
        for (0..8) |i| {
            payload_len = (payload_len << 8) | @as(u64, data[2 + i]);
        }
        offset = 10;
    }

    // Get masking key if present
    var mask: [4]u8 = undefined;
    if (masked) {
        if (data.len < offset + 4) return error.FrameTooShort;
        @memcpy(&mask, data[offset .. offset + 4]);
        offset += 4;
    }

    // Get payload
    if (data.len < offset + payload_len) return error.FrameTooShort;
    const raw_payload = data[offset .. offset + payload_len];

    const total_consumed = offset + payload_len;

    // Unmask if needed - copy to static buffer
    if (masked) {
        if (payload_len > decode_buffer.len) return error.PayloadTooLarge;
        for (raw_payload, 0..) |byte, i| {
            decode_buffer[i] = byte ^ mask[i % 4];
        }
        return Frame{
            .fin = fin,
            .opcode = opcode,
            .masked = masked,
            .payload = decode_buffer[0..payload_len],
            .consumed = total_consumed,
        };
    }

    return Frame{
        .fin = fin,
        .opcode = opcode,
        .masked = masked,
        .payload = raw_payload,
        .consumed = total_consumed,
    };
}

// Encode WebSocket frame - returns frame buffer and actual length
pub fn encodeFrame(data: []const u8, opcode: Opcode, fin: bool) struct { frame: [4096]u8, len: usize } {
    var frame: [4096]u8 = undefined;
    var offset: usize = 0;

    // First byte: FIN + opcode
    frame[0] = (@as(u8, if (fin) 0x80 else 0x00)) | @intFromEnum(opcode);
    offset = 1;

    // Payload length (server frames are NOT masked, so mask bit = 0)
    if (data.len < 126) {
        frame[1] = @intCast(data.len); // No mask bit set (0x80)
        offset = 2;
    } else if (data.len < 65536) {
        frame[1] = 126; // No mask bit set
        frame[2] = @intCast((data.len >> 8) & 0xFF);
        frame[3] = @intCast(data.len & 0xFF);
        offset = 4;
    } else {
        frame[1] = 127; // No mask bit set
        for (0..8) |i| {
            frame[2 + i] = @intCast((data.len >> @intCast(56 - i * 8)) & 0xFF);
        }
        offset = 10;
    }

    // Copy payload
    @memcpy(frame[offset .. offset + data.len], data);

    const total_len = offset + data.len;
    return .{ .frame = frame, .len = total_len };
}

// Check if request is WebSocket upgrade
pub fn isWebSocketUpgrade(headers: std.StringHashMap([]const u8)) bool {
    if (headers.get("Upgrade")) |upgrade| {
        return std.ascii.eqlIgnoreCase(upgrade, "websocket");
    }
    return false;
}

// Handle WebSocket connection
pub fn handleWebSocket(allocator: Allocator, stream: net.Stream, request_data: []const u8) !void {
    // Perform handshake
    try performHandshake(allocator, stream, request_data);

    // Create connection
    const conn_id = ws_next_id;
    ws_next_id += 1;

    var conn = try allocator.create(WebSocketConnection);
    conn.* = WebSocketConnection.init(allocator, stream, conn_id);
    try ws_connections.put(conn_id, conn);

    // Call open callback
    if (ws_context) |ctx| {
        const js_conn = createJsConnection(ctx, conn);
        var args = [_]c.JSValue{js_conn};
        const result = c.JS_Call(ctx, ws_open_callback, engine.makeUndefined(), 1, &args);
        c.JS_FreeValue(ctx, result);
    }

    // Read loop with support for multiple frames per read
    var buf: [8192]u8 = undefined;
    var leftover: usize = 0; // Bytes left from previous read

    while (true) {
        // Read new data after any leftover bytes
        const bytes_read = stream.read(buf[leftover..]) catch break;
        if (bytes_read == 0) break;

        const total_bytes = leftover + bytes_read;
        var processed: usize = 0;

        // Process all complete frames in buffer
        std.debug.print("WS read: {} bytes, leftover: {}, total: {}\n", .{ bytes_read, leftover, total_bytes });

        while (processed < total_bytes) {
            const remaining = buf[processed..total_bytes];
            if (remaining.len < 2) break; // Need at least 2 bytes for frame header

            const frame = decodeFrame(remaining) catch |err| {
                std.debug.print("WS decode error at offset {}: {}\n", .{ processed, err });
                if (err == error.FrameTooShort) break; // Wait for more data
                // Other errors - skip this connection
                leftover = 0;
                break;
            };

            std.debug.print("WS frame: opcode={}, payload_len={}, consumed={}\n", .{ @intFromEnum(frame.opcode), frame.payload.len, frame.consumed });

            switch (frame.opcode) {
                .text, .binary => {
                    // Call message callback
                    if (ws_context) |ctx| {
                        const js_conn = createJsConnection(ctx, conn);
                        const js_data = c.JS_NewStringLen(ctx, frame.payload.ptr, frame.payload.len);
                        var args = [_]c.JSValue{ js_conn, js_data };
                        const result = c.JS_Call(ctx, ws_message_callback, engine.makeUndefined(), 2, &args);
                        c.JS_FreeValue(ctx, result);
                    }
                },
                .close => {
                    leftover = 0;
                    break;
                },
                .ping => {
                    // Send pong
                    const pong_result = encodeFrame(frame.payload, .pong, true);
                    _ = stream.write(pong_result.frame[0..pong_result.len]) catch break;
                },
                .pong => {},
                else => {},
            }

            processed += frame.consumed;

            if (frame.opcode == .close) break;
        }

        // Move unprocessed bytes to beginning of buffer
        if (processed < total_bytes) {
            const remaining = total_bytes - processed;
            std.mem.copyForwards(u8, buf[0..remaining], buf[processed..total_bytes]);
            leftover = remaining;
        } else {
            leftover = 0;
        }
    }

    // Call close callback
    if (ws_context) |ctx| {
        const js_conn = createJsConnection(ctx, conn);
        var args = [_]c.JSValue{js_conn};
        const result = c.JS_Call(ctx, ws_close_callback, engine.makeUndefined(), 1, &args);
        c.JS_FreeValue(ctx, result);
    }

    // Cleanup
    _ = ws_connections.remove(conn_id);
    conn.deinit();
    allocator.destroy(conn);
}

// Create JS connection object
fn createJsConnection(ctx: *c.JSContext, conn: *WebSocketConnection) c.JSValue {
    const obj = c.JS_NewObject(ctx);

    // Add id
    _ = c.JS_SetPropertyStr(ctx, obj, "id", c.JS_NewInt64(ctx, @intCast(conn.id)));

    // Store pointer for native calls
    _ = c.JS_SetPropertyStr(ctx, obj, "_ptr", c.JS_NewInt64(ctx, @intCast(@intFromPtr(conn))));

    return obj;
}

// Kiren.ws() implementation
fn kirenWs(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "ws() requires an options object");
    }

    const options = argv[0];
    if (c.JS_IsObject(options) == 0) {
        return c.JS_ThrowTypeError(ctx, "ws() requires an options object");
    }

    // Initialize if needed
    if (!ws_initialized) {
        ws_connections = std.AutoHashMap(u64, *WebSocketConnection).init(std.heap.page_allocator);
        ws_room_manager = RoomManager.init(std.heap.page_allocator);
        ws_initialized = true;
    }

    // Get port
    const port_val = c.JS_GetPropertyStr(ctx, options, "port");
    defer c.JS_FreeValue(ctx, port_val);

    var port: i32 = 8080;
    if (c.JS_IsNumber(port_val) != 0) {
        _ = c.JS_ToInt32(ctx, &port, port_val);
    }

    // Get callbacks
    const open_val = c.JS_GetPropertyStr(ctx, options, "open");
    const message_val = c.JS_GetPropertyStr(ctx, options, "message");
    const close_val = c.JS_GetPropertyStr(ctx, options, "close");

    ws_context = context;
    ws_open_callback = c.JS_DupValue(ctx, open_val);
    ws_message_callback = c.JS_DupValue(ctx, message_val);
    ws_close_callback = c.JS_DupValue(ctx, close_val);

    c.JS_FreeValue(ctx, open_val);
    c.JS_FreeValue(ctx, message_val);
    c.JS_FreeValue(ctx, close_val);

    // Start WebSocket server
    startWsServer(@intCast(port)) catch |err| {
        std.debug.print("WebSocket server error: {}\n", .{err});
        return c.JS_ThrowInternalError(ctx, "Failed to start WebSocket server");
    };

    return engine.makeUndefined();
}

fn startWsServer(port: u16) !void {
    const allocator = std.heap.page_allocator;

    const address = net.Address.initIp4(.{ 0, 0, 0, 0 }, port);
    var server = try address.listen(.{
        .reuse_address = true,
    });
    defer server.deinit();

    std.debug.print("Kiren WebSocket server listening on ws://localhost:{d}\n", .{port});

    while (true) {
        const connection = server.accept() catch |err| {
            std.debug.print("Accept error: {}\n", .{err});
            continue;
        };

        // Read HTTP upgrade request
        var buf: [4096]u8 = undefined;
        const bytes_read = connection.stream.read(&buf) catch continue;
        if (bytes_read == 0) continue;

        handleWebSocket(allocator, connection.stream, buf[0..bytes_read]) catch |err| {
            std.debug.print("WebSocket error: {}\n", .{err});
        };
    }
}

// ws.send() - send to specific connection
fn wsSend(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 2) return engine.makeUndefined();

    const conn_obj = argv[0];
    const data_val = argv[1];

    // Get connection pointer
    const ptr_val = c.JS_GetPropertyStr(ctx, conn_obj, "_ptr");
    defer c.JS_FreeValue(ctx, ptr_val);

    var ptr: i64 = 0;
    _ = c.JS_ToInt64(ctx, &ptr, ptr_val);

    const conn: *WebSocketConnection = @ptrFromInt(@as(usize, @intCast(ptr)));

    // Get data string
    const str = c.JS_ToCString(ctx, data_val);
    if (str != null) {
        conn.send(std.mem.span(str)) catch {};
        c.JS_FreeCString(ctx, str);
    }

    return engine.makeUndefined();
}

// ws.broadcast() - broadcast to all connections
fn wsBroadcast(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) return engine.makeUndefined();

    const data_val = argv[0];
    const str = c.JS_ToCString(ctx, data_val);
    if (str == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, str);

    const data = std.mem.span(str);

    // Broadcast to all connections
    var iter = ws_connections.iterator();
    while (iter.next()) |entry| {
        entry.value_ptr.*.send(data) catch {};
    }

    return engine.makeUndefined();
}

// ws.joinRoom() - join a room
fn wsJoinRoom(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 2) return engine.makeUndefined();

    const conn_obj = argv[0];
    const room_val = argv[1];

    // Get connection pointer
    const ptr_val = c.JS_GetPropertyStr(ctx, conn_obj, "_ptr");
    defer c.JS_FreeValue(ctx, ptr_val);

    var ptr: i64 = 0;
    _ = c.JS_ToInt64(ctx, &ptr, ptr_val);

    const conn: *WebSocketConnection = @ptrFromInt(@as(usize, @intCast(ptr)));

    // Get room name
    const room_str = c.JS_ToCString(ctx, room_val);
    if (room_str == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, room_str);

    // Join the room
    ws_room_manager.join(std.mem.span(room_str), conn) catch {};

    return engine.makeUndefined();
}

// ws.broadcastRoom() - broadcast to room
fn wsBroadcastRoom(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 2) return engine.makeUndefined();

    const room_val = argv[0];
    const data_val = argv[1];

    const room_str = c.JS_ToCString(ctx, room_val);
    if (room_str == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, room_str);

    const data_str = c.JS_ToCString(ctx, data_val);
    if (data_str == null) return engine.makeUndefined();
    defer c.JS_FreeCString(ctx, data_str);

    ws_room_manager.broadcast(std.mem.span(room_str), std.mem.span(data_str), null);

    return engine.makeUndefined();
}

// Register WebSocket API
pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    // Get or create Kiren namespace
    var kiren = c.JS_GetPropertyStr(eng.context, global, "Kiren");
    if (c.JS_IsUndefined(kiren) != 0) {
        kiren = eng.newObject();
    }

    // Add WebSocket functions
    eng.setProperty(kiren, "ws", eng.newCFunction(kirenWs, "ws", 1));
    eng.setProperty(kiren, "wsSend", eng.newCFunction(wsSend, "wsSend", 2));
    eng.setProperty(kiren, "wsBroadcast", eng.newCFunction(wsBroadcast, "wsBroadcast", 1));
    eng.setProperty(kiren, "wsJoinRoom", eng.newCFunction(wsJoinRoom, "wsJoinRoom", 2));
    eng.setProperty(kiren, "wsBroadcastRoom", eng.newCFunction(wsBroadcastRoom, "wsBroadcastRoom", 2));

    eng.setProperty(global, "Kiren", kiren);
}
