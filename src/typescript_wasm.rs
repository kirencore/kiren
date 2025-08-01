// WASM-compatible TypeScript transpiler using only regex
// This is a simplified version without SWC dependencies

use anyhow::{anyhow, Result};
use regex::Regex;

/// WASM-compatible TypeScript transpiler using regex only
pub struct WasmTypeScriptTranspiler {
    type_annotations: Regex,
    interfaces: Regex,
    import_types: Regex,
    export_types: Regex,
    decorators: Regex,
    generic_types: Regex,
    access_modifiers: Regex,
    enum_declarations: Regex,
}

impl WasmTypeScriptTranspiler {
    pub fn new() -> Self {
        Self {
            // Remove type annotations: : Type (but not in comments)
            type_annotations: Regex::new(
                r"(?m)^(\s*)([^/\n]*?):\s*[A-Za-z_][A-Za-z0-9_\[\]<>|\s]*(\s*[,=){};\n])",
            )
            .unwrap(),

            // Remove interface declarations
            interfaces: Regex::new(r"(?m)^interface\s+[A-Za-z_][A-Za-z0-9_]*\s*\{[^}]*\}\s*")
                .unwrap(),

            // Remove type-only imports
            import_types: Regex::new(
                r"import\s+type\s+\{[^}]*\}\s+from\s+['\x22][^'\x22]*['\x22]\s*;?",
            )
            .unwrap(),

            // Remove type exports
            export_types: Regex::new(r"export\s+type\s+\{[^}]*\}\s*;?").unwrap(),

            // Remove decorators (but not in comments)
            decorators: Regex::new(r"(?m)^(\s*)@[A-Za-z_][A-Za-z0-9_]*(\([^)]*\))?\s*").unwrap(),

            // Remove generic type parameters
            generic_types: Regex::new(r"<[A-Za-z_][A-Za-z0-9_,\s<>]*>(\s*[\(\{])").unwrap(),

            // Remove access modifiers
            access_modifiers: Regex::new(r"\b(public|private|protected|readonly)\s+").unwrap(),

            // Remove enum declarations (convert to object)
            enum_declarations: Regex::new(r"(?m)^enum\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{([^}]*)\}")
                .unwrap(),
        }
    }

    /// Transpile TypeScript code to JavaScript using regex
    pub fn transpile(&self, typescript_code: &str) -> Result<String> {
        let mut js_code = typescript_code.to_string();

        // Remove type-only imports first
        js_code = self.import_types.replace_all(&js_code, "").to_string();

        // Remove type exports
        js_code = self.export_types.replace_all(&js_code, "").to_string();

        // Remove interface declarations
        js_code = self.interfaces.replace_all(&js_code, "").to_string();

        // Convert enums to objects
        js_code = self.convert_enums(&js_code);

        // Remove decorators
        js_code = self.decorators.replace_all(&js_code, "").to_string();

        // Remove access modifiers
        js_code = self.access_modifiers.replace_all(&js_code, "").to_string();

        // Remove generic type parameters
        js_code = self.generic_types.replace_all(&js_code, "$1").to_string();

        // Remove type annotations (this should be last to avoid breaking other patterns)
        js_code = self
            .type_annotations
            .replace_all(&js_code, "$1$2$3")
            .to_string();

        // Clean up multiple whitespace and empty lines
        js_code = self.cleanup_whitespace(&js_code);

        Ok(js_code)
    }

    /// Convert TypeScript enums to JavaScript objects
    fn convert_enums(&self, code: &str) -> String {
        self.enum_declarations
            .replace_all(code, |caps: &regex::Captures| {
                let enum_name = &caps[1];
                let enum_body = &caps[2];

                // Parse enum members
                let members: Vec<&str> = enum_body.split(',').collect();
                let mut js_members = Vec::new();

                for (index, member) in members.iter().enumerate() {
                    let member = member.trim();
                    if member.is_empty() {
                        continue;
                    }

                    if member.contains('=') {
                        // Member has explicit value
                        let parts: Vec<&str> = member.split('=').collect();
                        if parts.len() == 2 {
                            let key = parts[0].trim();
                            let value = parts[1].trim();
                            js_members.push(format!("  {}: {}", key, value));
                        }
                    } else {
                        // Auto-increment numeric value
                        js_members.push(format!("  {}: {}", member, index));
                    }
                }

                format!("const {} = {{\n{}\n}};", enum_name, js_members.join(",\n"))
            })
            .to_string()
    }

    /// Clean up excessive whitespace
    fn cleanup_whitespace(&self, code: &str) -> String {
        // Remove multiple consecutive empty lines
        let multiple_newlines = Regex::new(r"\n\s*\n\s*\n").unwrap();
        let cleaned = multiple_newlines.replace_all(code, "\n\n").to_string();

        // Remove trailing whitespace
        let trailing_whitespace = Regex::new(r"[ \t]+$").unwrap();
        trailing_whitespace.replace_all(&cleaned, "").to_string()
    }
}

/// Helper function to transpile TypeScript content for WASM
pub fn transpile_typescript_content(content: &str) -> Result<String> {
    let transpiler = WasmTypeScriptTranspiler::new();
    transpiler.transpile(content)
}