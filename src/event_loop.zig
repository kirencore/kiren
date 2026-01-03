const std = @import("std");
const engine = @import("engine.zig");
const job_queue = @import("job_queue.zig");
const poller = @import("io/poller.zig");
const c = engine.c;

const Allocator = std.mem.Allocator;
const JobQueue = job_queue.JobQueue;
const Poller = poller.Poller;
const IoEvent = poller.IoEvent;

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

/// I/O callback registration
pub const IoHandler = struct {
    fd: std.posix.fd_t,
    callback: c.JSValue,
    event_type: poller.EventType,
    one_shot: bool,
};

/// Unified Event Loop with async I/O, timers, and job queue integration
pub const EventLoop = struct {
    allocator: Allocator,
    context: *c.JSContext,
    /// Job queue for microtasks and async jobs
    jobs: JobQueue,
    /// I/O poller for async I/O
    io_poller: Poller,
    /// Timer storage
    timers: std.ArrayListUnmanaged(Timer),
    /// I/O handlers
    io_handlers: std.AutoHashMap(std.posix.fd_t, IoHandler),
    /// Next timer ID
    next_timer_id: u32,
    /// Whether the event loop is running
    running: bool,
    /// Whether we should exit after current iteration
    should_exit: bool,

    pub fn init(allocator: Allocator, context: *c.JSContext) !EventLoop {
        return EventLoop{
            .allocator = allocator,
            .context = context,
            .jobs = JobQueue.init(allocator, context),
            .io_poller = try Poller.init(allocator),
            .timers = .{},
            .io_handlers = std.AutoHashMap(std.posix.fd_t, IoHandler).init(allocator),
            .next_timer_id = 1,
            .running = false,
            .should_exit = false,
        };
    }

    pub fn deinit(self: *EventLoop) void {
        // Drain any remaining QuickJS jobs before cleanup
        self.jobs.executeQuickJSJobs();

        // Free all remaining timer callbacks
        for (self.timers.items) |timer| {
            if (!timer.cleared) {
                c.JS_FreeValue(self.context, timer.callback);
            }
        }
        self.timers.deinit(self.allocator);

        // Free I/O handler callbacks
        var it = self.io_handlers.valueIterator();
        while (it.next()) |handler| {
            c.JS_FreeValue(self.context, handler.callback);
        }
        self.io_handlers.deinit();

        self.jobs.deinit();
        self.io_poller.deinit();
    }

    // ===== Timer API =====

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
                return;
            }
        }
    }

    // ===== I/O API =====

    /// Register a file descriptor for async I/O
    pub fn registerIo(self: *EventLoop, fd: std.posix.fd_t, event_type: poller.EventType, callback: c.JSValue, one_shot: bool) !void {
        const cb_dup = c.JS_DupValue(self.context, callback);

        try self.io_handlers.put(fd, IoHandler{
            .fd = fd,
            .callback = cb_dup,
            .event_type = event_type,
            .one_shot = one_shot,
        });

        try self.io_poller.register(fd, event_type, null, null);
    }

    /// Unregister a file descriptor
    pub fn unregisterIo(self: *EventLoop, fd: std.posix.fd_t) void {
        if (self.io_handlers.fetchRemove(fd)) |entry| {
            c.JS_FreeValue(self.context, entry.value.callback);
        }
        self.io_poller.unregister(fd);
    }

    // ===== Main Event Loop =====

    /// Check if there is pending work
    pub fn hasPendingWork(self: *EventLoop) bool {
        // Check for pending jobs
        if (self.jobs.hasPendingJobs()) return true;

        // Check for active timers
        for (self.timers.items) |timer| {
            if (!timer.cleared) return true;
        }

        // Check for I/O handlers
        if (self.io_handlers.count() > 0) return true;

        return false;
    }

    /// Get timeout until next timer (in milliseconds)
    fn getNextTimerTimeout(self: *EventLoop) i32 {
        var min_timeout: i64 = std.math.maxInt(i64);
        const now = std.time.milliTimestamp();

        for (self.timers.items) |timer| {
            if (!timer.cleared) {
                const remaining = timer.next_run - now;
                if (remaining < min_timeout) {
                    min_timeout = remaining;
                }
            }
        }

        if (min_timeout == std.math.maxInt(i64)) {
            // No timers, use default timeout or infinite if no I/O
            return if (self.io_handlers.count() > 0) 100 else -1;
        }

        // Clamp to valid range
        if (min_timeout <= 0) return 0;
        if (min_timeout > 60000) return 60000; // Max 60 seconds
        return @intCast(min_timeout);
    }

    /// Execute ready timers
    fn executeReadyTimers(self: *EventLoop) void {
        const now = std.time.milliTimestamp();

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

                // Drain microtasks after timer execution
                self.jobs.drainMicrotasks();

                // Handle interval vs timeout
                if (timer.timer_type == .interval and !timer.cleared) {
                    timer.next_run = now + @as(i64, @intCast(timer.interval_ms));
                } else {
                    timer.cleared = true;
                }
            }

            i += 1;
        }
    }

    /// Handle I/O events
    fn handleIoEvents(self: *EventLoop, events: []IoEvent) void {
        for (events) |event| {
            if (self.io_handlers.get(event.fd)) |handler| {
                // Create event object for JavaScript
                const event_obj = c.JS_NewObject(self.context);
                _ = c.JS_SetPropertyStr(self.context, event_obj, "fd", c.JS_NewInt32(self.context, event.fd));
                _ = c.JS_SetPropertyStr(self.context, event_obj, "readable", engine.makeBool(event.readable));
                _ = c.JS_SetPropertyStr(self.context, event_obj, "writable", engine.makeBool(event.writable));
                _ = c.JS_SetPropertyStr(self.context, event_obj, "error", engine.makeBool(event.error_occurred));
                _ = c.JS_SetPropertyStr(self.context, event_obj, "hangup", engine.makeBool(event.hangup));

                // Call the callback
                var args = [_]c.JSValue{event_obj};
                const result = c.JS_Call(
                    self.context,
                    handler.callback,
                    engine.makeUndefined(),
                    1,
                    &args,
                );

                // Handle exception
                if (c.JS_IsException(result) != 0) {
                    self.dumpError();
                }
                c.JS_FreeValue(self.context, result);
                c.JS_FreeValue(self.context, event_obj);

                // Drain microtasks after I/O callback
                self.jobs.drainMicrotasks();

                // Remove if one-shot
                if (handler.one_shot) {
                    self.unregisterIo(event.fd);
                }
            }
        }
    }

    /// Run the event loop
    pub fn run(self: *EventLoop) void {
        self.running = true;
        self.should_exit = false;

        while (self.running and !self.should_exit and self.hasPendingWork()) {
            // 1. Drain all microtasks first
            self.jobs.drainMicrotasks();

            // 2. Calculate timeout based on nearest timer
            const timeout = self.getNextTimerTimeout();

            // 3. Poll for I/O events
            const io_events = self.io_poller.poll(timeout) catch |err| {
                std.debug.print("I/O poll error: {}\n", .{err});
                continue;
            };

            // 4. Handle I/O events
            if (io_events.len > 0) {
                self.handleIoEvents(io_events);
            }

            // 5. Execute ready timers
            self.executeReadyTimers();

            // 6. Execute immediate callbacks
            while (self.jobs.executeNextImmediate()) {
                // Keep executing immediates until none left
            }

            // 7. Cleanup cleared timers periodically
            self.cleanupCleared();

            // If no I/O or timers and only jobs, give CPU a break
            if (io_events.len == 0 and !self.hasActiveTimers()) {
                if (self.jobs.hasPendingJobs()) {
                    // Just process jobs without sleeping
                    continue;
                } else if (self.io_handlers.count() == 0) {
                    // Nothing to do, exit
                    break;
                }
            }
        }

        self.running = false;
    }

    /// Run one iteration of the event loop (for integration with external loops)
    pub fn runOnce(self: *EventLoop) bool {
        // Drain microtasks
        self.jobs.drainMicrotasks();

        // Poll with 0 timeout (non-blocking)
        const io_events = self.io_poller.poll(0) catch return false;

        // Handle I/O
        if (io_events.len > 0) {
            self.handleIoEvents(io_events);
        }

        // Execute ready timers
        self.executeReadyTimers();

        // Execute one immediate
        _ = self.jobs.executeNextImmediate();

        return self.hasPendingWork();
    }

    /// Stop the event loop
    pub fn stop(self: *EventLoop) void {
        self.should_exit = true;
    }

    fn hasActiveTimers(self: *EventLoop) bool {
        for (self.timers.items) |timer| {
            if (!timer.cleared) return true;
        }
        return false;
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

// Global event loop pointer
var global_event_loop: ?*EventLoop = null;

pub fn setGlobalEventLoop(loop: *EventLoop) void {
    global_event_loop = loop;
    // Also set the job queue global
    job_queue.setGlobalJobQueue(&loop.jobs);
    // And the poller global
    poller.setGlobalPoller(&loop.io_poller);
}

pub fn getGlobalEventLoop() ?*EventLoop {
    return global_event_loop;
}

/// Convenience function to drain microtasks from anywhere
pub fn drainMicrotasks() void {
    if (global_event_loop) |loop| {
        loop.jobs.drainMicrotasks();
    }
}

// ===== JavaScript API functions =====

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
    return jsClearTimeout(ctx, this, argc, argv);
}

/// Register timer functions to global object
pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    eng.setProperty(global, "setTimeout", eng.newCFunction(jsSetTimeout, "setTimeout", 2));
    eng.setProperty(global, "setInterval", eng.newCFunction(jsSetInterval, "setInterval", 2));
    eng.setProperty(global, "clearTimeout", eng.newCFunction(jsClearTimeout, "clearTimeout", 1));
    eng.setProperty(global, "clearInterval", eng.newCFunction(jsClearInterval, "clearInterval", 1));

    // Also register job queue functions
    job_queue.register(eng);
}
