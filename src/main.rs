mod lua_engine;

mod modules {
    pub mod basic;
    pub mod io;
    pub mod process;
    pub mod text;
    pub mod utility;
}
use anyhow::Result;
use clap::Parser;
use colored::*;
use lua_engine::LuaEngine;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
struct Args {
    #[arg(help = "Path to the Lua build script")]
    script: PathBuf,

    #[arg(short, long, help = "Specify build target")]
    target: Option<String>,

    #[arg(short = 'l', long, help = "List available targets")]
    list_targets: bool,

    #[arg(short, long, help = "Enable verbose output")]
    verbose: bool,
}

#[cfg(windows)]
fn setup_console() {
    // Установка UTF-8 через chcp команду
    Command::new("chcp")
        .arg("65001")
        .output()
        .ok();
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_console();

    let args = Args::parse();

    if args.verbose {
        println!(
            "{}",
            "LuaBuild - Modern Build Automation System"
                .bright_cyan()
                .bold()
        );
        println!(
            "{}",
            format!("Using script: {}", args.script.display()).dimmed()
        );
    }

    println!("Script: {}", args.script.display());

    if let Some(target) = args.target.clone() {
        println!("Target: {}", target);
    }

    if args.list_targets {
        println!("Listing targets...");
        return Ok(());
    }

    let mut lua_engine = LuaEngine::new();
    lua_engine
        .execute_script(&args.script, &args.target, args.verbose)
        .await?;



    Ok(())
}
