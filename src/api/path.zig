const std = @import("std");
const engine = @import("../engine.zig");
const c = engine.c;
const builtin = @import("builtin");

// Path module - implemented in JavaScript for simplicity
const path_js =
    \\const path = {
    \\  sep: (function() {
    \\    return process.platform === 'win32' ? '\\' : '/';
    \\  })(),
    \\
    \\  delimiter: (function() {
    \\    return process.platform === 'win32' ? ';' : ':';
    \\  })(),
    \\
    \\  basename: function(filepath, ext) {
    \\    if (typeof filepath !== 'string') return '';
    \\    let base = filepath.split(/[/\\]/).filter(Boolean).pop() || '';
    \\    if (ext && base.endsWith(ext)) {
    \\      base = base.slice(0, -ext.length);
    \\    }
    \\    return base;
    \\  },
    \\
    \\  dirname: function(filepath) {
    \\    if (typeof filepath !== 'string') return '.';
    \\    const parts = filepath.split(/[/\\]/).filter(Boolean);
    \\    parts.pop();
    \\    if (parts.length === 0) {
    \\      return filepath.startsWith('/') ? '/' : '.';
    \\    }
    \\    const result = parts.join(this.sep);
    \\    return filepath.startsWith('/') ? '/' + result : result;
    \\  },
    \\
    \\  extname: function(filepath) {
    \\    if (typeof filepath !== 'string') return '';
    \\    const base = this.basename(filepath);
    \\    const dotIndex = base.lastIndexOf('.');
    \\    if (dotIndex <= 0) return '';
    \\    return base.slice(dotIndex);
    \\  },
    \\
    \\  join: function(...parts) {
    \\    const filtered = parts.filter(p => typeof p === 'string' && p.length > 0);
    \\    if (filtered.length === 0) return '.';
    \\    const joined = filtered.join(this.sep);
    \\    return this.normalize(joined);
    \\  },
    \\
    \\  normalize: function(filepath) {
    \\    if (typeof filepath !== 'string') return '.';
    \\    if (filepath.length === 0) return '.';
    \\
    \\    const isAbsolute = filepath.startsWith('/');
    \\    const parts = filepath.split(/[/\\]/).filter(Boolean);
    \\    const result = [];
    \\
    \\    for (const part of parts) {
    \\      if (part === '..') {
    \\        if (result.length > 0 && result[result.length - 1] !== '..') {
    \\          result.pop();
    \\        } else if (!isAbsolute) {
    \\          result.push('..');
    \\        }
    \\      } else if (part !== '.') {
    \\        result.push(part);
    \\      }
    \\    }
    \\
    \\    let normalized = result.join(this.sep);
    \\    if (isAbsolute) normalized = '/' + normalized;
    \\    return normalized || '.';
    \\  },
    \\
    \\  resolve: function(...parts) {
    \\    let resolved = '';
    \\
    \\    for (let i = parts.length - 1; i >= 0; i--) {
    \\      const part = parts[i];
    \\      if (typeof part !== 'string') continue;
    \\      if (part.length === 0) continue;
    \\
    \\      resolved = part + (resolved ? this.sep + resolved : '');
    \\
    \\      if (part.startsWith('/')) break;
    \\    }
    \\
    \\    if (!resolved.startsWith('/')) {
    \\      resolved = process.cwd() + this.sep + resolved;
    \\    }
    \\
    \\    return this.normalize(resolved);
    \\  },
    \\
    \\  isAbsolute: function(filepath) {
    \\    if (typeof filepath !== 'string') return false;
    \\    return filepath.startsWith('/');
    \\  },
    \\
    \\  relative: function(from, to) {
    \\    const fromParts = this.resolve(from).split(this.sep).filter(Boolean);
    \\    const toParts = this.resolve(to).split(this.sep).filter(Boolean);
    \\
    \\    let commonLength = 0;
    \\    for (let i = 0; i < Math.min(fromParts.length, toParts.length); i++) {
    \\      if (fromParts[i] === toParts[i]) {
    \\        commonLength++;
    \\      } else {
    \\        break;
    \\      }
    \\    }
    \\
    \\    const upCount = fromParts.length - commonLength;
    \\    const downParts = toParts.slice(commonLength);
    \\
    \\    const result = [];
    \\    for (let i = 0; i < upCount; i++) {
    \\      result.push('..');
    \\    }
    \\    result.push(...downParts);
    \\
    \\    return result.join(this.sep) || '.';
    \\  },
    \\
    \\  parse: function(filepath) {
    \\    if (typeof filepath !== 'string') {
    \\      return { root: '', dir: '', base: '', ext: '', name: '' };
    \\    }
    \\
    \\    const isAbs = this.isAbsolute(filepath);
    \\    const root = isAbs ? '/' : '';
    \\    const dir = this.dirname(filepath);
    \\    const base = this.basename(filepath);
    \\    const ext = this.extname(filepath);
    \\    const name = base.slice(0, base.length - ext.length);
    \\
    \\    return { root, dir, base, ext, name };
    \\  },
    \\
    \\  format: function(pathObject) {
    \\    if (!pathObject || typeof pathObject !== 'object') return '';
    \\
    \\    const dir = pathObject.dir || pathObject.root || '';
    \\    const base = pathObject.base ||
    \\      (pathObject.name || '') + (pathObject.ext || '');
    \\
    \\    if (!dir) return base;
    \\    if (dir === pathObject.root) return dir + base;
    \\    return dir + this.sep + base;
    \\  }
    \\};
    \\
    \\// Make path available globally and as a module
    \\globalThis.path = path;
;

pub fn register(eng: *engine.Engine) void {
    // Evaluate the path module JavaScript
    const result = c.JS_Eval(
        eng.context,
        path_js.ptr,
        path_js.len,
        "<path>",
        c.JS_EVAL_TYPE_GLOBAL,
    );

    if (c.JS_IsException(result) != 0) {
        eng.dumpError();
    }

    c.JS_FreeValue(eng.context, result);
}
