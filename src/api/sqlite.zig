const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;

const Allocator = std.mem.Allocator;

// SQLite C bindings
const sqlite = @cImport({
    @cInclude("sqlite3.h");
});

// Database class ID for JavaScript
var db_class_id: c.JSClassID = 0;
var stmt_class_id: c.JSClassID = 0;

// Database wrapper
const Database = struct {
    db: ?*sqlite.sqlite3,
    path: []const u8,
    allocator: Allocator,

    pub fn open(allocator: Allocator, path: []const u8) !*Database {
        var self = try allocator.create(Database);
        self.allocator = allocator;
        self.path = try allocator.dupe(u8, path);

        // Create null-terminated path
        const path_z = try allocator.allocSentinel(u8, path.len, 0);
        defer allocator.free(path_z);
        @memcpy(path_z, path);

        var db: ?*sqlite.sqlite3 = null;
        const rc = sqlite.sqlite3_open(path_z.ptr, &db);

        if (rc != sqlite.SQLITE_OK) {
            if (db) |d| {
                _ = sqlite.sqlite3_close(d);
            }
            allocator.free(self.path);
            allocator.destroy(self);
            return error.DatabaseOpenFailed;
        }

        self.db = db;
        return self;
    }

    pub fn close(self: *Database) void {
        if (self.db) |db| {
            _ = sqlite.sqlite3_close(db);
            self.db = null;
        }
        self.allocator.free(self.path);
        self.allocator.destroy(self);
    }

    pub fn exec(self: *Database, sql: [:0]const u8) !void {
        var err_msg: [*c]u8 = null;
        const rc = sqlite.sqlite3_exec(self.db, sql.ptr, null, null, &err_msg);

        if (rc != sqlite.SQLITE_OK) {
            if (err_msg) |msg| {
                std.debug.print("SQLite error: {s}\n", .{msg});
                sqlite.sqlite3_free(msg);
            }
            return error.ExecFailed;
        }
    }

    pub fn getError(self: *Database) []const u8 {
        if (self.db) |db| {
            const msg = sqlite.sqlite3_errmsg(db);
            if (msg != null) {
                return std.mem.span(msg);
            }
        }
        return "Unknown error";
    }
};

