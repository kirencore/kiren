use v8;

pub fn initialize_events_api(
    scope: &mut v8::HandleScope,
    global: v8::Local<v8::Object>,
) -> Result<(), anyhow::Error> {
    // Create EventEmitter constructor
    let eventemitter_name = v8::String::new(scope, "EventEmitter").unwrap();
    let eventemitter_template = v8::FunctionTemplate::new(scope, eventemitter_constructor);

    // Prototype methods
    let proto = eventemitter_template.prototype_template(scope);

    let on_name = v8::String::new(scope, "on").unwrap();
    let on_fn = v8::FunctionTemplate::new(scope, event_on);
    proto.set(on_name.into(), on_fn.into());

    let addlistener_name = v8::String::new(scope, "addListener").unwrap();
    let addlistener_fn = v8::FunctionTemplate::new(scope, event_on);
    proto.set(addlistener_name.into(), addlistener_fn.into());

    let once_name = v8::String::new(scope, "once").unwrap();
    let once_fn = v8::FunctionTemplate::new(scope, event_once);
    proto.set(once_name.into(), once_fn.into());

    let emit_name = v8::String::new(scope, "emit").unwrap();
    let emit_fn = v8::FunctionTemplate::new(scope, event_emit);
    proto.set(emit_name.into(), emit_fn.into());

    let off_name = v8::String::new(scope, "off").unwrap();
    let off_fn = v8::FunctionTemplate::new(scope, event_off);
    proto.set(off_name.into(), off_fn.into());

    let removelistener_name = v8::String::new(scope, "removeListener").unwrap();
    let removelistener_fn = v8::FunctionTemplate::new(scope, event_off);
    proto.set(removelistener_name.into(), removelistener_fn.into());

    let removealllisteners_name = v8::String::new(scope, "removeAllListeners").unwrap();
    let removealllisteners_fn = v8::FunctionTemplate::new(scope, event_remove_all_listeners);
    proto.set(removealllisteners_name.into(), removealllisteners_fn.into());

    let listeners_name = v8::String::new(scope, "listeners").unwrap();
    let listeners_fn = v8::FunctionTemplate::new(scope, event_listeners);
    proto.set(listeners_name.into(), listeners_fn.into());

    let listenercount_name = v8::String::new(scope, "listenerCount").unwrap();
    let listenercount_fn = v8::FunctionTemplate::new(scope, event_listener_count);
    proto.set(listenercount_name.into(), listenercount_fn.into());

    let eventnames_name = v8::String::new(scope, "eventNames").unwrap();
    let eventnames_fn = v8::FunctionTemplate::new(scope, event_names);
    proto.set(eventnames_name.into(), eventnames_fn.into());

    let setmaxlisteners_name = v8::String::new(scope, "setMaxListeners").unwrap();
    let setmaxlisteners_fn = v8::FunctionTemplate::new(scope, event_set_max_listeners);
    proto.set(setmaxlisteners_name.into(), setmaxlisteners_fn.into());

    let getmaxlisteners_name = v8::String::new(scope, "getMaxListeners").unwrap();
    let getmaxlisteners_fn = v8::FunctionTemplate::new(scope, event_get_max_listeners);
    proto.set(getmaxlisteners_name.into(), getmaxlisteners_fn.into());

    // Create constructor function
    let eventemitter_fn = eventemitter_template.get_function(scope).unwrap();
    global.set(scope, eventemitter_name.into(), eventemitter_fn.into());

    Ok(())
}

fn eventemitter_constructor(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();

    // Initialize internal listeners object
    let listeners_key = v8::String::new(scope, "_events").unwrap();
    let listeners_obj = v8::Object::new(scope);
    this.set(scope, listeners_key.into(), listeners_obj.into());

    // Initialize max listeners (default 10)
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = v8::Number::new(scope, 10.0);
    this.set(scope, max_listeners_key.into(), max_listeners_val.into());

    retval.set(this.into());
}

