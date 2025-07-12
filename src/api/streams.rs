use v8;

pub fn initialize_streams_api(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) -> Result<(), anyhow::Error> {
    // Create stream module object
    let stream_obj = v8::Object::new(scope);
    
    // Create Readable constructor
    let readable_name = v8::String::new(scope, "Readable").unwrap();
    let readable_template = v8::FunctionTemplate::new(scope, readable_constructor);
    setup_readable_prototype(scope, &readable_template)?;
    let readable_fn = readable_template.get_function(scope).unwrap();
    stream_obj.set(scope, readable_name.into(), readable_fn.into());
    
    // Create Writable constructor
    let writable_name = v8::String::new(scope, "Writable").unwrap();
    let writable_template = v8::FunctionTemplate::new(scope, writable_constructor);
    setup_writable_prototype(scope, &writable_template)?;
    let writable_fn = writable_template.get_function(scope).unwrap();
    stream_obj.set(scope, writable_name.into(), writable_fn.into());
    
    // Create Transform constructor
    let transform_name = v8::String::new(scope, "Transform").unwrap();
    let transform_template = v8::FunctionTemplate::new(scope, transform_constructor);
    setup_transform_prototype(scope, &transform_template)?;
    let transform_fn = transform_template.get_function(scope).unwrap();
    stream_obj.set(scope, transform_name.into(), transform_fn.into());
    
    // Create PassThrough constructor
    let passthrough_name = v8::String::new(scope, "PassThrough").unwrap();
    let passthrough_template = v8::FunctionTemplate::new(scope, passthrough_constructor);
    setup_passthrough_prototype(scope, &passthrough_template)?;
    let passthrough_fn = passthrough_template.get_function(scope).unwrap();
    stream_obj.set(scope, passthrough_name.into(), passthrough_fn.into());
    
    // Set stream module in global scope
    let stream_module_name = v8::String::new(scope, "stream").unwrap();
    global.set(scope, stream_module_name.into(), stream_obj.into());
    
    Ok(())
}

fn setup_readable_prototype(scope: &mut v8::HandleScope, template: &v8::FunctionTemplate) -> Result<(), anyhow::Error> {
    let proto = template.prototype_template(scope);
    
    // Readable methods
    let read_name = v8::String::new(scope, "read").unwrap();
    let read_fn = v8::FunctionTemplate::new(scope, readable_read);
    proto.set(read_name.into(), read_fn.into());
    
    let push_name = v8::String::new(scope, "push").unwrap();
    let push_fn = v8::FunctionTemplate::new(scope, readable_push);
    proto.set(push_name.into(), push_fn.into());
    
    let unshift_name = v8::String::new(scope, "unshift").unwrap();
    let unshift_fn = v8::FunctionTemplate::new(scope, readable_unshift);
    proto.set(unshift_name.into(), unshift_fn.into());
    
    let resume_name = v8::String::new(scope, "resume").unwrap();
    let resume_fn = v8::FunctionTemplate::new(scope, readable_resume);
    proto.set(resume_name.into(), resume_fn.into());
    
    let pause_name = v8::String::new(scope, "pause").unwrap();
    let pause_fn = v8::FunctionTemplate::new(scope, readable_pause);
    proto.set(pause_name.into(), pause_fn.into());
    
    let pipe_name = v8::String::new(scope, "pipe").unwrap();
    let pipe_fn = v8::FunctionTemplate::new(scope, readable_pipe);
    proto.set(pipe_name.into(), pipe_fn.into());
    
    let unpipe_name = v8::String::new(scope, "unpipe").unwrap();
    let unpipe_fn = v8::FunctionTemplate::new(scope, readable_unpipe);
    proto.set(unpipe_name.into(), unpipe_fn.into());
    
    Ok(())
}

