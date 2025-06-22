use anyhow::Result;
use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;

mod api;
mod config;
mod modules;
mod runtime;
use runtime::engine::Engine;
use config::KirenConfig;
use api::console;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("kiren")
        .version("0.2.0")
        .about("A high-performance JavaScript runtime built with Rust")
        .arg(
            Arg::new("file")
                .help("JavaScript file to execute")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("repl")
                .short('r')
                .long("repl")
                .help("Start in REPL mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .help("Watch file for changes and restart automatically")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file path")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("env")
                .short('e')
                .long("env")
                .help("Environment (development, production, test)")
                .value_name("ENV"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("Default HTTP server port")
                .value_name("PORT"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("silent")
                .short('s')
                .long("silent")
                .help("Suppress output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Load configuration
    let mut config = KirenConfig::load();
    config.merge_with_env();

    // Override with CLI arguments
    if let Some(config_path) = matches.get_one::<String>("config") {
        let path = std::path::PathBuf::from(config_path);
        match KirenConfig::load_from_file(&path) {
            Ok(cli_config) => {
                config = cli_config;
                eprintln!("Loaded config from: {}", path.display());
            }
            Err(e) => eprintln!("Failed to load config from {}: {}", path.display(), e),
        }
    }

    if let Some(env) = matches.get_one::<String>("env") {
        config.environment.insert("NODE_ENV".to_string(), env.clone());
    }

    if let Some(port) = matches.get_one::<String>("port") {
        if let Ok(port) = port.parse::<u16>() {
            config.server.default_port = port;
        }
    }

    // Set logging level
    let verbose = matches.get_flag("verbose");
    let silent = matches.get_flag("silent");
    
    // Configure console logging
    console::configure_console(verbose, silent);

    if matches.get_flag("repl") || matches.get_one::<String>("file").is_none() {
        run_repl_with_config(&config).await?;
    } else if let Some(filename) = matches.get_one::<String>("file") {
        if matches.get_flag("watch") {
            run_watch_mode_with_config(filename, &config).await?;
        } else {
            execute_file_with_config(filename, &config).await?;
        }
    }

    Ok(())
}

async fn execute_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    let mut engine = Engine::new()?;

    // Check file extension to determine execution mode
    let is_module = filename.ends_with(".mjs") || 
                   filename.ends_with(".esm.js") ||
                   source.trim_start().starts_with("import ") ||
                   source.contains("export ");

    let result = if is_module {
        println!("Executing ES module: {}", filename);
        engine.execute_module(&source, filename)
    } else {
        engine.execute(&source)
    };

    match result {
        Ok(_) => {
            // Check if this is an HTTP server script
            if source.contains("http.createServer") || source.contains("server.listen") {
                // Extract port from server.listen(port) calls
                let port = extract_port_from_source(&source);
                println!("Starting HTTP server on port {}", port);
                println!("Press Ctrl+C to stop the server.");
                
                // Give server thread time to start
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                // Keep the process alive for HTTP server
                let mut counter = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    counter += 1;

                    // Process timer callbacks
                    if let Err(e) = engine.execute_with_callbacks("", true) {
                        if !e.to_string().contains("Failed to compile") {
                            eprintln!("Timer callback error: {}", e);
                        }
                    }
                }
            } else {
                // Quick timer callback processing for regular scripts (reduced from 50 iterations)
                for _ in 0..3 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    if let Err(e) = engine.execute_with_callbacks("", true) {
                        if !e.to_string().contains("Failed to compile") {
                            eprintln!("Timer callback error: {}", e);
                        }
                    }
                }
                println!("Script execution completed");
            }
        }
        Err(e) => eprintln!("Error executing {}: {}", filename, e),
    }

    Ok(())
}

fn extract_port_from_source(source: &str) -> u16 {
    // Look for server.listen(port) patterns
    if let Some(listen_pos) = source.find("server.listen(") {
        let after_listen = &source[listen_pos + 14..];
        if let Some(closing_paren) = after_listen.find(')') {
            let port_str = after_listen[..closing_paren].trim();
            if let Ok(port) = port_str.parse::<u16>() {
                return port;
            }
        }
    }
    
    // Look for .listen(port) patterns
    if let Some(listen_pos) = source.find(".listen(") {
        let after_listen = &source[listen_pos + 8..];
        if let Some(closing_paren) = after_listen.find(')') {
            let port_str = after_listen[..closing_paren].trim();
            if let Ok(port) = port_str.parse::<u16>() {
                return port;
            }
        }
    }
    
    // Default port
    3000
}