fn event_on(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 2 {
        let msg = v8::String::new(
            scope,
            "EventEmitter.on() requires an event name and listener",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let listener = args.get(1);

    if !listener.is_function() {
        let msg = v8::String::new(scope, "Listener must be a function").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    // Get or create events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();
    let events_obj = if events_obj.is_object() {
        unsafe { v8::Local::<v8::Object>::cast(events_obj) }
    } else {
        let new_events = v8::Object::new(scope);
        this.set(scope, events_key.into(), new_events.into());
        new_events
    };

    // Get or create listener array for this event
    let event_key = v8::String::new(scope, &event_name).unwrap();
    let listeners_val = events_obj.get(scope, event_key.into()).unwrap();

    let listeners_array = if listeners_val.is_array() {
        unsafe { v8::Local::<v8::Array>::cast(listeners_val) }
    } else {
        let new_array = v8::Array::new(scope, 0);
        events_obj.set(scope, event_key.into(), new_array.into());
        new_array
    };

    // Add listener to array
    let length = listeners_array.length();
    listeners_array.set_index(scope, length, listener);

    // Check max listeners
    check_max_listeners(scope, this, &event_name, length + 1);

    retval.set(this.into());
}

fn event_once(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 2 {
        let msg = v8::String::new(
            scope,
            "EventEmitter.once() requires an event name and listener",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let listener = args.get(1);

    if !listener.is_function() {
        let msg = v8::String::new(scope, "Listener must be a function").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    // Get or create events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();
    let events_obj = if events_obj.is_object() {
        unsafe { v8::Local::<v8::Object>::cast(events_obj) }
    } else {
        let new_events = v8::Object::new(scope);
        this.set(scope, events_key.into(), new_events.into());
        new_events
    };

    // Get or create listener array for this event
    let event_key = v8::String::new(scope, &event_name).unwrap();
    let listeners_val = events_obj.get(scope, event_key.into()).unwrap();

    let listeners_array = if listeners_val.is_array() {
        unsafe { v8::Local::<v8::Array>::cast(listeners_val) }
    } else {
        let new_array = v8::Array::new(scope, 0);
        events_obj.set(scope, event_key.into(), new_array.into());
        new_array
    };

    // Create a simple wrapper object to mark as "once"
    let wrapper_obj = v8::Object::new(scope);
    let listener_key = v8::String::new(scope, "listener").unwrap();
    let once_key = v8::String::new(scope, "once").unwrap();
    wrapper_obj.set(scope, listener_key.into(), listener);
    let once_val = v8::Boolean::new(scope, true);
    wrapper_obj.set(scope, once_key.into(), once_val.into());

    // Add wrapper to array
    let length = listeners_array.length();
    listeners_array.set_index(scope, length, wrapper_obj.into());

    // Check max listeners
    check_max_listeners(scope, this, &event_name, length + 1);

    retval.set(this.into());
}

fn event_emit(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 1 {
        let msg = v8::String::new(scope, "EventEmitter.emit() requires an event name").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    // Get events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();

    if !events_obj.is_object() {
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }

    let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };

    // Get listeners for this event
    let event_key = v8::String::new(scope, &event_name).unwrap();
    let listeners_val = events_obj.get(scope, event_key.into()).unwrap();

    if !listeners_val.is_array() {
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }

    let listeners_array = unsafe { v8::Local::<v8::Array>::cast(listeners_val) };
    let listener_count = listeners_array.length();

    if listener_count == 0 {
        retval.set(v8::Boolean::new(scope, false).into());
        return;
    }

    // Collect event arguments (skip event name)
    let mut event_args = Vec::new();
    for i in 1..args.length() {
        event_args.push(args.get(i));
    }

    // Track listeners to remove (for once listeners)
    let mut to_remove = Vec::new();

    // Call each listener
    for i in 0..listener_count {
        let listener_val = listeners_array.get_index(scope, i).unwrap();

        if listener_val.is_function() {
            // Regular function listener
            let listener_fn = unsafe { v8::Local::<v8::Function>::cast(listener_val) };
            let _undefined = v8::undefined(scope);
            listener_fn.call(scope, this.into(), &event_args);
        } else if listener_val.is_object() {
            // Check if it's a once wrapper
            let listener_obj = unsafe { v8::Local::<v8::Object>::cast(listener_val) };
            let listener_key = v8::String::new(scope, "listener").unwrap();
            let once_key = v8::String::new(scope, "once").unwrap();

            let actual_listener = listener_obj.get(scope, listener_key.into()).unwrap();
            let is_once = listener_obj.get(scope, once_key.into()).unwrap();

            if actual_listener.is_function() {
                let listener_fn = unsafe { v8::Local::<v8::Function>::cast(actual_listener) };
                let _undefined = v8::undefined(scope);
                listener_fn.call(scope, this.into(), &event_args);

                // Mark for removal if it's a once listener
                if is_once.is_boolean() && is_once.boolean_value(scope) {
                    to_remove.push(i);
                }
            }
        }
    }

    // Remove once listeners (only the first one to avoid index issues)
    if let Some(&index) = to_remove.iter().rev().next() {
        // Create new array without the removed listener
        let new_array = v8::Array::new(scope, (listener_count - 1) as i32);
        let mut new_index = 0;

        for j in 0..listener_count {
            if j != index {
                let listener = listeners_array.get_index(scope, j).unwrap();
                new_array.set_index(scope, new_index, listener);
                new_index += 1;
            }
        }

        events_obj.set(scope, event_key.into(), new_array.into());
    }

    retval.set(v8::Boolean::new(scope, true).into());
}

fn event_off(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 2 {
        let msg = v8::String::new(
            scope,
            "EventEmitter.off() requires an event name and listener",
        )
        .unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let listener_to_remove = args.get(1);

    // Get events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();

    if !events_obj.is_object() {
        retval.set(this.into());
        return;
    }

    let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };

    // Get listeners for this event
    let event_key = v8::String::new(scope, &event_name).unwrap();
    let listeners_val = events_obj.get(scope, event_key.into()).unwrap();

    if !listeners_val.is_array() {
        retval.set(this.into());
        return;
    }

    let listeners_array = unsafe { v8::Local::<v8::Array>::cast(listeners_val) };
    let length = listeners_array.length();

    // Find and remove the listener
    let new_array = v8::Array::new(scope, 0);
    let mut new_index = 0;

    for i in 0..length {
        let listener = listeners_array.get_index(scope, i).unwrap();

        // Check if this is the listener to remove
        let should_remove = if listener.strict_equals(listener_to_remove) {
            true
        } else if listener.is_object() {
            // Check for once() wrapper functions
            let listener_obj = unsafe { v8::Local::<v8::Object>::cast(listener) };
            let listener_key = v8::String::new(scope, "listener").unwrap();
            let wrapped_listener = listener_obj.get(scope, listener_key.into()).unwrap();
            wrapped_listener.strict_equals(listener_to_remove)
        } else {
            false
        };

        if !should_remove {
            new_array.set_index(scope, new_index, listener);
            new_index += 1;
        }
    }

    // Update the listeners array
    if new_index == 0 {
        // No listeners left, remove the event
        events_obj.delete(scope, event_key.into());
    } else {
        events_obj.set(scope, event_key.into(), new_array.into());
    }

    retval.set(this.into());
}

fn event_remove_all_listeners(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();

    if args.length() == 0 {
        // Remove all listeners for all events
        let events_key = v8::String::new(scope, "_events").unwrap();
        let new_events = v8::Object::new(scope);
        this.set(scope, events_key.into(), new_events.into());
    } else {
        // Remove all listeners for specific event
        let event_name = args
            .get(0)
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);

        let events_key = v8::String::new(scope, "_events").unwrap();
        let events_obj = this.get(scope, events_key.into()).unwrap();

        if events_obj.is_object() {
            let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };
            let event_key = v8::String::new(scope, &event_name).unwrap();
            events_obj.delete(scope, event_key.into());
        }
    }

    retval.set(this.into());
}

fn event_listeners(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 1 {
        let msg =
            v8::String::new(scope, "EventEmitter.listeners() requires an event name").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    // Get events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();

    if !events_obj.is_object() {
        let empty_array = v8::Array::new(scope, 0);
        retval.set(empty_array.into());
        return;
    }

    let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };

    // Get listeners for this event
    let event_key = v8::String::new(scope, &event_name).unwrap();
    let listeners_val = events_obj.get(scope, event_key.into()).unwrap();

    if listeners_val.is_array() {
        // Return unwrapped listeners
        let listeners_array = unsafe { v8::Local::<v8::Array>::cast(listeners_val) };
        let result_array = v8::Array::new(scope, listeners_array.length() as i32);

        for i in 0..listeners_array.length() {
            let listener = listeners_array.get_index(scope, i).unwrap();
            if listener.is_function() {
                result_array.set_index(scope, i, listener);
            } else if listener.is_object() {
                // Unwrap once listeners
                let listener_obj = unsafe { v8::Local::<v8::Object>::cast(listener) };
                let listener_key = v8::String::new(scope, "listener").unwrap();
                let actual_listener = listener_obj.get(scope, listener_key.into()).unwrap();
                result_array.set_index(scope, i, actual_listener);
            }
        }

        retval.set(result_array.into());
    } else {
        let empty_array = v8::Array::new(scope, 0);
        retval.set(empty_array.into());
    }
}

fn event_listener_count(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 1 {
        let msg =
            v8::String::new(scope, "EventEmitter.listenerCount() requires an event name").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let event_name = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    // Get events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();

    if !events_obj.is_object() {
        retval.set(v8::Number::new(scope, 0.0).into());
        return;
    }

    let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };

    // Get listeners for this event
    let event_key = v8::String::new(scope, &event_name).unwrap();
    let listeners_val = events_obj.get(scope, event_key.into()).unwrap();

    if listeners_val.is_array() {
        let listeners_array = unsafe { v8::Local::<v8::Array>::cast(listeners_val) };
        retval.set(v8::Number::new(scope, listeners_array.length() as f64).into());
    } else {
        retval.set(v8::Number::new(scope, 0.0).into());
    }
}

fn event_names(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();

    // Get events object
    let events_key = v8::String::new(scope, "_events").unwrap();
    let events_obj = this.get(scope, events_key.into()).unwrap();

    if !events_obj.is_object() {
        let empty_array = v8::Array::new(scope, 0);
        retval.set(empty_array.into());
        return;
    }

    let events_obj = unsafe { v8::Local::<v8::Object>::cast(events_obj) };

    // Get property names
    let property_names = events_obj
        .get_own_property_names(scope, Default::default())
        .unwrap();
    retval.set(property_names.into());
}

fn event_set_max_listeners(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    if args.length() < 1 {
        let msg =
            v8::String::new(scope, "EventEmitter.setMaxListeners() requires a number").unwrap();
        let exception = v8::Exception::type_error(scope, msg);
        scope.throw_exception(exception);
        return;
    }

    let this = args.this();
    let max_listeners = args.get(0).number_value(scope).unwrap_or(10.0);

    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = v8::Number::new(scope, max_listeners);
    this.set(scope, max_listeners_key.into(), max_listeners_val.into());

    retval.set(this.into());
}

fn event_get_max_listeners(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let this = args.this();

    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = this.get(scope, max_listeners_key.into()).unwrap();

    if max_listeners_val.is_number() {
        retval.set(max_listeners_val);
    } else {
        retval.set(v8::Number::new(scope, 10.0).into());
    }
}

fn check_max_listeners(
    scope: &mut v8::HandleScope,
    this: v8::Local<v8::Object>,
    event_name: &str,
    count: u32,
) {
    let max_listeners_key = v8::String::new(scope, "_maxListeners").unwrap();
    let max_listeners_val = this.get(scope, max_listeners_key.into()).unwrap();
    let max_listeners = max_listeners_val.number_value(scope).unwrap_or(10.0) as u32;

    if count > max_listeners && max_listeners > 0 {
        let warning = format!(
            "MaxListenersExceededWarning: Possible EventEmitter memory leak detected. {} {} listeners added. Use emitter.setMaxListeners() to increase limit",
            count, event_name
        );

        // Print warning to console
        println!("(node:warning) {}", warning);
    }
}
