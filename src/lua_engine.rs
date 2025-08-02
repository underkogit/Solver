use anyhow::Result;
use colored::*;
use mlua::Lua;
use std::path::PathBuf;
use tokio::fs;

pub struct LuaEngine {
    lua: Lua,
}

impl LuaEngine {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
        }
    }

    pub async fn execute_script(
        &mut self,
        script_path: &PathBuf,
        target: &Option<String>,
        verbose: bool,
    ) -> Result<()> {
        let script_content = fs::read_to_string(script_path).await?;

        self.setup_globals(target, verbose)?;

        if verbose {
            println!("{}", "Executing Lua script...".green());
        }

        self.lua.load(&script_content).exec_async().await?;

        Ok(())
    }

    fn setup_globals(&self, target: &Option<String>, verbose: bool) -> Result<()> {
        let globals = self.lua.globals();

        globals.set("verbose", verbose)?;

        if let Some(target) = target {
            globals.set("target", target.clone())?;
        }

        let print_fn = self.lua.create_function(|_, text: String| {
            println!("{}", text);
            Ok(())
        })?;
        globals.set("print", print_fn)?;

        let print_success = self.lua.create_function(|_, text: String| {
            println!("{}", text.green());
            Ok(())
        })?;
        globals.set("print_success", print_success)?;

        let print_error = self.lua.create_function(|_, text: String| {
            eprintln!("{}", text.red());
            Ok(())
        })?;
        globals.set("print_error", print_error)?;

        Ok(())
    }
}