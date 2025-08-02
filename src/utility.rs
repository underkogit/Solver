use mlua::Lua;

pub fn setup_globals_utility(lua: Lua , script_path: String, target: &Option<String>, verbose: bool) -> anyhow::Result<()> {
    let globals = lua.globals();
    let base_path = std::path::Path::new(&script_path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    
    // ================ Utility Functions ================
    let to_string = lua.create_function(|_, value: mlua::Value| {
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


    let include = lua.create_async_function(|lua, path: String| async move {
        let script_content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => return Err(mlua::Error::external(format!("Failed to read file '{}': {}", path, e))),
        };

        lua.load(&script_content).exec_async().await
    })?;
    globals.set("include", include)?;





    let include_local = lua.create_async_function(move |lua, path: String| {
        let base_path = base_path.clone();
        async move {
            let full_path = base_path.join(&path);
            let script_content = match tokio::fs::read_to_string(&full_path).await {
                Ok(content) => content,
                Err(e) => return Err(mlua::Error::external(format!("Failed to read file '{}': {}", full_path.display(), e))),
            };

            lua.load(&script_content).exec_async().await
        }
    })?;
    globals.set("include_local", include_local)?;
    Ok(())
}