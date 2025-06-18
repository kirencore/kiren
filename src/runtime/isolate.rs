use anyhow::Result;
use v8;

pub struct Isolate {
    isolate: v8::OwnedIsolate,
}

impl Isolate {
    pub fn new() -> Result<Self> {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        
        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
        
        Ok(Isolate { isolate })
    }

    pub fn execute_script(&mut self, source: &str, filename: &str) -> Result<Option<String>> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(scope);
        let scope = &mut v8::ContextScope::new(scope, context);

        let source = v8::String::new(scope, source).unwrap();
        let filename = v8::String::new(scope, filename).unwrap();
        let undefined_val = v8::undefined(scope);
        let origin = v8::ScriptOrigin::new(
            scope,
            filename.into(),
            0,
            0,
            false,
            0,
            undefined_val.into(),
            false,
            false,
            false,
        );

        let script = match v8::Script::compile(scope, source, Some(&origin)) {
            Some(script) => script,
            None => {
                return Err(anyhow::anyhow!("Compilation error"));
            }
        };

        match script.run(scope) {
            Some(result) => {
                let result_string = result.to_string(scope).unwrap();
                Ok(Some(result_string.to_rust_string_lossy(scope)))
            }
            None => {
                Err(anyhow::anyhow!("Runtime error"))
            }
        }
    }
}