use std::path::PathBuf;
use colored::Colorize;
use mlua::Lua;

pub fn setup_globals_basic(lua: Lua, script_path: String, target: &Option<String>, verbose: bool) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ================ Basic Variables ================
    globals.set("verbose", verbose)?;

    if let Some(target) = target {
        globals.set("target", target.clone())?;
    }

    globals.set("lua_script_path", script_path)?;



    // ================ Success Print ================
    let print_success = lua.create_function(|_, text: String| {
        println!("{}", text.green());
        Ok(())
    })?;
    globals.set("print_success", print_success)?;

    // ================ Error Print ================
    let print_error = lua.create_function(|_, text: String| {
        eprintln!("{}", text.red());
        Ok(())
    })?;
    globals.set("print_error", print_error)?;

    Ok(())
}