fn setup_writable_prototype(scope: &mut v8::HandleScope, template: &v8::FunctionTemplate) -> Result<(), anyhow::Error> {
    let proto = template.prototype_template(scope);
    
    // Writable methods
    let write_name = v8::String::new(scope, "write").unwrap();
    let write_fn = v8::FunctionTemplate::new(scope, writable_write);
    proto.set(write_name.into(), write_fn.into());
    
    let end_name = v8::String::new(scope, "end").unwrap();
    let end_fn = v8::FunctionTemplate::new(scope, writable_end);
    proto.set(end_name.into(), end_fn.into());
    
    let cork_name = v8::String::new(scope, "cork").unwrap();
    let cork_fn = v8::FunctionTemplate::new(scope, writable_cork);
    proto.set(cork_name.into(), cork_fn.into());
    
    let uncork_name = v8::String::new(scope, "uncork").unwrap();
    let uncork_fn = v8::FunctionTemplate::new(scope, writable_uncork);
    proto.set(uncork_name.into(), uncork_fn.into());
    
    Ok(())
}

fn setup_transform_prototype(scope: &mut v8::HandleScope, template: &v8::FunctionTemplate) -> Result<(), anyhow::Error> {
    let proto = template.prototype_template(scope);
    
    // Transform methods (inherits from both Readable and Writable)
    let transform_name = v8::String::new(scope, "_transform").unwrap();
    let transform_fn = v8::FunctionTemplate::new(scope, transform_transform);
    proto.set(transform_name.into(), transform_fn.into());
    
    let flush_name = v8::String::new(scope, "_flush").unwrap();
    let flush_fn = v8::FunctionTemplate::new(scope, transform_flush);
    proto.set(flush_name.into(), flush_fn.into());
    
    Ok(())
}

fn setup_passthrough_prototype(scope: &mut v8::HandleScope, template: &v8::FunctionTemplate) -> Result<(), anyhow::Error> {
    let proto = template.prototype_template(scope);
    
    // PassThrough inherits everything from Transform
    let transform_name = v8::String::new(scope, "_transform").unwrap();
    let transform_fn = v8::FunctionTemplate::new(scope, passthrough_transform);
    proto.set(transform_name.into(), transform_fn.into());
    
    Ok(())
}

fn readable_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    // Initialize readable state
    let readable_state = v8::Object::new(scope);
    
    // Set default options
    let flowing_key = v8::String::new(scope, "flowing").unwrap();
    let flowing_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, flowing_key.into(), flowing_val.into());
    
    let ended_key = v8::String::new(scope, "ended").unwrap();
    let ended_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, ended_key.into(), ended_val.into());
    
    let reading_key = v8::String::new(scope, "reading").unwrap();
    let reading_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, reading_key.into(), reading_val.into());
    
    let buffer_key = v8::String::new(scope, "buffer").unwrap();
    let buffer_val = v8::Array::new(scope, 0);
    readable_state.set(scope, buffer_key.into(), buffer_val.into());
    
    let high_water_mark_key = v8::String::new(scope, "highWaterMark").unwrap();
    let high_water_mark_val = v8::Number::new(scope, 16384.0); // 16KB default
    readable_state.set(scope, high_water_mark_key.into(), high_water_mark_val.into());
    
    // Store state on instance
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    this.set(scope, state_key.into(), readable_state.into());
    
    // Initialize EventEmitter functionality
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = v8::Object::new(scope);
    this.set(scope, events_key.into(), events_obj.into());
    
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = v8::Number::new(scope, 10.0);
    this.set(scope, max_listeners_key.into(), max_listeners_val.into());
    
    retval.set(this.into());
}

