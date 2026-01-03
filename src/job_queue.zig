const std = @import("std");
const engine = @import("engine.zig");
const c = engine.c;

const Allocator = std.mem.Allocator;

/// Job types for the async runtime
pub const JobType = enum {
    /// Promise microtask (highest priority)
    microtask,
    /// Timer callback
    timer,
    /// I/O completion callback
    io_completion,
    /// Immediate callback (setImmediate equivalent)
    immediate,
};

/// A job in the queue
pub const Job = struct {
    job_type: JobType,
    callback: c.JSValue,
    /// Optional data associated with the job
    data: ?*anyopaque = null,
    /// Cleanup function for data
    cleanup: ?*const fn (?*anyopaque) void = null,
};

/// Job Queue for managing Promise microtasks and other async jobs
pub const JobQueue = struct {
    allocator: Allocator,
    context: *c.JSContext,
    /// High priority queue (microtasks)
    microtasks: std.ArrayListUnmanaged(Job),
    /// Normal priority queue (timers, I/O)
    tasks: std.ArrayListUnmanaged(Job),
    /// Low priority queue (immediate)
    immediates: std.ArrayListUnmanaged(Job),
    /// Whether we're currently draining the queue
    draining: bool,

    pub fn init(allocator: Allocator, context: *c.JSContext) JobQueue {
        return JobQueue{
            .allocator = allocator,
            .context = context,
            .microtasks = .{},
            .tasks = .{},
            .immediates = .{},
            .draining = false,
        };
    }

    pub fn deinit(self: *JobQueue) void {
        // Free all remaining callbacks
        for (self.microtasks.items) |job| {
            c.JS_FreeValue(self.context, job.callback);
            if (job.cleanup) |cleanup| {
                cleanup(job.data);
            }
        }
        for (self.tasks.items) |job| {
            c.JS_FreeValue(self.context, job.callback);
            if (job.cleanup) |cleanup| {
                cleanup(job.data);
            }
        }
        for (self.immediates.items) |job| {
            c.JS_FreeValue(self.context, job.callback);
            if (job.cleanup) |cleanup| {
                cleanup(job.data);
            }
        }

        self.microtasks.deinit(self.allocator);
        self.tasks.deinit(self.allocator);
        self.immediates.deinit(self.allocator);
    }

    /// Enqueue a job
    pub fn enqueue(self: *JobQueue, job: Job) !void {
        // Duplicate callback to prevent premature GC
        const cb_dup = c.JS_DupValue(self.context, job.callback);
        const job_with_dup = Job{
            .job_type = job.job_type,
            .callback = cb_dup,
            .data = job.data,
            .cleanup = job.cleanup,
        };

        switch (job.job_type) {
            .microtask => try self.microtasks.append(self.allocator, job_with_dup),
            .timer, .io_completion => try self.tasks.append(self.allocator, job_with_dup),
            .immediate => try self.immediates.append(self.allocator, job_with_dup),
        }
    }

    /// Enqueue a microtask (highest priority)
    pub fn enqueueMicrotask(self: *JobQueue, callback: c.JSValue) !void {
        try self.enqueue(.{
            .job_type = .microtask,
            .callback = callback,
        });
    }

    /// Check if there are pending jobs
    pub fn hasPendingJobs(self: *JobQueue) bool {
        return self.microtasks.items.len > 0 or
            self.tasks.items.len > 0 or
            self.immediates.items.len > 0;
    }

    /// Check if there are pending microtasks
    pub fn hasPendingMicrotasks(self: *JobQueue) bool {
        return self.microtasks.items.len > 0;
    }

    /// Drain all microtasks (run until empty)
    /// This is called after every I/O completion or timer execution
    pub fn drainMicrotasks(self: *JobQueue) void {
        if (self.draining) return;
        self.draining = true;
        defer self.draining = false;

        // First, execute QuickJS pending jobs (Promise callbacks)
        self.executeQuickJSJobs();

        // Then execute our microtask queue
        while (self.microtasks.items.len > 0) {
            const job = self.microtasks.orderedRemove(0);
            self.executeJob(job);

            // After each microtask, check for more QuickJS jobs
            self.executeQuickJSJobs();
        }
    }

    /// Execute all QuickJS pending jobs (Promise.then callbacks etc.)
    pub fn executeQuickJSJobs(self: *JobQueue) void {
        var ctx: ?*c.JSContext = null;

        while (true) {
            const ret = c.JS_ExecutePendingJob(c.JS_GetRuntime(self.context), &ctx);
            if (ret <= 0) {
                if (ret < 0) {
                    self.dumpError();
                }
                break;
            }
        }
    }

    /// Execute a single job from the task queue
    pub fn executeNextTask(self: *JobQueue) bool {
        if (self.tasks.items.len == 0) return false;

        const job = self.tasks.orderedRemove(0);
        self.executeJob(job);

        // Drain microtasks after task execution
        self.drainMicrotasks();

        return true;
    }

    /// Execute a single job from the immediate queue
    pub fn executeNextImmediate(self: *JobQueue) bool {
        if (self.immediates.items.len == 0) return false;

        const job = self.immediates.orderedRemove(0);
        self.executeJob(job);

        // Drain microtasks after immediate execution
        self.drainMicrotasks();

        return true;
    }

    /// Execute a job
    fn executeJob(self: *JobQueue, job: Job) void {
        const result = c.JS_Call(
            self.context,
            job.callback,
            engine.makeUndefined(),
            0,
            null,
        );

        // Handle exception
        if (c.JS_IsException(result) != 0) {
            self.dumpError();
        }
        c.JS_FreeValue(self.context, result);

        // Free the callback
        c.JS_FreeValue(self.context, job.callback);

        // Cleanup data if needed
        if (job.cleanup) |cleanup| {
            cleanup(job.data);
        }
    }

    fn dumpError(self: *JobQueue) void {
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

// Global job queue pointer
var global_job_queue: ?*JobQueue = null;

pub fn setGlobalJobQueue(queue: *JobQueue) void {
    global_job_queue = queue;
}

pub fn getGlobalJobQueue() ?*JobQueue {
    return global_job_queue;
}

/// Convenience function to drain microtasks from anywhere
pub fn drainMicrotasks() void {
    if (global_job_queue) |queue| {
        queue.drainMicrotasks();
    }
}

// JavaScript API functions

/// queueMicrotask(callback) - Queue a microtask
pub fn jsQueueMicrotask(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "queueMicrotask requires 1 argument");
    }

    const callback = argv[0];
    if (c.JS_IsFunction(ctx, callback) == 0) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a function");
    }

    const queue = getGlobalJobQueue() orelse {
        return c.JS_ThrowInternalError(ctx, "Job queue not initialized");
    };

    queue.enqueueMicrotask(callback) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to queue microtask");
    };

    return engine.makeUndefined();
}

/// setImmediate(callback) - Queue an immediate callback (runs after I/O)
pub fn jsSetImmediate(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "setImmediate requires 1 argument");
    }

    const callback = argv[0];
    if (c.JS_IsFunction(ctx, callback) == 0) {
        return c.JS_ThrowTypeError(ctx, "First argument must be a function");
    }

    const queue = getGlobalJobQueue() orelse {
        return c.JS_ThrowInternalError(ctx, "Job queue not initialized");
    };

    queue.enqueue(.{
        .job_type = .immediate,
        .callback = callback,
    }) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to queue immediate");
    };

    return engine.makeUndefined();
}

/// Register job queue functions to global object
pub fn register(eng: *engine.Engine) void {
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    eng.setProperty(global, "queueMicrotask", eng.newCFunction(jsQueueMicrotask, "queueMicrotask", 1));
    eng.setProperty(global, "setImmediate", eng.newCFunction(jsSetImmediate, "setImmediate", 1));
}
