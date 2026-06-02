#![allow(dead_code)]

mod driver;
mod error;
mod lexer;
mod parser;
mod typeck;
mod hir;
mod mir;
mod codegen;
mod runtime;
mod pkg;

use clap::{Parser, Subcommand};
use driver::session::Session;
use error::Diagnostics;

#[derive(Parser)]
#[command(name = "oys", version = "0.1.0", about = "OY# Compiler for the Y# Programming Language — Ultra-Optimized for Games & Simulations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short = 't', long = "target", default_value = "native")]
        target: String,
        #[arg(short = 'o', long = "output")]
        output: Option<String>,
        #[arg(short = 'L', long = "log-level", default_value = "warn")]
        log_level: String,
        file: String,
    },
    Run {
        #[arg(short = 't', long = "target", default_value = "native")]
        target: String,
        file: String,
    },
    Pack {
        #[command(subcommand)]
        command: PackCommands,
    },
    Test {
        file: String,
    },
    New {
        name: String,
    },
}

#[derive(Subcommand)]
enum PackCommands {
    Add { package: String },
    Remove { package: String },
    Publish,
}

fn create_project_template(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(name)?;
    std::fs::write(
        format!("{}/main.ys", name),
        format!("Program {name} {{\n    Print(\"Hello from Y#!\");\n}}\n"),
    )?;
    std::fs::write(
        format!("{}/oy.toml", name),
        format!("[project]\nname = \"{name}\"\nversion = \"0.1.0\"\ntarget = \"native\"\n"),
    )?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { target, output, log_level, file } => {
            let diag = Diagnostics::new();
            let mut session = Session::new(&diag, &log_level);
            let result = session.build(&file, &target, output.as_deref());
            match result {
                Ok(path) => eprintln!("Build succeeded: {}", path),
                Err(e) => {
                    eprintln!("Build failed: {}", e);
                    diag.emit();
                    std::process::exit(1);
                }
            }
        }
        Commands::Run { target, file } => {
            let diag = Diagnostics::new();
            let mut session = Session::new(&diag, "warn");
            match session.build(&file, &target, None) {
                Ok(path) => {
                    let status = std::process::Command::new(&path)
                        .status()
                        .expect("failed to run compiled binary");
                    std::process::exit(status.code().unwrap_or(0));
                }
                Err(e) => {
                    eprintln!("Build failed: {}", e);
                    diag.emit();
                    std::process::exit(1);
                }
            }
        }
        Commands::Pack { command } => {
            let result = match command {
                PackCommands::Add { package } => pkg::install(&package),
                PackCommands::Remove { package } => pkg::remove(&package),
                PackCommands::Publish => pkg::publish(),
            };
            if let Err(e) = result {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Test { file } => {
            let paths: Vec<std::path::PathBuf> = {
                let p = std::path::Path::new(&file);
                if p.is_dir() {
                    std::fs::read_dir(p)
                        .unwrap()
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.extension().map(|e| e == "ys").unwrap_or(false))
                        .collect()
                } else {
                    vec![p.to_path_buf()]
                }
            };

            if paths.is_empty() {
                eprintln!("{}: no test files found", "\x1b[1;33mwarning\x1b[0m");
                return;
            }

            let total = paths.len();
            let mut passed = 0u32;
            let mut failed = 0u32;

            eprintln!("   \x1b[1;34mRunning Y# tests\x1b[0m");
            eprintln!();

            for path in &paths {
                let diag = Diagnostics::new();
                let mut session = Session::new(&diag, "error");
                let filename = path.file_name().unwrap().to_string_lossy();
                print!("  \x1b[1;37mtest\x1b[0m {} ... ", filename);

                let result = session.build(
                    &path.to_string_lossy(),
                    "native",
                    None,
                );

                match result {
                    Ok(exe_path) => {
                        // Run the compiled binary and capture output
                        let run_result = std::process::Command::new(&exe_path)
                            .output();
                        match run_result {
                            Ok(output) => {
                                if output.status.success() {
                                    println!("\x1b[1;32mok\x1b[0m");
                                    passed += 1;
                                } else {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    println!("\x1b[1;31mFAILED\x1b[0m");
                                    if !stderr.trim().is_empty() {
                                        for line in stderr.lines() {
                                            eprintln!("    {}", line);
                                        }
                                    }
                                    failed += 1;
                                }
                                // Clean up compiled binary
                                let _ = std::fs::remove_file(&exe_path);
                            }
                            Err(e) => {
                                println!("\x1b[1;31mFAILED\x1b[0m (could not run: {})", e);
                                failed += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("\x1b[1;31mFAILED\x1b[0m");
                        diag.emit();
                        eprintln!("  \x1b[2m{}\x1b[0m", e);
                        failed += 1;
                    }
                }
            }

            eprintln!();
            let summary = if failed > 0 {
                format!(
                    "\x1b[1;31m{} passed, {} failed, {} total\x1b[0m",
                    passed, failed, total
                )
            } else {
                format!(
                    "\x1b[1;32m{} passed, {} failed, {} total\x1b[0m",
                    passed, failed, total
                )
            };
            eprintln!("  {}", summary);
            if failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::New { name } => {
            std::fs::create_dir_all(&name).expect("failed to create project dir");
            create_project_template(&name).expect("failed to create project");
            eprintln!("Created new Y# project '{}'", name);
            eprintln!("  cd {} && oys build main.ys", name);
        }
    }
}
