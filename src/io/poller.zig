const std = @import("std");
const builtin = @import("builtin");
const posix = std.posix;

const Allocator = std.mem.Allocator;

/// Event types that can be monitored
pub const EventType = enum {
    read,
    write,
    read_write,
};

/// An I/O event returned by poll()
pub const IoEvent = struct {
    fd: posix.fd_t,
    readable: bool,
    writable: bool,
    error_occurred: bool,
    hangup: bool,
    /// User data associated with this fd
    data: ?*anyopaque,
};

/// Callback type for I/O completion
pub const IoCallback = *const fn (event: IoEvent) void;

/// Registration info for a file descriptor
const FdRegistration = struct {
    fd: posix.fd_t,
    event_type: EventType,
    callback: ?IoCallback,
    data: ?*anyopaque,
};

/// Cross-platform I/O Poller
/// Uses kqueue on macOS/BSD, epoll on Linux
pub const Poller = struct {
    allocator: Allocator,
    /// Platform-specific poll fd
    poll_fd: posix.fd_t,
    /// Registered file descriptors
    registrations: std.AutoHashMap(posix.fd_t, FdRegistration),
    /// Event buffer for poll results
    events_buffer: []align(8) u8,

    const MAX_EVENTS = 64;

    pub fn init(allocator: Allocator) !Poller {
        const poll_fd = try createPollFd();

        // Allocate event buffer based on platform
        const buffer_size = if (builtin.os.tag == .macos or builtin.os.tag == .freebsd or builtin.os.tag == .netbsd or builtin.os.tag == .openbsd)
            @sizeOf(posix.Kevent) * MAX_EVENTS
        else if (builtin.os.tag == .linux)
            @sizeOf(std.os.linux.epoll_event) * MAX_EVENTS
        else
            @sizeOf(posix.pollfd) * MAX_EVENTS;

        const events_buffer = try allocator.alignedAlloc(u8, .@"8", buffer_size);

        return Poller{
            .allocator = allocator,
            .poll_fd = poll_fd,
            .registrations = std.AutoHashMap(posix.fd_t, FdRegistration).init(allocator),
            .events_buffer = events_buffer,
        };
    }

    pub fn deinit(self: *Poller) void {
        posix.close(self.poll_fd);
        self.registrations.deinit();
        self.allocator.free(self.events_buffer);
    }

    fn createPollFd() !posix.fd_t {
        if (builtin.os.tag == .macos or builtin.os.tag == .freebsd or builtin.os.tag == .netbsd or builtin.os.tag == .openbsd) {
            // kqueue
            return try posix.kqueue();
        } else if (builtin.os.tag == .linux) {
            // epoll
            return try posix.epoll_create1(0);
        } else {
            // Fallback - not implemented
            return error.UnsupportedPlatform;
        }
    }

    /// Register a file descriptor for monitoring
    pub fn register(self: *Poller, fd: posix.fd_t, event_type: EventType, callback: ?IoCallback, data: ?*anyopaque) !void {
        if (builtin.os.tag == .macos or builtin.os.tag == .freebsd or builtin.os.tag == .netbsd or builtin.os.tag == .openbsd) {
            try self.registerKqueue(fd, event_type);
        } else if (builtin.os.tag == .linux) {
            try self.registerEpoll(fd, event_type);
        }

        try self.registrations.put(fd, FdRegistration{
            .fd = fd,
            .event_type = event_type,
            .callback = callback,
            .data = data,
        });
    }

    /// Unregister a file descriptor
    pub fn unregister(self: *Poller, fd: posix.fd_t) void {
        if (builtin.os.tag == .macos or builtin.os.tag == .freebsd or builtin.os.tag == .netbsd or builtin.os.tag == .openbsd) {
            self.unregisterKqueue(fd);
        } else if (builtin.os.tag == .linux) {
            self.unregisterEpoll(fd);
        }

        _ = self.registrations.remove(fd);
    }

    /// Modify event type for a registered fd
    pub fn modify(self: *Poller, fd: posix.fd_t, event_type: EventType) !void {
        if (self.registrations.get(fd)) |reg| {
            var new_reg = reg;
            new_reg.event_type = event_type;

            if (builtin.os.tag == .linux) {
                try self.modifyEpoll(fd, event_type);
            } else {
                // For kqueue, unregister and re-register
                self.unregisterKqueue(fd);
                try self.registerKqueue(fd, event_type);
            }

            try self.registrations.put(fd, new_reg);
        }
    }

    /// Poll for events with timeout (in milliseconds)
    /// Returns slice of IoEvents
    /// timeout_ms: -1 for infinite, 0 for non-blocking, >0 for timeout
    pub fn poll(self: *Poller, timeout_ms: i32) ![]IoEvent {
        if (builtin.os.tag == .macos or builtin.os.tag == .freebsd or builtin.os.tag == .netbsd or builtin.os.tag == .openbsd) {
            return self.pollKqueue(timeout_ms);
        } else if (builtin.os.tag == .linux) {
            return self.pollEpoll(timeout_ms);
        } else {
            return &[_]IoEvent{};
        }
    }

    /// Check if there are any registered fds
    pub fn hasRegistrations(self: *Poller) bool {
        return self.registrations.count() > 0;
    }

    // ===== kqueue implementation (macOS/BSD) =====

    fn registerKqueue(self: *Poller, fd: posix.fd_t, event_type: EventType) !void {
        var changelist: [2]posix.Kevent = undefined;
        var num_changes: usize = 0;

        if (event_type == .read or event_type == .read_write) {
            changelist[num_changes] = posix.Kevent{
                .ident = @intCast(fd),
                .filter = posix.system.EVFILT.READ,
                .flags = posix.system.EV.ADD,
                .fflags = 0,
                .data = 0,
                .udata = @intFromPtr(self.registrations.getPtr(fd)),
            };
            num_changes += 1;
        }

        if (event_type == .write or event_type == .read_write) {
            changelist[num_changes] = posix.Kevent{
                .ident = @intCast(fd),
                .filter = posix.system.EVFILT.WRITE,
                .flags = posix.system.EV.ADD,
                .fflags = 0,
                .data = 0,
                .udata = @intFromPtr(self.registrations.getPtr(fd)),
            };
            num_changes += 1;
        }

        _ = try posix.kevent(self.poll_fd, changelist[0..num_changes], &[_]posix.Kevent{}, null);
    }

    fn unregisterKqueue(self: *Poller, fd: posix.fd_t) void {
        var changelist: [2]posix.Kevent = undefined;

        changelist[0] = posix.Kevent{
            .ident = @intCast(fd),
            .filter = posix.system.EVFILT.READ,
            .flags = posix.system.EV.DELETE,
            .fflags = 0,
            .data = 0,
            .udata = 0,
        };

        changelist[1] = posix.Kevent{
            .ident = @intCast(fd),
            .filter = posix.system.EVFILT.WRITE,
            .flags = posix.system.EV.DELETE,
            .fflags = 0,
            .data = 0,
            .udata = 0,
        };

        // Ignore errors - fd might not be registered for both
        _ = posix.kevent(self.poll_fd, &changelist, &[_]posix.Kevent{}, null) catch {};
    }

    fn pollKqueue(self: *Poller, timeout_ms: i32) ![]IoEvent {
        const events_ptr: [*]posix.Kevent = @ptrCast(@alignCast(self.events_buffer.ptr));
        const events = events_ptr[0..MAX_EVENTS];

        var timeout_val = posix.timespec{
            .sec = @divTrunc(timeout_ms, 1000),
            .nsec = @rem(timeout_ms, 1000) * 1_000_000,
        };

        const timeout_ptr: ?*const posix.timespec = if (timeout_ms < 0) null else &timeout_val;

        const num_events = try posix.kevent(self.poll_fd, &[_]posix.Kevent{}, events, timeout_ptr);

        // Convert to IoEvents (reuse buffer space after kevent data)
        const io_events_ptr: [*]IoEvent = @ptrCast(@alignCast(self.events_buffer.ptr));
        const io_events = io_events_ptr[0..num_events];

        for (events[0..num_events], 0..) |event, i| {
            const fd: posix.fd_t = @intCast(event.ident);
            const reg = self.registrations.get(fd);

            io_events[i] = IoEvent{
                .fd = fd,
                .readable = event.filter == posix.system.EVFILT.READ,
                .writable = event.filter == posix.system.EVFILT.WRITE,
                .error_occurred = (event.flags & posix.system.EV.ERROR) != 0,
                .hangup = (event.flags & posix.system.EV.EOF) != 0,
                .data = if (reg) |r| r.data else null,
            };
        }

        return io_events;
    }

    // ===== epoll implementation (Linux) =====

    fn registerEpoll(self: *Poller, fd: posix.fd_t, event_type: EventType) !void {
        var events: u32 = std.os.linux.EPOLL.ET; // Edge-triggered

        if (event_type == .read or event_type == .read_write) {
            events |= std.os.linux.EPOLL.IN;
        }
        if (event_type == .write or event_type == .read_write) {
            events |= std.os.linux.EPOLL.OUT;
        }

        var ev = std.os.linux.epoll_event{
            .events = events,
            .data = .{ .fd = fd },
        };

        try posix.epoll_ctl(self.poll_fd, std.os.linux.EPOLL.CTL_ADD, fd, &ev);
    }

    fn unregisterEpoll(self: *Poller, fd: posix.fd_t) void {
        posix.epoll_ctl(self.poll_fd, std.os.linux.EPOLL.CTL_DEL, fd, null) catch {};
    }

    fn modifyEpoll(self: *Poller, fd: posix.fd_t, event_type: EventType) !void {
        var events: u32 = std.os.linux.EPOLL.ET;

        if (event_type == .read or event_type == .read_write) {
            events |= std.os.linux.EPOLL.IN;
        }
        if (event_type == .write or event_type == .read_write) {
            events |= std.os.linux.EPOLL.OUT;
        }

        var ev = std.os.linux.epoll_event{
            .events = events,
            .data = .{ .fd = fd },
        };

        try posix.epoll_ctl(self.poll_fd, std.os.linux.EPOLL.CTL_MOD, fd, &ev);
    }

    fn pollEpoll(self: *Poller, timeout_ms: i32) ![]IoEvent {
        const events_ptr: [*]std.os.linux.epoll_event = @ptrCast(@alignCast(self.events_buffer.ptr));
        const events = events_ptr[0..MAX_EVENTS];

        const num_events = posix.epoll_wait(self.poll_fd, events, timeout_ms);

        // Convert to IoEvents
        const io_events_ptr: [*]IoEvent = @ptrCast(@alignCast(self.events_buffer.ptr));
        const io_events = io_events_ptr[0..num_events];

        for (events[0..num_events], 0..) |event, i| {
            const fd = event.data.fd;
            const reg = self.registrations.get(fd);

            io_events[i] = IoEvent{
                .fd = fd,
                .readable = (event.events & std.os.linux.EPOLL.IN) != 0,
                .writable = (event.events & std.os.linux.EPOLL.OUT) != 0,
                .error_occurred = (event.events & std.os.linux.EPOLL.ERR) != 0,
                .hangup = (event.events & std.os.linux.EPOLL.HUP) != 0,
                .data = if (reg) |r| r.data else null,
            };
        }

        return io_events;
    }
};

// Global poller instance
var global_poller: ?*Poller = null;

pub fn setGlobalPoller(poller: *Poller) void {
    global_poller = poller;
}

pub fn getGlobalPoller() ?*Poller {
    return global_poller;
}
