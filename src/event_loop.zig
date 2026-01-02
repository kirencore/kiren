const std = @import("std");
const engine = @import("engine.zig");
const c = engine.c;

const Allocator = std.mem.Allocator;

pub const TimerType = enum {
    timeout,
    interval,
};

pub const Timer = struct {
    id: u32,
    callback: c.JSValue,
    interval_ms: u64,
    next_run: i64,
    timer_type: TimerType,
    cleared: bool,
};

pub const EventLoop = struct {
    allocator: Allocator,
    context: *c.JSContext,
    timers: std.ArrayListUnmanaged(Timer),
    next_timer_id: u32,
    running: bool,

    pub fn init(allocator: Allocator, context: *c.JSContext) EventLoop {
        return EventLoop{
            .allocator = allocator,
            .context = context,
            .timers = .{},
            .next_timer_id = 1,
            .running = false,
        };
    }

    pub fn deinit(self: *EventLoop) void {
        // Free all remaining timer callbacks
        for (self.timers.items) |timer| {
            if (!timer.cleared) {
                c.JS_FreeValue(self.context, timer.callback);
            }
        }
        self.timers.deinit(self.allocator);
    }

    pub fn addTimer(self: *EventLoop, callback: c.JSValue, delay_ms: u64, timer_type: TimerType) !u32 {
        const id = self.next_timer_id;
        self.next_timer_id += 1;

        const now = std.time.milliTimestamp();

        // Duplicate the callback to prevent it from being freed
        const cb_dup = c.JS_DupValue(self.context, callback);

        try self.timers.append(self.allocator, Timer{
            .id = id,
            .callback = cb_dup,
            .interval_ms = delay_ms,
            .next_run = now + @as(i64, @intCast(delay_ms)),
            .timer_type = timer_type,
            .cleared = false,
        });

        return id;
    }

    pub fn clearTimer(self: *EventLoop, id: u32) void {
        for (self.timers.items) |*timer| {
            if (timer.id == id and !timer.cleared) {
                timer.cleared = true;
                // Don't free callback here - will be freed in cleanupCleared
                return;
            }
        }
    }

    pub fn hasPendingTimers(self: *EventLoop) bool {
        for (self.timers.items) |timer| {
            if (!timer.cleared) {
                return true;
            }
        }
        return false;
    }

    pub fn run(self: *EventLoop) void {
        self.running = true;

        while (self.running and self.hasPendingTimers()) {
            const now = std.time.milliTimestamp();
            var executed_any = false;

            // Check and execute ready timers
            var i: usize = 0;
            while (i < self.timers.items.len) {
                var timer = &self.timers.items[i];

                if (timer.cleared) {
                    i += 1;
                    continue;
                }

                if (now >= timer.next_run) {
                    // Execute callback
                    const result = c.JS_Call(
                        self.context,
                        timer.callback,
                        engine.makeUndefined(),
                        0,
                        null,
                    );

                    // Handle exception
                    if (c.JS_IsException(result) != 0) {
                        self.dumpError();
                    }
                    c.JS_FreeValue(self.context, result);

                    executed_any = true;

                    // Handle interval vs timeout
                    if (timer.timer_type == .interval and !timer.cleared) {
                        timer.next_run = now + @as(i64, @intCast(timer.interval_ms));
                    } else {
                        // Mark as cleared - callback will be freed in cleanupCleared
                        timer.cleared = true;
                    }
                }

                i += 1;
            }

            // Execute pending jobs (Promises)
            _ = self.executePendingJobs();

            // Small sleep to prevent CPU spinning
            if (!executed_any) {
                std.Thread.sleep(1 * std.time.ns_per_ms);
            }

            // Cleanup cleared timers periodically
            self.cleanupCleared();
        }

        self.running = false;
    }

    pub fn executePendingJobs(self: *EventLoop) bool {
        var ctx: ?*c.JSContext = null;
        var executed = false;

        while (true) {
            const ret = c.JS_ExecutePendingJob(c.JS_GetRuntime(self.context), &ctx);
            if (ret <= 0) {
                if (ret < 0) {
                    self.dumpError();
                }
                break;
            }
            executed = true;
        }

        return executed;
    }

    pub fn stop(self: *EventLoop) void {
        self.running = false;
    }

    fn cleanupCleared(self: *EventLoop) void {
        var i: usize = 0;
        while (i < self.timers.items.len) {
            const timer = &self.timers.items[i];
            if (timer.cleared) {
                c.JS_FreeValue(self.context, timer.callback);
                _ = self.timers.orderedRemove(i);
            } else {
                i += 1;
            }
        }
    }

    fn dumpError(self: *EventLoop) void {
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
};

// Global event loop pointer for C callbacks
var global_event_loop: ?*EventLoop = null;

pub fn setGlobalEventLoop(loop: *EventLoop) void {
    global_event_loop = loop;
}

pub fn getGlobalEventLoop() ?*EventLoop {
    return global_event_loop;
}

// JavaScript API functions
pub fn jsSetTimeout(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "setTimeout requires at least 1 argument");
    }

    const callback = argv[0];
    if (c.JS_IsFunction(ctx, callback) == 0) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a function");
    }

    var delay: f64 = 0;
    if (argc >= 2) {
        _ = c.JS_ToFloat64(ctx, &delay, argv[1]);
    }
    if (delay < 0) delay = 0;

    const loop = getGlobalEventLoop() orelse {
        return c.JS_ThrowInternalError(ctx, "Event loop not initialized");
    };

    const id = loop.addTimer(callback, @intFromFloat(delay), .timeout) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to create timer");
    };

    return c.JS_NewInt32(ctx, @intCast(id));
}

pub fn jsSetInterval(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "setInterval requires at least 1 argument");
    }

    const callback = argv[0];
    if (c.JS_IsFunction(ctx, callback) == 0) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a function");
    }

    var delay: f64 = 0;
    if (argc >= 2) {
        _ = c.JS_ToFloat64(ctx, &delay, argv[1]);
    }
    if (delay < 0) delay = 0;
    if (delay < 10) delay = 10; // Minimum interval

    const loop = getGlobalEventLoop() orelse {
        return c.JS_ThrowInternalError(ctx, "Event loop not initialized");
    };

    const id = loop.addTimer(callback, @intFromFloat(delay), .interval) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to create timer");
    };

    return c.JS_NewInt32(ctx, @intCast(id));
}

pub fn jsClearTimeout(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return engine.makeUndefined();
    }

    var id: i32 = 0;
    _ = c.JS_ToInt32(ctx, &id, argv[0]);

    if (id > 0) {
        const loop = getGlobalEventLoop() orelse {
            return engine.makeUndefined();
        };
        loop.clearTimer(@intCast(id));
    }

    return engine.makeUndefined();
}

pub fn jsClearInterval(
    ctx: ?*c.JSContext,
    this: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    // Same implementation as clearTimeout
    return jsClearTimeout(ctx, this, argc, argv);
}

// Register timer functions to global object
pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    eng.setProperty(global, "setTimeout", eng.newCFunction(jsSetTimeout, "setTimeout", 2));
    eng.setProperty(global, "setInterval", eng.newCFunction(jsSetInterval, "setInterval", 2));
    eng.setProperty(global, "clearTimeout", eng.newCFunction(jsClearTimeout, "clearTimeout", 1));
    eng.setProperty(global, "clearInterval", eng.newCFunction(jsClearInterval, "clearInterval", 1));
}
