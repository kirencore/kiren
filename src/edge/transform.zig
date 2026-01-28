const std = @import("std");

/// ES Module -> CommonJS transform
/// Transforms:
///   export default X  ->  module.exports = X
///   export { X }      ->  module.exports.X = X
///   import X from 'Y' ->  const X = require('Y')
pub fn transformEsModule(allocator: std.mem.Allocator, source: []const u8) ![]u8 {
    var output: std.ArrayListUnmanaged(u8) = .{};
    errdefer output.deinit(allocator);
    const writer = output.writer(allocator);

    var i: usize = 0;
    while (i < source.len) {
        // Skip whitespace at line start for detection
        const line_start = i;
        while (i < source.len and (source[i] == ' ' or source[i] == '\t')) {
            i += 1;
        }

        // Check for "export default"
        if (i + 14 <= source.len and std.mem.eql(u8, source[i .. i + 14], "export default")) {
            // Write any leading whitespace
            try writer.writeAll(source[line_start..i]);
            // Replace with module.exports =
            try writer.writeAll("module.exports =");
            i += 14;
            continue;
        }

        // Check for "export {"
        if (i + 7 <= source.len and std.mem.eql(u8, source[i .. i + 7], "export {")) {
            // Find closing brace
            var end = i + 7;
            var brace_depth: usize = 1;
            while (end < source.len and brace_depth > 0) {
                if (source[end] == '{') brace_depth += 1;
                if (source[end] == '}') brace_depth -= 1;
                end += 1;
            }

            // Parse names between braces
            const names_str = source[i + 8 .. end - 1];
            var names = std.mem.splitScalar(u8, names_str, ',');
            while (names.next()) |name_raw| {
                const name = std.mem.trim(u8, name_raw, " \t\n\r");
                if (name.len > 0) {
                    // Handle "X as Y" syntax
                    if (std.mem.indexOf(u8, name, " as ")) |as_idx| {
                        const local = std.mem.trim(u8, name[0..as_idx], " \t");
                        const exported = std.mem.trim(u8, name[as_idx + 4 ..], " \t");
                        try writer.print("module.exports.{s} = {s};\n", .{ exported, local });
                    } else {
                        try writer.print("module.exports.{s} = {s};\n", .{ name, name });
                    }
                }
            }

            i = end;
            // Skip optional semicolon and newline
            while (i < source.len and (source[i] == ';' or source[i] == '\n' or source[i] == '\r')) {
                i += 1;
            }
            continue;
        }

        // Check for "import"
        if (i + 6 <= source.len and std.mem.eql(u8, source[i .. i + 6], "import")) {
            // Check it's a keyword (not importSomething)
            if (i + 6 < source.len and (source[i + 6] == ' ' or source[i + 6] == '\t' or source[i + 6] == '{')) {
                const line_end = std.mem.indexOfScalarPos(u8, source, i, '\n') orelse source.len;
                const line = source[i..line_end];

                if (try transformImport(allocator, line)) |transformed| {
                    defer allocator.free(transformed);
                    try writer.writeAll(transformed);
                    i = line_end;
                    continue;
                }
            }
        }

        // Check for "export const/let/var/function/class"
        if (i + 7 <= source.len and std.mem.eql(u8, source[i .. i + 7], "export ")) {
            const after_export = source[i + 7 ..];

            // export const X = ...
            if (after_export.len >= 6 and std.mem.eql(u8, after_export[0..6], "const ")) {
                try writer.writeAll("const ");
                const name_start: usize = 6;
                var name_end = name_start;
                while (name_end < after_export.len and after_export[name_end] != ' ' and after_export[name_end] != '=') {
                    name_end += 1;
                }
                const var_name = after_export[name_start..name_end];
                i += 7; // Skip "export "

                // Find end of statement (next newline or semicolon at depth 0)
                const stmt_end = findStatementEnd(source, i);
                try writer.writeAll(source[i..stmt_end]);

                // Add export
                try writer.print("\nmodule.exports.{s} = {s};", .{ var_name, var_name });

                i = stmt_end;
                continue;
            }

            // export function X() {} or export class X {}
            if ((after_export.len >= 9 and std.mem.eql(u8, after_export[0..9], "function ")) or
                (after_export.len >= 6 and std.mem.eql(u8, after_export[0..6], "class ")))
            {
                const is_func = after_export[0] == 'f';
                const keyword_len: usize = if (is_func) 9 else 6;

                var name_end = keyword_len;
                while (name_end < after_export.len and
                    after_export[name_end] != ' ' and
                    after_export[name_end] != '(' and
                    after_export[name_end] != '{')
                {
                    name_end += 1;
                }
                const func_name = after_export[keyword_len..name_end];

                i += 7; // Skip "export "

                // Find end of function/class (matching braces)
                const block_end = findBlockEnd(source, i);
                try writer.writeAll(source[i..block_end]);

                // Add export
                try writer.print("\nmodule.exports.{s} = {s};", .{ func_name, func_name });

                i = block_end;
                continue;
            }
        }

        // Reset to line start and write character by character
        i = line_start;
        try writer.writeByte(source[i]);
        i += 1;
    }

    return output.toOwnedSlice(allocator);
}

