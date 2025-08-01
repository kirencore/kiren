#![allow(dead_code)]
use anyhow::{anyhow, Result};

#[cfg(test)]
mod tests;
use regex::Regex;
use std::fs;
use std::path::Path;
#[cfg(feature = "swc_common")]
use swc_common::{errors::Handler, sync::Lrc, SourceMap};
#[cfg(feature = "swc_ts_fast_strip")]
use swc_ts_fast_strip::{operate, Options};

/// TypeScript to JavaScript transpiler using SWC
pub struct TypeScriptTranspiler {
    #[cfg(feature = "swc_common")]
    source_map: Lrc<SourceMap>,
    // Keep regex patterns for fallback compatibility
    type_annotations: Regex,
    interfaces: Regex,
    import_types: Regex,
    export_types: Regex,
    decorators: Regex,
    generic_types: Regex,
    access_modifiers: Regex,
    enum_declarations: Regex,
}

impl TypeScriptTranspiler {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "swc_common")]
            source_map: Lrc::new(SourceMap::default()),
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

    /// Transpile TypeScript code to JavaScript using SWC
    pub fn transpile(&self, typescript_code: &str) -> Result<String> {
        // Try SWC first for better performance and accuracy
        match self.transpile_with_swc(typescript_code) {
            Ok(js_code) => Ok(js_code),
            Err(_) => {
                // Fall back to regex-based transpilation if SWC fails
                self.transpile_with_regex(typescript_code)
            }
        }
    }

    /// Transpile using SWC for better performance and accuracy
    #[cfg(all(feature = "swc_common", feature = "swc_ts_fast_strip"))]
    fn transpile_with_swc(&self, typescript_code: &str) -> Result<String> {
        let handler = Handler::with_emitter_writer(
            Box::new(std::io::stderr()),
            Some(self.source_map.clone()),
        );

        let options = Options::default();

        // Use the handler with HANDLER context
        swc_common::errors::HANDLER.set(&handler, || {
            match operate(
                &self.source_map,
                &handler,
                typescript_code.to_string(),
                options,
            ) {
                Ok(result) => Ok(result.code),
                Err(e) => Err(anyhow!("SWC transpilation failed: {:?}", e)),
            }
        })
    }

    /// Fallback when SWC is not available
    #[cfg(not(all(feature = "swc_common", feature = "swc_ts_fast_strip")))]
    fn transpile_with_swc(&self, _typescript_code: &str) -> Result<String> {
        Err(anyhow!("SWC not available, using regex fallback"))
    }

    /// Original regex-based transpilation as fallback
    fn transpile_with_regex(&self, typescript_code: &str) -> Result<String> {
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

    /// Load and transpile a TypeScript file
    pub fn transpile_file<P: AsRef<Path>>(&self, file_path: P) -> Result<String> {
        let path = file_path.as_ref();

        if !path.exists() {
            return Err(anyhow!("TypeScript file not found: {}", path.display()));
        }

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "ts" | "tsx" => {
                let typescript_content = fs::read_to_string(path)?;
                self.transpile(&typescript_content)
            }
            "js" | "jsx" => {
                // Already JavaScript, return as-is
                Ok(fs::read_to_string(path)?)
            }
            _ => Err(anyhow!("Unsupported file extension: {}", extension)),
        }
    }

    /// Check if a file is TypeScript
    pub fn is_typescript_file<P: AsRef<Path>>(file_path: P) -> bool {
        let path = file_path.as_ref();

        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return matches!(ext_str, "ts" | "tsx");
            }
        }

        false
    }

    /// Enhanced transpilation with NestJS decorator support
    pub fn transpile_nestjs(&self, typescript_code: &str) -> Result<String> {
        // Try SWC first, fall back to regex-based approach
        match self.transpile_with_swc(typescript_code) {
            Ok(js_code) => {
                // Post-process for NestJS decorators if needed
                Ok(self.convert_nestjs_decorators(&js_code))
            }
            Err(_) => {
                // Use regex-based approach with NestJS support
                self.transpile_nestjs_with_regex(typescript_code)
            }
        }
    }

    /// NestJS transpilation using regex (fallback)
    fn transpile_nestjs_with_regex(&self, typescript_code: &str) -> Result<String> {
        let mut js_code = typescript_code.to_string();

        // Convert common NestJS decorators to function calls FIRST
        js_code = self.convert_nestjs_decorators(&js_code);

        // Remove type-only imports first
        js_code = self.import_types.replace_all(&js_code, "").to_string();

        // Remove type exports
        js_code = self.export_types.replace_all(&js_code, "").to_string();

        // Remove interface declarations
        js_code = self.interfaces.replace_all(&js_code, "").to_string();

        // Convert enums to objects
        js_code = self.convert_enums(&js_code);

        // Remove remaining decorators (after NestJS conversion)
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

    /// Convert NestJS decorators to function calls (simplified)
    fn convert_nestjs_decorators(&self, code: &str) -> String {
        let mut js_code = code.to_string();

        // Convert @Controller() to a comment for now
        let controller_pattern = Regex::new(r"@Controller\s*\([^)]*\)\s*").unwrap();
        js_code = controller_pattern
            .replace_all(&js_code, "// @Controller\n")
            .to_string();

        // Convert @Get(), @Post(), etc.
        let method_patterns = Regex::new(r"@(Get|Post|Put|Delete|Patch)\s*\(\s*\)\s*").unwrap();
        js_code = method_patterns
            .replace_all(&js_code, "// @$1\n    ")
            .to_string();

        // Convert @Injectable()
        let injectable_pattern = Regex::new(r"@Injectable\s*\(\s*\)\s*").unwrap();
        js_code = injectable_pattern
            .replace_all(&js_code, "// @Injectable\n")
            .to_string();

        // Convert @Body(), @Query(), @Param()
        let param_patterns = Regex::new(r"@(Body|Query|Param|Headers)\s*\([^)]*\)\s*").unwrap();
        js_code = param_patterns
            .replace_all(&js_code, "/* @$1 */ ")
            .to_string();

        js_code
    }
}

/// Helper function to transpile TypeScript on the fly
pub fn transpile_typescript_content(content: &str) -> Result<String> {
    let transpiler = TypeScriptTranspiler::new();
    transpiler.transpile(content)
}

/// Helper function to check and transpile a file if it's TypeScript
pub fn transpile_file_if_typescript<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let transpiler = TypeScriptTranspiler::new();
    transpiler.transpile_file(file_path)
}
