use colored::Colorize;
use mlua::Lua;
use std::path::PathBuf;

pub fn setup_globals_basic(
    lua: Lua,
    script_path: String,
    target: &Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    let globals = lua.globals();
    
    
    
    let path = PathBuf::from(&script_path.clone());
    let directory = path.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_string_lossy()
        .to_string();
    
    
    // ================ Базовые переменные ================

    // Глобальная переменная для включения подробного вывода
    // if verbose then print("Verbose mode enabled") end
    // Тип: boolean
    globals.set("verbose", verbose)?;

    // Текущая цель сборки (может быть nil)
    // if target then print("Building for: " .. target) end
    // Тип: string | nil
    if let Some(target) = target {
        globals.set("target", target.clone())?;
    }

    // Полный путь к текущему Lua скрипту
    // print("Script location: " .. lua_script_path)
    // Тип: string
    globals.set("lua_script_path", script_path.clone())?;




    // Возвращает директорию текущего Lua скрипта
    // local dir = lua_script_directory
    // print("Script directory: " .. dir)
    // Тип: string
    
    globals.set("lua_script_directory", directory.clone())?;


    // ================ Функции вывода ================

    // Выводит текст зеленым цветом (для успешных операций)
    // print_success("Build completed successfully!")
    // Вывод: зеленый текст в консоли
    let print_success = lua.create_function(|_, text: String| {
        println!("{}", text.green());
        Ok(())
    })?;
    globals.set("print_success", print_success)?;

    // Выводит текст красным цветом в stderr (для ошибок)
    // print_error("Failed to compile project")
    // Вывод: красный текст в stderr
    let print_error = lua.create_function(|_, text: String| {
        eprintln!("{}", text.red());
        Ok(())
    })?;
    globals.set("print_error", print_error)?;

    // Выводит обычный текст в stdout
    // println("Processing files...")
    // Вывод: обычный текст в stdout
    let println = lua.create_function(|_, text: String| {
        println!("{}", text);
        Ok(())
    })?;
    globals.set("println", println)?;

    Ok(())
}