fn transformImport(allocator: std.mem.Allocator, line: []const u8) !?[]u8 {
    // Find "from" keyword
    const from_idx = std.mem.indexOf(u8, line, " from ") orelse return null;

    const import_part = std.mem.trim(u8, line[6..from_idx], " \t");

    // Find module path (between quotes)
    const after_from = line[from_idx + 6 ..];
    var path_start: usize = 0;
    while (path_start < after_from.len and after_from[path_start] != '\'' and after_from[path_start] != '"') {
        path_start += 1;
    }
    if (path_start >= after_from.len) return null;

    const quote = after_from[path_start];
    path_start += 1;
    var path_end = path_start;
    while (path_end < after_from.len and after_from[path_end] != quote) {
        path_end += 1;
    }
    if (path_end >= after_from.len) return null;

    const mod_path = after_from[path_start..path_end];

    // Check import type
    if (std.mem.startsWith(u8, import_part, "{")) {
        // Named imports: import { X, Y } from 'Z'
        const names_end = std.mem.indexOf(u8, import_part, "}") orelse return null;
        const names_str = import_part[1..names_end];

        var result: std.ArrayListUnmanaged(u8) = .{};
        const writer = result.writer(allocator);

        try writer.print("const __mod_{s} = require('{s}');\n", .{ sanitizeName(mod_path), mod_path });

        var names = std.mem.splitScalar(u8, names_str, ',');
        while (names.next()) |name_raw| {
            const name = std.mem.trim(u8, name_raw, " \t\n\r");
            if (name.len > 0) {
                if (std.mem.indexOf(u8, name, " as ")) |as_idx| {
                    const orig = std.mem.trim(u8, name[0..as_idx], " \t");
                    const alias = std.mem.trim(u8, name[as_idx + 4 ..], " \t");
                    try writer.print("const {s} = __mod_{s}.{s};\n", .{ alias, sanitizeName(mod_path), orig });
                } else {
                    try writer.print("const {s} = __mod_{s}.{s};\n", .{ name, sanitizeName(mod_path), name });
                }
            }
        }

        return try result.toOwnedSlice(allocator);
    } else if (std.mem.startsWith(u8, import_part, "* as ")) {
        // Namespace import: import * as X from 'Y'
        const alias = std.mem.trim(u8, import_part[5..], " \t");
        return try std.fmt.allocPrint(allocator, "const {s} = require('{s}');\n", .{ alias, mod_path });
    } else {
        // Default import: import X from 'Y'
        return try std.fmt.allocPrint(
            allocator,
            "const {s} = (function() {{ const m = require('{s}'); return m && m.default ? m.default : m; }})();\n",
            .{ import_part, mod_path },
        );
    }
}