fn writable_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    // Initialize writable state
    let writable_state = v8::Object::new(scope);
    
    let ended_key = v8::String::new(scope, "ended").unwrap();
    let ended_val = v8::Boolean::new(scope, false);
    writable_state.set(scope, ended_key.into(), ended_val.into());
    
    let finished_key = v8::String::new(scope, "finished").unwrap();
    let finished_val = v8::Boolean::new(scope, false);
    writable_state.set(scope, finished_key.into(), finished_val.into());
    
    let corked_key = v8::String::new(scope, "corked").unwrap();
    let corked_val = v8::Number::new(scope, 0.0);
    writable_state.set(scope, corked_key.into(), corked_val.into());
    
    let high_water_mark_key = v8::String::new(scope, "highWaterMark").unwrap();
    let high_water_mark_val = v8::Number::new(scope, 16384.0);
    writable_state.set(scope, high_water_mark_key.into(), high_water_mark_val.into());
    
    // Store state on instance
    let state_key = v8::String::new(scope, "_writableState").unwrap();
    this.set(scope, state_key.into(), writable_state.into());
    
    // Initialize EventEmitter functionality
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = v8::Object::new(scope);
    this.set(scope, events_key.into(), events_obj.into());
    
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = v8::Number::new(scope, 10.0);
    this.set(scope, max_listeners_key.into(), max_listeners_val.into());
    
    retval.set(this.into());
}

fn transform_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    // Initialize readable state
    let readable_state = v8::Object::new(scope);
    let flowing_key = v8::String::new(scope, "flowing").unwrap();
    let flowing_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, flowing_key.into(), flowing_val.into());
    
    let ended_key = v8::String::new(scope, "ended").unwrap();
    let ended_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, ended_key.into(), ended_val.into());
    
    let buffer_key = v8::String::new(scope, "buffer").unwrap();
    let buffer_val = v8::Array::new(scope, 0);
    readable_state.set(scope, buffer_key.into(), buffer_val.into());
    
    let readable_state_key = v8::String::new(scope, "_readableState").unwrap();
    this.set(scope, readable_state_key.into(), readable_state.into());
    
    // Initialize writable state
    let writable_state = v8::Object::new(scope);
    let writable_ended_key = v8::String::new(scope, "ended").unwrap();
    let writable_ended_val = v8::Boolean::new(scope, false);
    writable_state.set(scope, writable_ended_key.into(), writable_ended_val.into());
    
    let finished_key = v8::String::new(scope, "finished").unwrap();
    let finished_val = v8::Boolean::new(scope, false);
    writable_state.set(scope, finished_key.into(), finished_val.into());
    
    let writable_state_key = v8::String::new(scope, "_writableState").unwrap();
    this.set(scope, writable_state_key.into(), writable_state.into());
    
    // Initialize EventEmitter functionality
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = v8::Object::new(scope);
    this.set(scope, events_key.into(), events_obj.into());
    
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = v8::Number::new(scope, 10.0);
    this.set(scope, max_listeners_key.into(), max_listeners_val.into());
    
    // Transform-specific state
    let transform_state = v8::Object::new(scope);
    let transform_state_key = v8::String::new(scope, "_transformState").unwrap();
    this.set(scope, transform_state_key.into(), transform_state.into());
    
    retval.set(this.into());
}

fn passthrough_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // PassThrough is just a Transform - copy the same logic
    let this = args.this();
    
    // Initialize readable state
    let readable_state = v8::Object::new(scope);
    let flowing_key = v8::String::new(scope, "flowing").unwrap();
    let flowing_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, flowing_key.into(), flowing_val.into());
    
    let ended_key = v8::String::new(scope, "ended").unwrap();
    let ended_val = v8::Boolean::new(scope, false);
    readable_state.set(scope, ended_key.into(), ended_val.into());
    
    let buffer_key = v8::String::new(scope, "buffer").unwrap();
    let buffer_val = v8::Array::new(scope, 0);
    readable_state.set(scope, buffer_key.into(), buffer_val.into());
    
    let readable_state_key = v8::String::new(scope, "_readableState").unwrap();
    this.set(scope, readable_state_key.into(), readable_state.into());
    
    // Initialize writable state
    let writable_state = v8::Object::new(scope);
    let writable_ended_key = v8::String::new(scope, "ended").unwrap();
    let writable_ended_val = v8::Boolean::new(scope, false);
    writable_state.set(scope, writable_ended_key.into(), writable_ended_val.into());
    
    let finished_key = v8::String::new(scope, "finished").unwrap();
    let finished_val = v8::Boolean::new(scope, false);
    writable_state.set(scope, finished_key.into(), finished_val.into());
    
    let writable_state_key = v8::String::new(scope, "_writableState").unwrap();
    this.set(scope, writable_state_key.into(), writable_state.into());
    
    // Initialize EventEmitter functionality
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = v8::Object::new(scope);
    this.set(scope, events_key.into(), events_obj.into());
    
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = v8::Number::new(scope, 10.0);
    this.set(scope, max_listeners_key.into(), max_listeners_val.into());
    
    retval.set(this.into());
}

