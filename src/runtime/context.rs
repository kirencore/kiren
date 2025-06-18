use anyhow::Result;
use v8;

pub struct ExecutionContext {
    isolate: v8::OwnedIsolate,
    global_context: Option<v8::Global<v8::Context>>,
}

impl ExecutionContext {
    pub fn new() -> Result<Self> {
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        
        let global_context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = v8::Context::new(scope);
            let global_ctx = v8::Global::new(scope, context);
            let _scope = &mut v8::ContextScope::new(scope, context);
            
            
            Some(global_ctx)
        };

        Ok(ExecutionContext {
            isolate,
            global_context,
        })
    }

    pub fn execute(&mut self, source: &str, filename: &str) -> Result<Option<String>> {
        let scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = self.global_context.as_ref().unwrap().open(scope);
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