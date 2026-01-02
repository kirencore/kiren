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

    // C compiler flags
    const c_flags = [_][]const u8{
        "-DCONFIG_VERSION=\"2024-01-13\"",
        "-DCONFIG_BIGNUM",
        "-D_GNU_SOURCE",
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

    // Add include path
    exe.root_module.addIncludePath(b.path("deps/quickjs"));

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
    unit_tests.root_module.addIncludePath(b.path("deps/quickjs"));
    unit_tests.root_module.link_libc = true;

    const run_unit_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&run_unit_tests.step);
}