fn readable_read(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    let size = if args.length() > 0 {
        args.get(0).number_value(scope).unwrap_or(0.0) as usize
    } else {
        0
    };
    
    // Get readable state
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if !state.is_object() {
        retval.set(v8::null(scope).into());
        return;
    }
    
    let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
    
    // Get buffer
    let buffer_key = v8::String::new(scope, "buffer").unwrap();
    let buffer = state_obj.get(scope, buffer_key.into()).unwrap();
    
    if !buffer.is_array() {
        retval.set(v8::null(scope).into());
        return;
    }
    
    let buffer_array = unsafe { v8::Local::<v8::Array>::cast(buffer) };
    
    if buffer_array.length() == 0 {
        // No data available, try to read more
        let read_fn_key = v8::String::new(scope, "_read").unwrap();
        let read_fn = this.get(scope, read_fn_key.into()).unwrap();
        
        if read_fn.is_function() {
            let read_fn = unsafe { v8::Local::<v8::Function>::cast(read_fn) };
            let size_arg = v8::Number::new(scope, size as f64);
            read_fn.call(scope, this.into(), &[size_arg.into()]);
        }
        
        retval.set(v8::null(scope).into());
        return;
    }
    
    // Return first chunk from buffer
    let chunk = buffer_array.get_index(scope, 0).unwrap();
    
    // Remove from buffer (shift)
    let new_buffer = v8::Array::new(scope, (buffer_array.length() - 1) as i32);
    for i in 1..buffer_array.length() {
        let item = buffer_array.get_index(scope, i).unwrap();
        new_buffer.set_index(scope, i - 1, item);
    }
    state_obj.set(scope, buffer_key.into(), new_buffer.into());
    
    retval.set(chunk);
}

fn readable_push(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    if args.length() == 0 {
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }
    
    let chunk = args.get(0);
    
    // Check for null (end of stream)
    if chunk.is_null() {
        // Set ended state
        let state_key = v8::String::new(scope, "_readableState").unwrap();
        let state = this.get(scope, state_key.into()).unwrap();
        
        if state.is_object() {
            let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
            let ended_key = v8::String::new(scope, "ended").unwrap();
            let ended_val = v8::Boolean::new(scope, true);
            state_obj.set(scope, ended_key.into(), ended_val.into());
        }
        
        // Emit 'end' event
        emit_event(scope, this, "end", &[]);
        
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }
    
    // Add chunk to buffer
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let buffer_key = v8::String::new(scope, "buffer").unwrap();
        let buffer = state_obj.get(scope, buffer_key.into()).unwrap();
        
        if buffer.is_array() {
            let buffer_array = unsafe { v8::Local::<v8::Array>::cast(buffer) };
            let length = buffer_array.length();
            buffer_array.set_index(scope, length, chunk);
        }
    }
    
    // Emit 'readable' event
    emit_event(scope, this, "readable", &[]);
    
    retval.set(v8::Boolean::new(scope, true).into());
}

