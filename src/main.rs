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
mod package;
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
        .subcommand(
            Command::new("init")
                .about("Initialize a new Kiren project")
                .arg(
                    Arg::new("name")
                        .help("Project name")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("install")
                .about("Install dependencies from kiren.toml")
                .alias("i"),
        )
        .subcommand(
            Command::new("add")
                .about("Add a package dependency")
                .arg(
                    Arg::new("package")
                        .help("Package to add (name@version)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("dev")
                        .long("dev")
                        .short('D')
                        .help("Add as dev dependency")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove a package dependency")
                .alias("rm")
                .arg(
                    Arg::new("package")
                        .help("Package to remove")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("Search for packages")
                .arg(
                    Arg::new("query")
                        .help("Search query")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("cache")
                .about("Manage package cache")
                .subcommand(Command::new("clean").about("Clean old packages from cache"))
                .subcommand(Command::new("stats").about("Show cache statistics")),
        )
        .subcommand(
            Command::new("test")
                .about("Run test files")
                .arg(
                    Arg::new("pattern")
                        .help("Test file pattern (e.g., test-*.js)")
                        .required(false)
                        .index(1),
                ),
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

    // Handle package manager subcommands
    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            return handle_init_command(sub_matches).await;
        }
        Some(("install", _)) | Some(("i", _)) => {
            return handle_install_command().await;
        }
        Some(("add", sub_matches)) => {
            return handle_add_command(sub_matches).await;
        }
        Some(("remove", sub_matches)) | Some(("rm", sub_matches)) => {
            return handle_remove_command(sub_matches).await;
        }
        Some(("search", sub_matches)) => {
            return handle_search_command(sub_matches).await;
        }
        Some(("cache", sub_matches)) => {
            return handle_cache_command(sub_matches).await;
        }
        Some(("test", sub_matches)) => {
            return handle_test_command(sub_matches).await;
        }
        _ => {}
    }

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
                   source.contains("import ") ||
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
                let mut _counter = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    _counter += 1;

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

async fn execute_file_with_config(filename: &str, _config: &KirenConfig) -> Result<()> {
    execute_file(filename).await
}

async fn run_repl_with_config(_config: &KirenConfig) -> Result<()> {
    run_repl().await
}

async fn run_watch_mode_with_config(filename: &str, _config: &KirenConfig) -> Result<()> {
    run_watch_mode(filename).await
}

async fn execute_file_with_engine(filename: &str, engine: &mut Engine) -> Result<()> {
    let source = fs::read_to_string(filename)?;

    // Check file extension to determine execution mode
    let is_module = filename.ends_with(".mjs") || 
                   filename.ends_with(".esm.js") ||
                   source.contains("import ") ||
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

// Package Manager Command Handlers

async fn handle_init_command(matches: &clap::ArgMatches) -> Result<()> {
    let project_name = matches.get_one::<String>("name")
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            "kiren-app"
        });

    let current_dir = std::env::current_dir()?;
    
    
    package::KirenPackageManager::init(&current_dir, project_name).await?;
    
    
    Ok(())
}

async fn handle_install_command() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    
    let package_manager = package::KirenPackageManager::new()?;
    let _packages = package_manager.install(&current_dir).await?;
    
    
    Ok(())
}

async fn handle_add_command(matches: &clap::ArgMatches) -> Result<()> {
    let package_spec = matches.get_one::<String>("package").unwrap();
    let is_dev = matches.get_flag("dev");
    
    
    let package_manager = package::KirenPackageManager::new()?;
    let resolved_package = package_manager.resolve(package_spec).await?;
    
    // Update kiren.toml
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("kiren.toml");
    
    if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        let mut config: package::ProjectConfig = toml::from_str(&content)?;
        
        if is_dev {
            config.add_dev_dependency(&resolved_package.package.name, &resolved_package.package.version);
        } else {
            config.add_dependency(&resolved_package.package.name, &resolved_package.package.version);
        }
        
        let updated_content = config.to_toml()?;
        tokio::fs::write(&config_path, updated_content).await?;
        
    } else {
        return Err(anyhow::anyhow!("No kiren.toml found"));
    }
    
    Ok(())
}

async fn handle_remove_command(matches: &clap::ArgMatches) -> Result<()> {
    let package_name = matches.get_one::<String>("package").unwrap();
    
    
    // Update kiren.toml
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("kiren.toml");
    
    if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        let mut config: package::ProjectConfig = toml::from_str(&content)?;
        
        let removed_from_deps = config.dependencies.remove(package_name).is_some();
        let removed_from_dev = config.dev_dependencies.remove(package_name).is_some();
        
        if removed_from_deps || removed_from_dev {
            let updated_content = config.to_toml()?;
            tokio::fs::write(&config_path, updated_content).await?;
            
        } else {
        }
    } else {
        return Err(anyhow::anyhow!("No kiren.toml found"));
    }
    
    Ok(())
}

