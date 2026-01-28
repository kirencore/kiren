const std = @import("std");

pub const Route = struct {
    pattern: []const u8,
    worker_index: usize,
    is_wildcard: bool,
    prefix_len: usize,
};

pub const Router = struct {
    allocator: std.mem.Allocator,
    routes: std.ArrayListUnmanaged(Route),

    pub fn init(allocator: std.mem.Allocator) Router {
        return .{
            .allocator = allocator,
            .routes = .{},
        };
    }

    pub fn deinit(self: *Router) void {
        self.routes.deinit(self.allocator);
    }

    pub fn addRoute(self: *Router, pattern: []const u8, worker_index: usize) !void {
        const is_wildcard = std.mem.endsWith(u8, pattern, "*");
        const prefix_len = if (is_wildcard) pattern.len - 1 else pattern.len;

        try self.routes.append(self.allocator, .{
            .pattern = pattern,
            .worker_index = worker_index,
            .is_wildcard = is_wildcard,
            .prefix_len = prefix_len,
        });

        // Sort: most specific first (longer prefix, non-wildcard before wildcard)
        std.mem.sort(Route, self.routes.items, {}, struct {
            fn lessThan(_: void, a: Route, b: Route) bool {
                // Non-wildcard routes have priority over wildcard routes
                if (!a.is_wildcard and b.is_wildcard) return true;
                if (a.is_wildcard and !b.is_wildcard) return false;

                // Longer prefix = more specific = higher priority
                return a.prefix_len > b.prefix_len;
            }
        }.lessThan);
    }

    /// Match a URL path to a worker index
    /// Returns null if no route matches
    pub fn match(self: *Router, path: []const u8) ?usize {
        for (self.routes.items) |route| {
            if (route.is_wildcard) {
                // Wildcard match: path must start with prefix
                const prefix = route.pattern[0..route.prefix_len];
                if (std.mem.startsWith(u8, path, prefix)) {
                    return route.worker_index;
                }
            } else {
                // Exact match
                if (std.mem.eql(u8, path, route.pattern)) {
                    return route.worker_index;
                }
            }
        }
        return null;
    }

    /// Debug: print all routes
    pub fn printRoutes(self: *Router) void {
        std.debug.print("Routes ({d}):\n", .{self.routes.items.len});
        for (self.routes.items, 0..) |route, i| {
            std.debug.print("  [{d}] {s} -> worker {d} (wildcard: {}, prefix_len: {d})\n", .{
                i,
                route.pattern,
                route.worker_index,
                route.is_wildcard,
                route.prefix_len,
            });
        }
    }
};