// Statement wrapper
const Statement = struct {
    stmt: ?*sqlite.sqlite3_stmt,
    db: *Database,
    allocator: Allocator,
    bound_strings: std.ArrayListUnmanaged([]u8),

    pub fn prepare(allocator: Allocator, db: *Database, sql: [:0]const u8) !*Statement {
        var self = try allocator.create(Statement);
        self.allocator = allocator;
        self.db = db;
        self.bound_strings = .{};

        var stmt: ?*sqlite.sqlite3_stmt = null;
        const rc = sqlite.sqlite3_prepare_v2(db.db, sql.ptr, @intCast(sql.len + 1), &stmt, null);

        if (rc != sqlite.SQLITE_OK) {
            self.bound_strings.deinit(allocator);
            allocator.destroy(self);
            return error.PrepareFailed;
        }

        self.stmt = stmt;
        return self;
    }

    pub fn finalize(self: *Statement) void {
        if (self.stmt) |stmt| {
            _ = sqlite.sqlite3_finalize(stmt);
            self.stmt = null;
        }
        // Free all bound strings
        for (self.bound_strings.items) |s| {
            self.allocator.free(s);
        }
        self.bound_strings.deinit(self.allocator);
        self.allocator.destroy(self);
    }

    pub fn bindValue(self: *Statement, ctx: *c.JSContext, idx: c_int, value: c.JSValue) !void {
        const stmt = self.stmt orelse return error.NoStatement;

        if (c.JS_IsNull(value) != 0 or c.JS_IsUndefined(value) != 0) {
            _ = sqlite.sqlite3_bind_null(stmt, idx);
        } else if (c.JS_IsBool(value) != 0) {
            const b = c.JS_ToBool(ctx, value);
            _ = sqlite.sqlite3_bind_int(stmt, idx, b);
        } else if (c.JS_IsNumber(value) != 0) {
            var num: f64 = 0;
            _ = c.JS_ToFloat64(ctx, &num, value);
            // Check if it's an integer
            const int_val: i64 = @intFromFloat(num);
            if (@as(f64, @floatFromInt(int_val)) == num) {
                _ = sqlite.sqlite3_bind_int64(stmt, idx, int_val);
            } else {
                _ = sqlite.sqlite3_bind_double(stmt, idx, num);
            }
        } else if (c.JS_IsString(value) != 0) {
            const str = c.JS_ToCString(ctx, value);
            if (str != null) {
                const len = std.mem.len(str);
                // Allocate a copy for SQLite
                const copy = self.allocator.alloc(u8, len) catch {
                    c.JS_FreeCString(ctx, str);
                    return;
                };
                @memcpy(copy, str[0..len]);
                c.JS_FreeCString(ctx, str);
                // Track for cleanup on finalize
                self.bound_strings.append(self.allocator, copy) catch {
                    self.allocator.free(copy);
                    return;
                };
                // Use null destructor - we manage the memory
                _ = sqlite.sqlite3_bind_text(stmt, idx, copy.ptr, @intCast(len), null);
            }
        } else {
            _ = sqlite.sqlite3_bind_null(stmt, idx);
        }
    }

    pub fn step(self: *Statement) !bool {
        const stmt = self.stmt orelse return error.NoStatement;
        const rc = sqlite.sqlite3_step(stmt);

        if (rc == sqlite.SQLITE_ROW) {
            return true;
        } else if (rc == sqlite.SQLITE_DONE) {
            return false;
        } else {
            return error.StepFailed;
        }
    }

    pub fn reset(self: *Statement) void {
        if (self.stmt) |stmt| {
            _ = sqlite.sqlite3_reset(stmt);
            _ = sqlite.sqlite3_clear_bindings(stmt);
        }
    }

    pub fn columnCount(self: *Statement) c_int {
        const stmt = self.stmt orelse return 0;
        return sqlite.sqlite3_column_count(stmt);
    }

    pub fn columnName(self: *Statement, idx: c_int) ?[*:0]const u8 {
        const stmt = self.stmt orelse return null;
        return sqlite.sqlite3_column_name(stmt, idx);
    }

    pub fn columnValue(self: *Statement, ctx: *c.JSContext, idx: c_int) c.JSValue {
        const stmt = self.stmt orelse return engine.makeNull();

        const col_type = sqlite.sqlite3_column_type(stmt, idx);

        switch (col_type) {
            sqlite.SQLITE_NULL => return engine.makeNull(),
            sqlite.SQLITE_INTEGER => {
                const val = sqlite.sqlite3_column_int64(stmt, idx);
                return c.JS_NewInt64(ctx, val);
            },
            sqlite.SQLITE_FLOAT => {
                const val = sqlite.sqlite3_column_double(stmt, idx);
                return c.JS_NewFloat64(ctx, val);
            },
            sqlite.SQLITE_TEXT => {
                const text = sqlite.sqlite3_column_text(stmt, idx);
                const len = sqlite.sqlite3_column_bytes(stmt, idx);
                if (text != null) {
                    return c.JS_NewStringLen(ctx, text, @intCast(len));
                }
                return engine.makeNull();
            },
            sqlite.SQLITE_BLOB => {
                // Return as string for now
                const blob = sqlite.sqlite3_column_blob(stmt, idx);
                const len = sqlite.sqlite3_column_bytes(stmt, idx);
                if (blob != null) {
                    return c.JS_NewStringLen(ctx, @ptrCast(blob), @intCast(len));
                }
                return engine.makeNull();
            },
            else => return engine.makeNull(),
        }
    }
};

// JavaScript API implementations

fn dbFinalizer(_: ?*c.JSRuntime, val: c.JSValue) callconv(.c) void {
    const ptr = c.JS_GetOpaque(val, db_class_id);
    if (ptr != null) {
        const db: *Database = @ptrCast(@alignCast(ptr));
        db.close();
    }
}

fn stmtFinalizer(_: ?*c.JSRuntime, val: c.JSValue) callconv(.c) void {
    const ptr = c.JS_GetOpaque(val, stmt_class_id);
    if (ptr != null) {
        const stmt: *Statement = @ptrCast(@alignCast(ptr));
        stmt.finalize();
    }
}

// Kiren.sqlite(path) - Open a database
fn jsSqliteOpen(
    ctx: ?*c.JSContext,
    _: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();
    const allocator = std.heap.page_allocator;

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "sqlite() requires a path argument");
    }

    const path_val = argv[0];
    const path_str = c.JS_ToCString(ctx, path_val);
    if (path_str == null) {
        return c.JS_ThrowTypeError(ctx, "Path must be a string");
    }
    defer c.JS_FreeCString(ctx, path_str);

    const path = std.mem.span(path_str);

    const db = Database.open(allocator, path) catch {
        return c.JS_ThrowInternalError(ctx, "Failed to open database");
    };

    // Create JS object
    const obj = c.JS_NewObjectClass(context, @intCast(db_class_id));
    c.JS_SetOpaque(obj, db);

    return obj;
}

