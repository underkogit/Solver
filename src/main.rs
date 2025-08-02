mod lua_engine;
mod utility;
mod basic;
mod io;
mod text;

use anyhow::Result;
use clap::Parser;
use colored::*;
use lua_engine::LuaEngine;
use std::path::PathBuf;

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("{}", "LuaBuild - Modern Build Automation System".bright_cyan().bold());
        println!("{}", format!("Using script: {}", args.script.display()).dimmed());
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
    lua_engine.execute_script(&args.script, &args.target, args.verbose).await?;

    Ok(())
}