use mlua::Lua;

pub fn setup_globals_io(lua: Lua, target: &Option<String>, verbose: bool) -> anyhow::Result<()> {
    let globals = lua.globals();


    // ================ File Operations ================
    let file_exists = lua.create_async_function(|_, path: String| async move {
        Ok(tokio::fs::metadata(path).await.is_ok())
    })?;
    globals.set("file_exists", file_exists)?;

    let dir_exists = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => Ok(metadata.is_dir()),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("dir_exists", dir_exists)?;

    let read_file = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(Some(content)),
            Err(_) => Ok(None),
        }
    })?;
    globals.set("read_file", read_file)?;

    let write_file = lua.create_async_function(|_, (path, content): (String, String)| async move {
        tokio::fs::write(path, content).await.map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("write_file", write_file)?;

    let create_dir = lua.create_async_function(|_, path: String| async move {
        tokio::fs::create_dir_all(path).await.map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("create_dir", create_dir)?;

    let delete_file = lua.create_async_function(|_, path: String| async move {
        tokio::fs::remove_file(path).await.map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("delete_file", delete_file)?;

    let delete_dir = lua.create_async_function(|_, path: String| async move {
        tokio::fs::remove_dir_all(path).await.map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("delete_dir", delete_dir)?;

    Ok(())
}

pub fn setup_globals_path(lua: Lua , script_path: String, target: &Option<String>, verbose: bool) -> anyhow::Result<()>{
    let globals = lua.globals();





    // ================ Path Operations ================
    let get_full_path = lua.create_function(|_, path: String| {
        let path_buf = std::path::Path::new(&path);
        let absolute = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(path_buf)
        };

        let path_str = absolute.to_str().unwrap_or(&path);
        let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
        let normalized = clean_path.replace('/', "\\");
        Ok(normalized)
    })?;
    globals.set("get_full_path", get_full_path)?;

    let script_dir = std::path::Path::new(&script_path)
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let resolve_path = lua.create_function(move |_, path: String| {
        let script_dir = script_dir.clone();
        let path_buf = std::path::Path::new(&path);
        let absolute = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            script_dir.join(path_buf)
        };

        let path_str = absolute.to_str().unwrap_or(&path);
        let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
        let normalized = clean_path.replace('/', "\\");
        Ok(normalized)
    })?;
    globals.set("resolve_path", resolve_path)?;
    Ok(())
}