// db.exec(sql) - Execute SQL statement
fn jsDbExec(
    ctx: ?*c.JSContext,
    this: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "exec() requires a SQL argument");
    }

    const ptr = c.JS_GetOpaque(this, db_class_id);
    if (ptr == null) {
        return c.JS_ThrowTypeError(ctx, "Invalid database object");
    }
    const db: *Database = @ptrCast(@alignCast(ptr));

    const sql_str = c.JS_ToCString(ctx, argv[0]);
    if (sql_str == null) {
        return c.JS_ThrowTypeError(ctx, "SQL must be a string");
    }
    defer c.JS_FreeCString(ctx, sql_str);

    db.exec(std.mem.span(sql_str)) catch {
        const err_msg = db.getError();
        var buf: [256]u8 = undefined;
        const msg = std.fmt.bufPrint(&buf, "SQL error: {s}", .{err_msg}) catch "SQL error";
        return c.JS_ThrowInternalError(ctx, @ptrCast(msg.ptr));
    };

    return engine.makeUndefined();
}

// db.run(sql, params) - Execute with parameters, return changes
fn jsDbRun(
    ctx: ?*c.JSContext,
    this: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();
    const allocator = std.heap.page_allocator;

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "run() requires a SQL argument");
    }

    const ptr = c.JS_GetOpaque(this, db_class_id);
    if (ptr == null) {
        return c.JS_ThrowTypeError(ctx, "Invalid database object");
    }
    const db: *Database = @ptrCast(@alignCast(ptr));

    const sql_str = c.JS_ToCString(ctx, argv[0]);
    if (sql_str == null) {
        return c.JS_ThrowTypeError(ctx, "SQL must be a string");
    }
    defer c.JS_FreeCString(ctx, sql_str);

    // Prepare statement
    const stmt = Statement.prepare(allocator, db, std.mem.span(sql_str)) catch {
        const err_msg = db.getError();
        var buf: [256]u8 = undefined;
        const msg = std.fmt.bufPrint(&buf, "Prepare error: {s}", .{err_msg}) catch "Prepare error";
        return c.JS_ThrowInternalError(ctx, @ptrCast(msg.ptr));
    };
    defer stmt.finalize();

    // Bind parameters if provided
    if (argc >= 2 and c.JS_IsArray(ctx, argv[1]) != 0) {
        const params = argv[1];
        const len_val = c.JS_GetPropertyStr(ctx, params, "length");
        var len: i32 = 0;
        _ = c.JS_ToInt32(ctx, &len, len_val);
        c.JS_FreeValue(ctx, len_val);

        var i: u32 = 0;
        while (i < @as(u32, @intCast(len))) : (i += 1) {
            const val = c.JS_GetPropertyUint32(ctx, params, i);
            defer c.JS_FreeValue(ctx, val);
            stmt.bindValue(context, @intCast(i + 1), val) catch {};
        }
    }

    // Execute
    _ = stmt.step() catch {
        const err_msg = db.getError();
        var buf: [256]u8 = undefined;
        const msg = std.fmt.bufPrint(&buf, "Execute error: {s}", .{err_msg}) catch "Execute error";
        return c.JS_ThrowInternalError(ctx, @ptrCast(msg.ptr));
    };

    // Return changes and lastInsertRowId
    const result = c.JS_NewObject(context);
    const changes = sqlite.sqlite3_changes(db.db);
    const last_id = sqlite.sqlite3_last_insert_rowid(db.db);

    _ = c.JS_SetPropertyStr(ctx, result, "changes", c.JS_NewInt32(ctx, changes));
    _ = c.JS_SetPropertyStr(ctx, result, "lastInsertRowid", c.JS_NewInt64(ctx, last_id));

    return result;
}

