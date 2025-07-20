use anyhow::Result;
use v8;

pub struct StackTrace {
    pub message: String,
    pub stack: String,
    pub frames: Vec<StackFrame>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub file_name: String,
    pub line_number: i32,
    pub column_number: i32,
    pub source_line: Option<String>,
}

pub fn setup_error_handling(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    // Enhanced Error constructor
    let error_key = v8::String::new(scope, "Error").unwrap();
    let error_template = v8::FunctionTemplate::new(scope, enhanced_error_constructor);
    let error_function = error_template.get_function(scope).unwrap();
    global.set(scope, error_key.into(), error_function.into());

    // Global error handler
    let error_handler_key = v8::String::new(scope, "__kirenErrorHandler").unwrap();
    let error_handler_template = v8::FunctionTemplate::new(scope, global_error_handler);
    let error_handler_function = error_handler_template.get_function(scope).unwrap();
    global.set(scope, error_handler_key.into(), error_handler_function.into());

    // Unhandled promise rejection handler
    let promise_handler_key = v8::String::new(scope, "__kirenPromiseRejectionHandler").unwrap();
    let promise_handler_template = v8::FunctionTemplate::new(scope, promise_rejection_handler);
    let promise_handler_function = promise_handler_template.get_function(scope).unwrap();
    global.set(scope, promise_handler_key.into(), promise_handler_function.into());

    Ok(())
}

fn enhanced_error_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let message = if args.length() > 0 {
        let arg = args.get(0);
        let str_val = arg.to_string(scope).unwrap();
        str_val.to_rust_string_lossy(scope)
    } else {
        "Error".to_string()
    };

    // Create error object
    let error_obj = v8::Object::new(scope);
    
    // Set message
    let message_key = v8::String::new(scope, "message").unwrap();
    let message_val = v8::String::new(scope, &message).unwrap();
    error_obj.set(scope, message_key.into(), message_val.into());

    // Set name
    let name_key = v8::String::new(scope, "name").unwrap();
    let name_val = v8::String::new(scope, "Error").unwrap();
    error_obj.set(scope, name_key.into(), name_val.into());

    // Generate stack trace
    let stack = generate_stack_trace(scope);
    let stack_key = v8::String::new(scope, "stack").unwrap();
    let stack_val = v8::String::new(scope, &stack).unwrap();
    error_obj.set(scope, stack_key.into(), stack_val.into());

    rv.set(error_obj.into());
}

fn global_error_handler(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }

    let error = args.get(0);
    let error_info = extract_error_info(scope, error);
    
    eprintln!("Uncaught Error: {}", error_info.message);
    eprintln!("{}", error_info.stack);
    
    // Exit with error code
    std::process::exit(1);
}

fn promise_rejection_handler(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        return;
    }

    let reason = args.get(0);
    let error_info = extract_error_info(scope, reason);
    
    eprintln!("Unhandled Promise Rejection: {}", error_info.message);
    eprintln!("{}", error_info.stack);
}

fn generate_stack_trace(scope: &mut v8::HandleScope) -> String {
    // Get V8 stack trace
    let stack_trace = v8::StackTrace::current_stack_trace(scope, 10);
    
    if let Some(trace) = stack_trace {
        let mut stack_lines = Vec::new();
        
        for i in 0..trace.get_frame_count() {
            if let Some(frame) = trace.get_frame(scope, i) {
                let function_name = frame.get_function_name(scope)
                    .map(|name| name.to_rust_string_lossy(scope))
                    .unwrap_or_else(|| "anonymous".to_string());
                
                let script_name = frame.get_script_name(scope)
                    .map(|name| name.to_rust_string_lossy(scope))
                    .unwrap_or_else(|| "<eval>".to_string());
                
                let line_number = frame.get_line_number();
                let column_number = frame.get_column();
                
                stack_lines.push(format!(
                    "    at {} ({}:{}:{})", 
                    function_name, 
                    script_name, 
                    line_number, 
                    column_number
                ));
            }
        }
        
        if stack_lines.is_empty() {
            "    at <anonymous>".to_string()
        } else {
            stack_lines.join("\n")
        }
    } else {
        "    at <anonymous>".to_string()
    }
}