fn readable_unshift(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    if args.length() == 0 {
        retval.set(v8::undefined(scope).into());
        return;
    }
    
    let chunk = args.get(0);
    
    // Add chunk to beginning of buffer
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let buffer_key = v8::String::new(scope, "buffer").unwrap();
        let buffer = state_obj.get(scope, buffer_key.into()).unwrap();
        
        if buffer.is_array() {
            let buffer_array = unsafe { v8::Local::<v8::Array>::cast(buffer) };
            let length = buffer_array.length();
            
            // Create new buffer with chunk at beginning
            let new_buffer = v8::Array::new(scope, (length + 1) as i32);
            new_buffer.set_index(scope, 0, chunk);
            
            for i in 0..length {
                let item = buffer_array.get_index(scope, i).unwrap();
                new_buffer.set_index(scope, i + 1, item);
            }
            
            state_obj.set(scope, buffer_key.into(), new_buffer.into());
        }
    }
    
    retval.set(v8::undefined(scope).into());
}

fn readable_resume(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let flowing_key = v8::String::new(scope, "flowing").unwrap();
        let flowing_val = v8::Boolean::new(scope, true);
        state_obj.set(scope, flowing_key.into(), flowing_val.into());
    }
    
    emit_event(scope, this, "resume", &[]);
    
    retval.set(this.into());
}

fn readable_pause(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    let state_key = v8::String::new(scope, "_readableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let flowing_key = v8::String::new(scope, "flowing").unwrap();
        let flowing_val = v8::Boolean::new(scope, false);
        state_obj.set(scope, flowing_key.into(), flowing_val.into());
    }
    
    emit_event(scope, this, "pause", &[]);
    
    retval.set(this.into());
}

fn readable_pipe(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    if args.length() == 0 {
        let msg = v8::String::new(scope, "pipe() requires a destination").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }
    
    let destination = args.get(0);
    
    // Simple pipe implementation - emit data events to destination
    emit_event(scope, this, "pipe", &[destination]);
    
    retval.set(destination);
}

fn readable_unpipe(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    let destination = if args.length() > 0 {
        args.get(0)
    } else {
        v8::undefined(scope).into()
    };
    
    emit_event(scope, this, "unpipe", &[destination]);
    
    retval.set(this.into());
}

fn writable_write(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    if args.length() == 0 {
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }
    
    let chunk = args.get(0);
    let encoding = if args.length() > 1 {
        args.get(1)
    } else {
        v8::String::new(scope, "utf8").unwrap().into()
    };
    let callback = if args.length() > 2 {
        args.get(2)
    } else {
        v8::undefined(scope).into()
    };
    
    // Call _write method if it exists
    let write_fn_key = v8::String::new(scope, "_write").unwrap();
    let write_fn = this.get(scope, write_fn_key.into()).unwrap();
    
    if write_fn.is_function() {
        let write_fn = unsafe { v8::Local::<v8::Function>::cast(write_fn) };
        write_fn.call(scope, this.into(), &[chunk, encoding, callback]);
    }
    
    retval.set(v8::Boolean::new(scope, true).into());
}

fn writable_end(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    // Optional final chunk
    if args.length() > 0 && !args.get(0).is_function() {
        let chunk = args.get(0);
        let encoding = if args.length() > 1 && !args.get(1).is_function() {
            args.get(1)
        } else {
            v8::String::new(scope, "utf8").unwrap().into()
        };
        
        // Write final chunk
        let write_fn_key = v8::String::new(scope, "_write").unwrap();
        let write_fn = this.get(scope, write_fn_key.into()).unwrap();
        
        if write_fn.is_function() {
            let write_fn = unsafe { v8::Local::<v8::Function>::cast(write_fn) };
            let undefined = v8::undefined(scope);
            write_fn.call(scope, this.into(), &[chunk, encoding, undefined.into()]);
        }
    }
    
    // Set ended state
    let state_key = v8::String::new(scope, "_writableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let ended_key = v8::String::new(scope, "ended").unwrap();
        let ended_val = v8::Boolean::new(scope, true);
        state_obj.set(scope, ended_key.into(), ended_val.into());
        
        let finished_key = v8::String::new(scope, "finished").unwrap();
        let finished_val = v8::Boolean::new(scope, true);
        state_obj.set(scope, finished_key.into(), finished_val.into());
    }
    
    // Emit 'finish' event
    emit_event(scope, this, "finish", &[]);
    
    retval.set(this.into());
}

