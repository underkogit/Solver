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
        self.script_path = full_path.to_str().unwrap()
            .strip_prefix(r"\\?\").unwrap_or(full_path.to_str().unwrap())
            .to_owned();









        let script_content = fs::read_to_string(script_path).await?;
        self.setup_globals(target, verbose)?;
        self.setup_globals_io(target, verbose)?;
        self.setup_globals_utility(target, verbose)?;
        if verbose {
            println!("{}", "Executing Lua script...".green());
        }

        self.lua.load(&script_content).exec_async().await?;

        Ok(())
    }

    fn setup_globals(&self, target: &Option<String>, verbose: bool) -> Result<()> {
        let globals = self.lua.globals();

        // ================ Basic Variables ================
        globals.set("verbose", verbose)?;

        if let Some(target) = target {
            globals.set("target", target.clone())?;
        }

        globals.set("lua_script_path", self.script_path.clone())?;

        // // ================ Print Functions ================
        // let print_fn = self.lua.create_function(|_, text: String| {
        //     println!("{}", text);
        //     Ok(())
        // })?;
        // globals.set("print", print_fn)?;

        // ================ Success Print ================
        let print_success = self.lua.create_function(|_, text: String| {
            println!("{}", text.green());
            Ok(())
        })?;
        globals.set("print_success", print_success)?;

        // ================ Error Print ================
        let print_error = self.lua.create_function(|_, text: String| {
            eprintln!("{}", text.red());
            Ok(())
        })?;
        globals.set("print_error", print_error)?;

        Ok(())
    }

    fn setup_globals_utility(&self, target: &Option<String>, verbose: bool) -> Result<()> {
        let globals = self.lua.globals();

        // ================ Utility Functions ================
        let to_string = self.lua.create_function(|_, value: mlua::Value| {
            match value {
                mlua::Value::String(s) => Ok(s.to_str()?.to_string()),
                mlua::Value::Integer(i) => Ok(i.to_string()),
                mlua::Value::Number(n) => Ok(n.to_string()),
                mlua::Value::Boolean(b) => Ok(b.to_string()),
                mlua::Value::Nil => Ok("nil".to_string()),
                mlua::Value::Table(_) => Ok("[table]".to_string()),
                mlua::Value::Function(_) => Ok("[function]".to_string()),
                _ => Ok("[unknown]".to_string()),
            }
        })?;
        globals.set("to_string", to_string)?;

        Ok(())
    }

    fn setup_globals_io(&self, target: &Option<String>, verbose: bool) -> Result<()> {
        let globals = self.lua.globals();


        // ================ File Operations ================
        let file_exists = self.lua.create_async_function(|_, path: String| async move {
            Ok(tokio::fs::metadata(path).await.is_ok())
        })?;
        globals.set("file_exists", file_exists)?;

        let dir_exists = self.lua.create_async_function(|_, path: String| async move {
            match tokio::fs::metadata(&path).await {
                Ok(metadata) => Ok(metadata.is_dir()),
                Err(_) => Ok(false),
            }
        })?;
        globals.set("dir_exists", dir_exists)?;

        let read_file = self.lua.create_async_function(|_, path: String| async move {
            match tokio::fs::read_to_string(path).await {
                Ok(content) => Ok(Some(content)),
                Err(_) => Ok(None),
            }
        })?;
        globals.set("read_file", read_file)?;

        let write_file = self.lua.create_async_function(|_, (path, content): (String, String)| async move {
            tokio::fs::write(path, content).await.map_err(|e| mlua::Error::external(e))
        })?;
        globals.set("write_file", write_file)?;

        let create_dir = self.lua.create_async_function(|_, path: String| async move {
            tokio::fs::create_dir_all(path).await.map_err(|e| mlua::Error::external(e))
        })?;
        globals.set("create_dir", create_dir)?;

        let delete_file = self.lua.create_async_function(|_, path: String| async move {
            tokio::fs::remove_file(path).await.map_err(|e| mlua::Error::external(e))
        })?;
        globals.set("delete_file", delete_file)?;

        let delete_dir = self.lua.create_async_function(|_, path: String| async move {
            tokio::fs::remove_dir_all(path).await.map_err(|e| mlua::Error::external(e))
        })?;
        globals.set("delete_dir", delete_dir)?;

        Ok(())
    }
}