fn extract_error_info(scope: &mut v8::HandleScope, error: v8::Local<v8::Value>) -> StackTrace {
    let mut message = "Unknown error".to_string();
    let mut stack = String::new();
    let mut frames = Vec::new();

    if error.is_object() {
        let error_obj = error.to_object(scope).unwrap();
        
        // Extract message
        let message_key = v8::String::new(scope, "message").unwrap();
        if let Some(message_val) = error_obj.get(scope, message_key.into()) {
            if let Some(message_str) = message_val.to_string(scope) {
                message = message_str.to_rust_string_lossy(scope);
            }
        }
        
        // Extract stack
        let stack_key = v8::String::new(scope, "stack").unwrap();
        if let Some(stack_val) = error_obj.get(scope, stack_key.into()) {
            if let Some(stack_str) = stack_val.to_string(scope) {
                stack = stack_str.to_rust_string_lossy(scope);
                frames = parse_stack_frames(&stack);
            }
        }
    } else if error.is_string() {
        let error_str = error.to_string(scope).unwrap();
        message = error_str.to_rust_string_lossy(scope);
        stack = generate_stack_trace(scope);
        frames = parse_stack_frames(&stack);
    }

    StackTrace {
        message,
        stack,
        frames,
    }
}

fn parse_stack_frames(stack: &str) -> Vec<StackFrame> {
    let mut frames = Vec::new();
    
    for line in stack.lines() {
        if line.trim().starts_with("at ") {
            let trimmed = line.trim_start_matches("at ").trim();
            
            // Parse different stack trace formats
            if let Some(frame) = parse_stack_frame(trimmed) {
                frames.push(frame);
            }
        }
    }
    
    frames
}

fn parse_stack_frame(frame_str: &str) -> Option<StackFrame> {
    // Handle format: "functionName (file:line:column)"
    if let Some(paren_pos) = frame_str.rfind('(') {
        let function_name = frame_str[..paren_pos].trim().to_string();
        let location_part = &frame_str[paren_pos + 1..];
        
        if let Some(closing_paren) = location_part.rfind(')') {
            let location = &location_part[..closing_paren];
            return parse_location(&function_name, location);
        }
    }
    
    // Handle format: "file:line:column"
    if frame_str.contains(':') {
        return parse_location("anonymous", frame_str);
    }
    
    // Fallback
    Some(StackFrame {
        function_name: frame_str.to_string(),
        file_name: "<unknown>".to_string(),
        line_number: 0,
        column_number: 0,
        source_line: None,
    })
}

fn parse_location(function_name: &str, location: &str) -> Option<StackFrame> {
    let parts: Vec<&str> = location.rsplitn(3, ':').collect();
    
    if parts.len() >= 3 {
        let column = parts[0].parse::<i32>().unwrap_or(0);
        let line = parts[1].parse::<i32>().unwrap_or(0);
        let file = parts[2..].join(":").to_string();
        
        Some(StackFrame {
            function_name: function_name.to_string(),
            file_name: file,
            line_number: line,
            column_number: column,
            source_line: None,
        })
    } else if parts.len() >= 2 {
        let line = parts[0].parse::<i32>().unwrap_or(0);
        let file = parts[1..].join(":").to_string();
        
        Some(StackFrame {
            function_name: function_name.to_string(),
            file_name: file,
            line_number: line,
            column_number: 0,
            source_line: None,
        })
    } else {
        Some(StackFrame {
            function_name: function_name.to_string(),
            file_name: location.to_string(),
            line_number: 0,
            column_number: 0,
            source_line: None,
        })
    }
}

pub fn format_error_for_display(error_info: &StackTrace) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("Error: {}\n", error_info.message));
    
    if !error_info.frames.is_empty() {
        output.push_str("Stack trace:\n");
        for (i, frame) in error_info.frames.iter().enumerate() {
            if i < 10 { // Limit stack trace depth
                output.push_str(&format!(
                    "  at {} ({}:{}:{})\n",
                    frame.function_name,
                    frame.file_name,
                    frame.line_number,
                    frame.column_number
                ));
            }
        }
    }
    
    output
}