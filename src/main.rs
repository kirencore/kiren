use anyhow::Result;
use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};

mod api;
mod modules;
mod runtime;
use runtime::engine::Engine;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("kiren")
        .version("0.1.0")
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
        .get_matches();

    if matches.get_flag("repl") || matches.get_one::<String>("file").is_none() {
        run_repl().await?;
    } else if let Some(filename) = matches.get_one::<String>("file") {
        execute_file(filename).await?;
    }

    Ok(())
}

async fn execute_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    let mut engine = Engine::new()?;

    match engine.execute(&source) {
        Ok(_) => {
            // Check if this is an HTTP server script
            if source.contains("http.createServer") || source.contains("server.listen") {
                println!("📡 HTTP server detected. Keeping process alive...");
                println!("Press Ctrl+C to stop the server.");
                println!("⏳ Waiting for server to fully initialize...");

                // Give server time to start
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

                // Keep the process alive for HTTP server
                let mut counter = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    counter += 1;

                    if counter % 30 == 0 {
                        // Every 30 seconds
                        println!("📊 HTTP server running for {} seconds", counter);
                    }

                    // Process timer callbacks
                    if let Err(e) = engine.execute_with_callbacks("", true) {
                        if !e.to_string().contains("Failed to compile") {
                            eprintln!("Timer callback error: {}", e);
                        }
                    }
                }
            } else {
                // Keep processing timer callbacks for a few seconds for regular scripts
                for _ in 0..50 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    if let Err(e) = engine.execute_with_callbacks("", true) {
                        if !e.to_string().contains("Failed to compile") {
                            eprintln!("Timer callback error: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Error executing {}: {}", filename, e),
    }

    Ok(())
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
