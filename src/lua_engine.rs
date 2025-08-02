use crate::modules::basic::setup_globals_basic;
use crate::modules::io::{setup_globals_io};
use crate::modules::process::setup_globals_process;
use crate::modules::text::setup_globals_text;
use crate::modules::utility::setup_globals_utility;
use anyhow::Result;
use colored::*;
use mlua::Lua;
use std::path::PathBuf;
use tokio::fs;

pub struct LuaEngine {
    lua: Lua,
    script_path: String,
}

impl LuaEngine {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            script_path: String::new(),
        }
    }

    pub async fn execute_script(
        &mut self,
        script_path: &PathBuf,
        target: &Option<String>,
        verbose: bool,
    ) -> Result<()> {
        let full_path = script_path.canonicalize()?;
        self.script_path = full_path
            .to_str()
            .unwrap()
            .strip_prefix(r"\\?\")
            .unwrap_or(full_path.to_str().unwrap())
            .to_owned();

        let script_content = fs::read_to_string(script_path).await?;

        setup_globals_io(self.lua.clone(), self.script_path.clone() , target, verbose)?;
        setup_globals_utility(self.lua.clone(), self.script_path.clone(), target, verbose)?;
        setup_globals_basic(self.lua.clone(), self.script_path.clone(), target, verbose)?;

        setup_globals_text(self.lua.clone(), self.script_path.clone(), target, verbose)?;
        setup_globals_process(self.lua.clone(), self.script_path.clone(), target, verbose)?;

        if verbose {
            println!("{}", "Executing Lua script...".green());
        }

        self.lua.load(&script_content).exec_async().await?;

        Ok(())
    }
}
