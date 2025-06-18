use anyhow::Result;
use reqwest;
use v8;

pub fn setup_fetch(scope: &mut v8::HandleScope, context: v8::Local<v8::Context>) -> Result<()> {
    let global = context.global(scope);

    let fetch_key = v8::String::new(scope, "fetch").unwrap();
    let fetch_tmpl = v8::FunctionTemplate::new(scope, fetch);
    let fetch_fn = fetch_tmpl.get_function(scope).unwrap();
    global.set(scope, fetch_key.into(), fetch_fn.into());

    Ok(())
}

fn fetch(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    if args.length() < 1 {
        let error = v8::String::new(scope, "fetch requires at least 1 argument").unwrap();
        let exception = v8::Exception::type_error(scope, error);
        scope.throw_exception(exception);
        return;
    }

    let url_arg = args.get(0);
    let url_str = url_arg.to_string(scope).unwrap();
    let url = url_str.to_rust_string_lossy(scope);

    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);
    rv.set(promise.into());

    let _resolver_global = v8::Global::new(scope, resolver);

    tokio::spawn(async move {
        match make_request(&url).await {
            Ok(response) => {
                println!("Fetch success: {}", response);
            }
            Err(e) => {
                println!("Fetch error: {}", e);
            }
        }
    });
}

async fn make_request(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let text = response.text().await?;
    Ok(text)
}
