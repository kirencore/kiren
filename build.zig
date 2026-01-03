const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // QuickJS C source files
    const quickjs_sources = [_][]const u8{
        "deps/quickjs/quickjs.c",
        "deps/quickjs/libregexp.c",
        "deps/quickjs/libunicode.c",
        "deps/quickjs/cutils.c",
        "deps/quickjs/libbf.c",
        "deps/quickjs/quickjs-libc.c",
    };

    // SQLite C source files
    const sqlite_sources = [_][]const u8{
        "deps/sqlite/sqlite3.c",
    };

    // C compiler flags for QuickJS
    const c_flags = [_][]const u8{
        "-DCONFIG_VERSION=\"2024-01-13\"",
        "-DCONFIG_BIGNUM",
        "-D_GNU_SOURCE",
        "-fno-sanitize=undefined", // QuickJS relies on C undefined behavior
        "-fwrapv", // Allow signed integer wrapping
    };

    // C compiler flags for SQLite
    const sqlite_flags = [_][]const u8{
        "-DSQLITE_DQS=0", // Disable double-quoted string literals
        "-DSQLITE_THREADSAFE=0", // Single-threaded for simplicity
        "-DSQLITE_DEFAULT_MEMSTATUS=0", // Disable memory status
        "-DSQLITE_DEFAULT_WAL_SYNCHRONOUS=1", // Normal sync for WAL
        "-DSQLITE_LIKE_DOESNT_MATCH_BLOBS", // Optimize LIKE
        "-DSQLITE_MAX_EXPR_DEPTH=0", // Unlimited expression depth
        "-DSQLITE_OMIT_DECLTYPE", // Omit decltype
        "-DSQLITE_OMIT_DEPRECATED", // Omit deprecated features
        "-DSQLITE_OMIT_PROGRESS_CALLBACK", // Omit progress callback
        "-DSQLITE_OMIT_SHARED_CACHE", // Omit shared cache
        "-DSQLITE_USE_ALLOCA", // Use alloca for temp allocations
    };

    // Main executable
    const exe = b.addExecutable(.{
        .name = "kiren",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    // Compile QuickJS C files
    exe.addCSourceFiles(.{
        .files = &quickjs_sources,
        .flags = &c_flags,
    });

    // Compile SQLite C files
    exe.addCSourceFiles(.{
        .files = &sqlite_sources,
        .flags = &sqlite_flags,
    });

    // Add include paths
    exe.root_module.addIncludePath(b.path("deps/quickjs"));
    exe.root_module.addIncludePath(b.path("deps/sqlite"));

    // Link libc
    exe.root_module.link_libc = true;

    // Install
    b.installArtifact(exe);

    // Run command
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run Kiren");
    run_step.dependOn(&run_cmd.step);

    // Tests
    const unit_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    unit_tests.addCSourceFiles(.{
        .files = &quickjs_sources,
        .flags = &c_flags,
    });
    unit_tests.addCSourceFiles(.{
        .files = &sqlite_sources,
        .flags = &sqlite_flags,
    });
    unit_tests.root_module.addIncludePath(b.path("deps/quickjs"));
    unit_tests.root_module.addIncludePath(b.path("deps/sqlite"));
    unit_tests.root_module.link_libc = true;

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&run_unit_tests.step);
}
