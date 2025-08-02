use mlua::Lua;
use std::path::Path;

pub fn setup_globals_io(
    lua: Lua,
    full_path_string: String,
    target: &Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ================ Проверка существования файлов и папок ================

    // Проверяет существование файла или папки
    // local exists = file_exists("config.toml")
    // Возвращает: boolean (true если файл/папка существует)
    let file_exists = lua.create_async_function(|_, path: String| async move {
        Ok(tokio::fs::metadata(path).await.is_ok())
    })?;
    globals.set("file_exists", file_exists)?;

    // Проверяет является ли путь директорией
    // local is_dir = dir_exists("src")
    // Возвращает: boolean (true если существует и является папкой)
    let dir_exists = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => Ok(metadata.is_dir()),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("dir_exists", dir_exists)?;

    // ================ Чтение и запись файлов ================

    // Читает содержимое файла как строку
    // local content = read_file("Cargo.toml")
    // Возвращает: string | nil (содержимое файла или nil при ошибке)
    let read_file = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(Some(content)),
            Err(_) => Ok(None),
        }
    })?;
    globals.set("read_file", read_file)?;

    // Записывает строку в файл
    // write_file("output.txt", "Hello, World!")
    // Результат: создает или перезаписывает файл
    let write_file = lua.create_async_function(|_, (path, content): (String, String)| async move {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("write_file", write_file)?;

    // ================ Операции с директориями ================

    // Создает директорию и все родительские папки
    // create_dir("build/debug/output")
    // Результат: создает полный путь директорий
    let create_dir = lua.create_async_function(|_, path: String| async move {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("create_dir", create_dir)?;

    // Удаляет файл
    // delete_file("temp.txt")
    // Результат: удаляет файл из файловой системы
    let delete_file = lua.create_async_function(|_, path: String| async move {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("delete_file", delete_file)?;

    // Удаляет директорию и все содержимое
    // delete_dir("temp_build")
    // Результат: рекурсивно удаляет папку со всем содержимым
    let delete_dir = lua.create_async_function(|_, path: String| async move {
        tokio::fs::remove_dir_all(path)
            .await
            .map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("delete_dir", delete_dir)?;

    // ================ Копирование ================

    // Копирует файл из одного места в другое
    // copy_file("src.txt", "dest.txt")
    // Результат: создает копию файла в новом месте
    let copy_file = lua.create_async_function(|_, (src, dest): (String, String)| async move {
        tokio::fs::copy(src, dest)
            .await
            .map_err(|e| mlua::Error::external(e))
            .map(|_| ())
    })?;
    globals.set("copy_file", copy_file)?;

    // Рекурсивно копирует директорию со всем содержимым
    // copy_dir("src_folder", "dest_folder")
    // Результат: создает полную копию папки со всеми файлами и подпапками
    let copy_dir = lua.create_async_function(|_, (src, dest): (String, String)| async move {
        copy_dir_iterative(&src, &dest)
            .await
            .map_err(|e| mlua::Error::external(e))
    })?;
    globals.set("copy_dir", copy_dir)?;

    // ================ Работа с путями ================

    // Преобразует относительный путь в абсолютный
    // local abs_path = get_full_path("../config")
    // Возвращает: string (полный абсолютный путь)
    let get_full_path = lua.create_function(|_, path: String| {
        let path_buf = Path::new(&path);
        let absolute = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(path_buf)
        };

        let path_str = absolute.to_str().unwrap_or(&path);
        let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
        #[cfg(windows)]
        let normalized = clean_path.replace('/', "\\");
        #[cfg(not(windows))]
        let normalized = clean_path.to_string();
        Ok(normalized)
    })?;
    globals.set("get_full_path", get_full_path)?;

    // Преобразует путь относительно директории скрипта в абсолютный
    // local script_relative = resolve_path("configs/build.toml")
    // Возвращает: string (путь относительно папки со скриптом)
    let script_dir = Path::new(&full_path_string)
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    let resolve_path = lua.create_function(move |_, path: String| {
        let script_dir = script_dir.clone();
        let path_buf = Path::new(&path);
        let absolute = if path_buf.is_absolute() {
            path_buf.to_path_buf()
        } else {
            script_dir.join(path_buf)
        };

        let path_str = absolute.to_str().unwrap_or(&path);
        let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
        #[cfg(windows)]
        let normalized = clean_path.replace('/', "\\");
        #[cfg(not(windows))]
        let normalized = clean_path.to_string();
        Ok(normalized)
    })?;
    globals.set("resolve_path", resolve_path)?;

    Ok(())
}

async fn copy_dir_iterative(src: &str, dest: &str) -> tokio::io::Result<()> {
    use std::collections::VecDeque;
    use tokio::fs;

    let mut queue = VecDeque::new();
    queue.push_back((src.to_string(), dest.to_string()));

    while let Some((current_src, current_dest)) = queue.pop_front() {
        fs::create_dir_all(&current_dest).await?;

        let mut entries = fs::read_dir(&current_src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let dest_path = Path::new(&current_dest).join(entry.file_name());

            if entry_path.is_dir() {
                queue.push_back((
                    entry_path.to_string_lossy().to_string(),
                    dest_path.to_string_lossy().to_string(),
                ));
            } else {
                fs::copy(&entry_path, &dest_path).await?;
            }
        }
    }

    Ok(())
}