fn sanitizeName(path: []const u8) []const u8 {
    // Return a safe identifier from path
    // For simplicity, just return the basename without extension
    var start: usize = 0;
    for (path, 0..) |ch, idx| {
        if (ch == '/' or ch == '\\') start = idx + 1;
    }
    var end = path.len;
    for (0..path.len) |idx| {
        const i = path.len - 1 - idx;
        if (path[i] == '.') {
            end = i;
            break;
        }
    }
    if (end <= start) return "mod";
    return path[start..end];
}

fn findStatementEnd(source: []const u8, start: usize) usize {
    var i = start;
    var brace_depth: usize = 0;
    var paren_depth: usize = 0;
    var in_string: u8 = 0;

    while (i < source.len) {
        const ch = source[i];

        if (in_string != 0) {
            if (ch == in_string and (i == 0 or source[i - 1] != '\\')) {
                in_string = 0;
            }
        } else {
            if (ch == '"' or ch == '\'' or ch == '`') {
                in_string = ch;
            } else if (ch == '{') {
                brace_depth += 1;
            } else if (ch == '}') {
                if (brace_depth > 0) brace_depth -= 1;
            } else if (ch == '(') {
                paren_depth += 1;
            } else if (ch == ')') {
                if (paren_depth > 0) paren_depth -= 1;
            } else if (ch == ';' and brace_depth == 0 and paren_depth == 0) {
                return i + 1;
            } else if (ch == '\n' and brace_depth == 0 and paren_depth == 0) {
                // Check if next non-whitespace is not a continuation
                var j = i + 1;
                while (j < source.len and (source[j] == ' ' or source[j] == '\t')) {
                    j += 1;
                }
                if (j >= source.len) return i;
                const next = source[j];
                if (next != '.' and next != '+' and next != '-' and next != '*' and next != '/' and next != '?' and next != ':') {
                    return i;
                }
            }
        }
        i += 1;
    }
    return source.len;
}

fn findBlockEnd(source: []const u8, start: usize) usize {
    var i = start;
    var brace_depth: usize = 0;
    var found_open = false;
    var in_string: u8 = 0;

    while (i < source.len) {
        const ch = source[i];

        if (in_string != 0) {
            if (ch == in_string and (i == 0 or source[i - 1] != '\\')) {
                in_string = 0;
            }
        } else {
            if (ch == '"' or ch == '\'' or ch == '`') {
                in_string = ch;
            } else if (ch == '{') {
                brace_depth += 1;
                found_open = true;
            } else if (ch == '}') {
                brace_depth -= 1;
                if (found_open and brace_depth == 0) {
                    return i + 1;
                }
            }
        }
        i += 1;
    }
    return source.len;
}

/// Check if source contains ES module syntax
pub fn isEsModule(source: []const u8) bool {
    var i: usize = 0;
    while (i < source.len) {
        // Skip strings
        if (source[i] == '"' or source[i] == '\'' or source[i] == '`') {
            const quote = source[i];
            i += 1;
            while (i < source.len) {
                if (source[i] == quote and source[i - 1] != '\\') break;
                i += 1;
            }
            i += 1;
            continue;
        }

        // Skip comments
        if (i + 1 < source.len and source[i] == '/') {
            if (source[i + 1] == '/') {
                // Line comment
                while (i < source.len and source[i] != '\n') i += 1;
                continue;
            } else if (source[i + 1] == '*') {
                // Block comment
                i += 2;
                while (i + 1 < source.len) {
                    if (source[i] == '*' and source[i + 1] == '/') {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }

        // Check for export/import at word boundary
        if (i + 7 <= source.len and std.mem.eql(u8, source[i .. i + 7], "export ")) {
            // Make sure it's not part of another word
            if (i == 0 or !isIdentChar(source[i - 1])) {
                return true;
            }
        }

        if (i + 7 <= source.len and std.mem.eql(u8, source[i .. i + 7], "import ")) {
            if (i == 0 or !isIdentChar(source[i - 1])) {
                return true;
            }
        }

        i += 1;
    }
    return false;
}

fn isIdentChar(ch: u8) bool {
    return (ch >= 'a' and ch <= 'z') or
        (ch >= 'A' and ch <= 'Z') or
        (ch >= '0' and ch <= '9') or
        ch == '_' or ch == '$';
}