fn writable_cork(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    let state_key = v8::String::new(scope, "_writableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let corked_key = v8::String::new(scope, "corked").unwrap();
        let corked = state_obj.get(scope, corked_key.into()).unwrap();
        let corked_val = corked.number_value(scope).unwrap_or(0.0) + 1.0;
        let new_corked = v8::Number::new(scope, corked_val);
        state_obj.set(scope, corked_key.into(), new_corked.into());
    }
    
    retval.set(v8::undefined(scope).into());
}

fn writable_uncork(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();
    
    let state_key = v8::String::new(scope, "_writableState").unwrap();
    let state = this.get(scope, state_key.into()).unwrap();
    
    if state.is_object() {
        let state_obj = unsafe { v8::Local::<v8::Object>::cast(state) };
        let corked_key = v8::String::new(scope, "corked").unwrap();
        let corked = state_obj.get(scope, corked_key.into()).unwrap();
        let corked_val = (corked.number_value(scope).unwrap_or(0.0) - 1.0).max(0.0);
        let new_corked = v8::Number::new(scope, corked_val);
        state_obj.set(scope, corked_key.into(), new_corked.into());
    }
    
    retval.set(v8::undefined(scope).into());
}

fn transform_transform(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Default transform implementation - just push chunk through
    if args.length() >= 3 {
        let chunk = args.get(0);
        let _encoding = args.get(1);
        let callback = args.get(2);
        
        // Push chunk to readable side
        let this = args.this();
        let push_fn_key = v8::String::new(scope, "push").unwrap();
        let push_fn = this.get(scope, push_fn_key.into()).unwrap();
        
        if push_fn.is_function() {
            let push_fn = unsafe { v8::Local::<v8::Function>::cast(push_fn) };
            push_fn.call(scope, this.into(), &[chunk]);
        }
        
        // Call callback
        if callback.is_function() {
            let callback_fn = unsafe { v8::Local::<v8::Function>::cast(callback) };
            let undefined = v8::undefined(scope);
            callback_fn.call(scope, undefined.into(), &[]);
        }
    }
    
    retval.set(v8::undefined(scope).into());
}

fn transform_flush(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // Default flush implementation - just call callback
    if args.length() > 0 {
        let callback = args.get(0);
        if callback.is_function() {
            let callback_fn = unsafe { v8::Local::<v8::Function>::cast(callback) };
            let undefined = v8::undefined(scope);
            callback_fn.call(scope, undefined.into(), &[]);
        }
    }
    
    retval.set(v8::undefined(scope).into());
}

fn passthrough_transform(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    // PassThrough just passes data through unchanged
    transform_transform(scope, args, retval);
}

fn emit_event(scope: &mut v8::HandleScope, this: v8::Local<v8::Object>, event_name: &str, args: &[v8::Local<v8::Value>]) {
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();
    
    if !events_obj.is_object() {
        return;
    }
    
    let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };
    let event_key = v8::String::new(scope, event_name).unwrap();
    let listeners = events_obj.get(scope, event_key.into()).unwrap();
    
    if listeners.is_array() {
        let listeners_array = unsafe { v8::Local::<v8::Array>::cast(listeners) };
        
        for i in 0..listeners_array.length() {
            let listener = listeners_array.get_index(scope, i).unwrap();
            if listener.is_function() {
                let listener_fn = unsafe { v8::Local::<v8::Function>::cast(listener) };
                listener_fn.call(scope, this.into(), args);
            }
        }
    }
}