async fn run_repl() -> Result<()> {
    println!("Kiren v0.1.0 - High-performance JavaScript Runtime");
    println!("Type '.exit' to quit");

    let mut engine = Engine::new()?;
    let mut input = String::new();

    loop {
        print!("kiren> ");
        io::stdout().flush()?;

        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached
                println!("Goodbye!");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Input error: {}", e);
                continue;
            }
        }

        let line = input.trim();

        if line == ".exit" {
            break;
        }

        if line.is_empty() {
            continue;
        }

        // Handle REPL commands
        if line.starts_with('.') {
            match line {
                ".exit" => break,
                ".help" => {
                    println!("Available commands:");
                    println!("  .exit  - Exit the REPL");
                    println!("  .help  - Show this help");
                    continue;
                }
                _ => {
                    println!("Unknown command: {}", line);
                    continue;
                }
            }
        }

        match engine.execute(line) {
            Ok(result) => {
                if result != "undefined" && !result.is_empty() {
                    println!("{}", result);
                }
                // Process timer callbacks in REPL
                if let Err(e) = engine.execute_with_callbacks("", true) {
                    if !e.to_string().contains("Failed to compile") {
                        eprintln!("Timer callback error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}

async fn execute_file_with_config(filename: &str, config: &KirenConfig) -> Result<()> {
    execute_file(filename).await
}

async fn run_repl_with_config(config: &KirenConfig) -> Result<()> {
    run_repl().await
}

async fn run_watch_mode_with_config(filename: &str, config: &KirenConfig) -> Result<()> {
    run_watch_mode(filename).await
}

async fn execute_file_with_engine(filename: &str, engine: &mut Engine) -> Result<()> {
    let source = fs::read_to_string(filename)?;

    // Check file extension to determine execution mode
    let is_module = filename.ends_with(".mjs") || 
                   filename.ends_with(".esm.js") ||
                   source.trim_start().starts_with("import ") ||
                   source.contains("export ");

    let result = if is_module {
        engine.execute_module(&source, filename)
    } else {
        engine.execute(&source)
    };

    match result {
        Ok(_) => {
            // Quick timer callback processing for all scripts
            for _ in 0..3 {
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                if let Err(e) = engine.execute_with_callbacks("", true) {
                    if !e.to_string().contains("Failed to compile") {
                        eprintln!("Timer callback error: {}", e);
                    }
                }
            }
            
            // Check if this is an HTTP server script
            if source.contains("http.createServer") || source.contains("server.listen") {
                let port = extract_port_from_source(&source);
                println!("Starting HTTP server on port {}", port);
                
                // Minimal startup delay for server binding
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            } else {
                println!("Script execution completed");
            }
        }
        Err(e) => eprintln!("Error executing {}: {}", filename, e),
    }

    Ok(())
}

async fn run_watch_mode(filename: &str) -> Result<()> {
    println!("Kiren v0.1.0 - File watching enabled");
    println!("Watching: {}", filename);
    println!("Press Ctrl+C to stop watching");
    println!();

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if let Err(e) = tx.send(event) {
                        eprintln!("Watch error: {}", e);
                    }
                }
                Err(e) => eprintln!("Watch error: {}", e),
            }
        },
        Config::default(),
    )?;

    // Watch the file
    watcher.watch(Path::new(filename), RecursiveMode::NonRecursive)?;

    // Create reusable engine for hot reload performance
    let mut engine = Engine::new()?;

    // Initial execution
    println!("Starting initial execution...");
    execute_file_with_engine(filename, &mut engine).await?;

    // Watch for changes with optimized polling
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                // Only process specific modify events (data changes, not metadata)
                if matches!(event.kind, notify::EventKind::Modify(notify::event::ModifyKind::Data(_))) {
                    println!("\nFile changed, restarting...");
                    
                    // Shutdown any existing HTTP servers first
                    api::http::shutdown_http_servers();
                    
                    // Minimal delay for file consistency
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    
                    println!("Reloading: {}", filename);
                    match execute_file_with_engine(filename, &mut engine).await {
                        Ok(_) => println!("Reload completed"),
                        Err(e) => eprintln!("Reload failed: {}", e),
                    }
                    println!();
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No events, continue with minimal delay
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(e) => {
                eprintln!("Watcher error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