async fn handle_search_command(matches: &clap::ArgMatches) -> Result<()> {
    let query = matches.get_one::<String>("query").unwrap();
    
    
    let _package_manager = package::KirenPackageManager::new()?;
    let registry = package::KirenRegistry::new("https://registry.kiren.dev".to_string());
    
    let results = registry.search(query).await?;
    
    if results.is_empty() {
        println!("No packages found matching '{}'", query);
    } else {
        println!("Found {} package(s):", results.len());
        for package in results {
            println!("  {}", package);
        }
        println!();
        println!("Add a package: kiren add <package>@<version>");
    }
    
    Ok(())
}

async fn handle_cache_command(matches: &clap::ArgMatches) -> Result<()> {
    let _package_manager = package::KirenPackageManager::new()?;
    let cache = package::GlobalCache::new(
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".kiren")
            .join("cache")
    )?;
    
    match matches.subcommand() {
        Some(("clean", _)) => {
            cache.clean(30).await?; // Clean packages older than 30 days
        }
        Some(("stats", _)) => {
            let stats = cache.stats().await?;
            stats.print();
        }
        _ => {
            println!("Available cache commands:");
            println!("  kiren cache clean  - Clean old packages");
            println!("  kiren cache stats  - Show cache statistics");
        }
    }
    
    Ok(())
}

async fn handle_test_command(matches: &clap::ArgMatches) -> Result<()> {
    let pattern = matches.get_one::<String>("pattern")
        .map(|s| s.as_str())
        .unwrap_or("test-*.js");
    
    let current_dir = std::env::current_dir()?;
    let test_dir = current_dir.join("tests");
    
    if !test_dir.exists() {
        println!("No tests directory found. Creating tests/...");
        std::fs::create_dir_all(&test_dir)?;
        println!("Place your test files in the tests/ directory.");
        return Ok(());
    }
    
    // Find test files
    let mut test_files = Vec::new();
    for entry in std::fs::read_dir(&test_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if name.ends_with(".js") && (pattern == "test-*.js" && name.starts_with("test-") || name.contains(pattern)) {
                    test_files.push(path);
                }
            }
        }
    }
    
    if test_files.is_empty() {
        println!("No test files found matching pattern: {}", pattern);
        return Ok(());
    }
    
    println!("Running {} test file(s)...\n", test_files.len());
    
    // Execute each test file
    for test_file in test_files {
        println!("Running: {}", test_file.display());
        
        let mut engine = Engine::new()?;
        let source = std::fs::read_to_string(&test_file)?;
        
        match engine.execute(&source) {
            Ok(_) => {
                println!("✅ {} completed", test_file.file_name().unwrap().to_string_lossy());
            }
            Err(e) => {
                println!("❌ {} failed: {}", test_file.file_name().unwrap().to_string_lossy(), e);
            }
        }
        println!();
    }
    
    // Print test summary
    crate::api::test::print_test_summary();
    
    Ok(())
}