// db.query(sql, params) - Query and return all rows
fn jsDbQuery(
    ctx: ?*c.JSContext,
    this: c.JSValue,
    argc: c_int,
    argv: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const context = ctx orelse return engine.makeUndefined();
    const allocator = std.heap.page_allocator;

    if (argc < 1) {
        return c.JS_ThrowTypeError(ctx, "query() requires a SQL argument");
    }

    const ptr = c.JS_GetOpaque(this, db_class_id);
    if (ptr == null) {
        return c.JS_ThrowTypeError(ctx, "Invalid database object");
    }
    const db: *Database = @ptrCast(@alignCast(ptr));

    const sql_str = c.JS_ToCString(ctx, argv[0]);
    if (sql_str == null) {
        return c.JS_ThrowTypeError(ctx, "SQL must be a string");
    }
    defer c.JS_FreeCString(ctx, sql_str);

    // Prepare statement
    const stmt = Statement.prepare(allocator, db, std.mem.span(sql_str)) catch {
        const err_msg = db.getError();
        var buf: [256]u8 = undefined;
        const msg = std.fmt.bufPrint(&buf, "Prepare error: {s}", .{err_msg}) catch "Prepare error";
        return c.JS_ThrowInternalError(ctx, @ptrCast(msg.ptr));
    };
    defer stmt.finalize();

    // Bind parameters if provided
    if (argc >= 2 and c.JS_IsArray(ctx, argv[1]) != 0) {
        const params = argv[1];
        const len_val = c.JS_GetPropertyStr(ctx, params, "length");
        var len: i32 = 0;
        _ = c.JS_ToInt32(ctx, &len, len_val);
        c.JS_FreeValue(ctx, len_val);

        var i: u32 = 0;
        while (i < @as(u32, @intCast(len))) : (i += 1) {
            const val = c.JS_GetPropertyUint32(ctx, params, i);
            defer c.JS_FreeValue(ctx, val);
            stmt.bindValue(context, @intCast(i + 1), val) catch {};
        }
    }

    // Get column count and names
    const col_count = stmt.columnCount();

    // Create result array
    const result = c.JS_NewArray(context);
    var row_idx: u32 = 0;

    // Fetch rows
    while (stmt.step() catch false) {
        const row = c.JS_NewObject(context);

        var col_idx: c_int = 0;
        while (col_idx < col_count) : (col_idx += 1) {
            const col_name = stmt.columnName(col_idx) orelse continue;
            const col_value = stmt.columnValue(context, col_idx);
            _ = c.JS_SetPropertyStr(ctx, row, col_name, col_value);
        }

        _ = c.JS_SetPropertyUint32(ctx, result, row_idx, row);
        row_idx += 1;
    }

    return result;
}

// db.close() - Close the database
fn jsDbClose(
    ctx: ?*c.JSContext,
    this: c.JSValue,
    _: c_int,
    _: [*c]c.JSValue,
) callconv(.c) c.JSValue {
    const ptr = c.JS_GetOpaque(this, db_class_id);
    if (ptr == null) {
        return c.JS_ThrowTypeError(ctx, "Invalid database object");
    }
    const db: *Database = @ptrCast(@alignCast(ptr));

    if (db.db) |d| {
        _ = sqlite.sqlite3_close(d);
        db.db = null;
    }

    return engine.makeUndefined();
}

// Register SQLite API
pub fn register(eng: *engine.Engine) void {
    const ctx = eng.context;
    const rt = eng.runtime;

    // Create Database class
    db_class_id = c.JS_NewClassID(&db_class_id);
    const db_class_def = c.JSClassDef{
        .class_name = "Database",
        .finalizer = dbFinalizer,
        .gc_mark = null,
        .call = null,
        .exotic = null,
    };
    _ = c.JS_NewClass(rt, db_class_id, &db_class_def);

    // Create Database prototype
    const db_proto = c.JS_NewObject(ctx);
    _ = c.JS_SetPropertyStr(ctx, db_proto, "exec", c.JS_NewCFunction(ctx, jsDbExec, "exec", 1));
    _ = c.JS_SetPropertyStr(ctx, db_proto, "run", c.JS_NewCFunction(ctx, jsDbRun, "run", 2));
    _ = c.JS_SetPropertyStr(ctx, db_proto, "query", c.JS_NewCFunction(ctx, jsDbQuery, "query", 2));
    _ = c.JS_SetPropertyStr(ctx, db_proto, "close", c.JS_NewCFunction(ctx, jsDbClose, "close", 0));
    c.JS_SetClassProto(ctx, db_class_id, db_proto);

    // Create Statement class
    stmt_class_id = c.JS_NewClassID(&stmt_class_id);
    const stmt_class_def = c.JSClassDef{
        .class_name = "Statement",
        .finalizer = stmtFinalizer,
        .gc_mark = null,
        .call = null,
        .exotic = null,
    };
    _ = c.JS_NewClass(rt, stmt_class_id, &stmt_class_def);

    // Add Kiren.sqlite() function
    const global = eng.getGlobalObject();
    defer eng.freeValue(global);

    const kiren = c.JS_GetPropertyStr(ctx, global, "Kiren");
    if (c.JS_IsObject(kiren) != 0) {
        _ = c.JS_SetPropertyStr(ctx, kiren, "sqlite", c.JS_NewCFunction(ctx, jsSqliteOpen, "sqlite", 1));
    }
    c.JS_FreeValue(ctx, kiren);
}
