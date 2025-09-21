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
    // Возвращает: boolean (true если успешно записан)
    let write_file = lua.create_async_function(|_, (path, content): (String, String)| async move {
        match tokio::fs::write(path, content).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("write_file", write_file)?;

    // ================ Операции с директориями ================

    // Создает директорию и все родительские папки
    // create_dir("build/debug/output")
    // Возвращает: boolean (true если успешно создана)
    let create_dir = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::create_dir_all(path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("create_dir", create_dir)?;

    // Удаляет файл
    // delete_file("temp.txt")
    // Возвращает: boolean (true если успешно удален)
    let delete_file = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::remove_file(path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("delete_file", delete_file)?;

    // Удаляет директорию и все содержимое
    // delete_dir("temp_build")
    // Возвращает: boolean (true если успешно удалена)
    let delete_dir = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::remove_dir_all(path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("delete_dir", delete_dir)?;

    // ================ Копирование ================

    // Копирует файл из одного места в другое
    // copy_file("src.txt", "dest.txt")
    // Возвращает: boolean (true если успешно скопирован)
    let copy_file = lua.create_async_function(|_, (src, dest): (String, String)| async move {
        match tokio::fs::copy(src, dest).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("copy_file", copy_file)?;

    // Рекурсивно копирует директорию со всем содержимым
    // copy_dir("src_folder", "dest_folder")
    // Возвращает: boolean (true если успешно скопирована)
    let copy_dir = lua.create_async_function(|_, (src, dest , dir_ignore): (String, String , String)| async move {
        match copy_dir_iterative(&src, &dest , &dir_ignore).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("copy_dir", copy_dir)?;

    // ================ Получение списков файлов и директорий ================

    // Получает список всех файлов и папок в директории
    // local items = list_dir("src")
    // Возвращает: table | nil (массив строк с именами файлов и папок)
    let list_dir = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut items = Vec::new();
                while let Some(entry) = entries.next_entry().await.map_err(|e| mlua::Error::external(e))? {
                    if let Some(name) = entry.file_name().to_str() {
                        items.push(name.to_string());
                    }
                }
                items.sort();
                Ok(Some(items))
            },
            Err(_) => Ok(None),
        }
    })?;
    globals.set("list_dir", list_dir)?;

    // Получает только файлы в директории (исключая папки)
    // local files = list_files("src")
    // Возвращает: table | nil (массив строк с именами только файлов)
    let list_files = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                while let Some(entry) = entries.next_entry().await.map_err(|e| mlua::Error::external(e))? {
                    let metadata = entry.metadata().await.map_err(|e| mlua::Error::external(e))?;
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            files.push(name.to_string());
                        }
                    }
                }
                files.sort();
                Ok(Some(files))
            },
            Err(_) => Ok(None),
        }
    })?;
    globals.set("list_files", list_files)?;

    // Получает только директории в папке (исключая файлы)
    // local dirs = list_dirs("src")
    // Возвращает: table | nil (массив строк с именами только папок)
    let list_dirs = lua.create_async_function(|_, path: String| async move {
        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut dirs = Vec::new();
                while let Some(entry) = entries.next_entry().await.map_err(|e| mlua::Error::external(e))? {
                    let metadata = entry.metadata().await.map_err(|e| mlua::Error::external(e))?;
                    if metadata.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            dirs.push(name.to_string());
                        }
                    }
                }
                dirs.sort();
                Ok(Some(dirs))
            },
            Err(_) => Ok(None),
        }
    })?;
    globals.set("list_dirs", list_dirs)?;

    // Рекурсивно получает все файлы в директории и поддиректориях
    // local all_files = list_files_recursive("src")
    // Возвращает: table | nil (массив строк с полными путями к файлам)
    let list_files_recursive = lua.create_async_function(|_, path: String| async move {
        match list_files_recursive_impl(&path).await {
            Ok(files) => Ok(Some(files)),
            Err(_) => Ok(None),
        }
    })?;
    globals.set("list_files_recursive", list_files_recursive)?;

    // Получает детальную информацию о содержимом директории
    // local info = list_dir_detailed("src")
    // Возвращает: table | nil с элементами {name, type, size, modified}
    let list_dir_detailed = lua.create_async_function(|lua, path: String| async move {
        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut items = Vec::new();
                while let Some(entry) = entries.next_entry().await.map_err(|e| mlua::Error::external(e))? {
                    if let Ok(metadata) = entry.metadata().await {
                        let item = lua.create_table()?;

                        if let Some(name) = entry.file_name().to_str() {
                            item.set("name", name)?;
                        }

                        let item_type = if metadata.is_file() { "file" } else { "directory" };
                        item.set("type", item_type)?;
                        item.set("size", metadata.len())?;

                        if let Ok(modified) = metadata.modified() {
                            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                                item.set("modified", duration.as_secs())?;
                            }
                        }

                        items.push(item);
                    }
                }
                Ok(Some(items))
            },
            Err(_) => Ok(None),
        }
    })?;
    globals.set("list_dir_detailed", list_dir_detailed)?;

    // Фильтрует файлы по расширению
    // local rs_files = list_files_by_extension("src", "rs")
    // local all_rs_files = list_files_by_extension(".", "rs") -- текущая папка
    // Возвращает: table | nil (массив файлов с указанным расширением)
    let list_files_by_extension = lua.create_async_function(|_, (path, extension): (String, String)| async move {
        match tokio::fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                let target_ext = if extension.starts_with('.') {
                    extension
                } else {
                    format!(".{}", extension)
                };

                while let Some(entry) = entries.next_entry().await.map_err(|e| mlua::Error::external(e))? {
                    let metadata = entry.metadata().await.map_err(|e| mlua::Error::external(e))?;
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(&target_ext) {
                                files.push(name.to_string());
                            }
                        }
                    }
                }
                files.sort();
                Ok(Some(files))
            },
            Err(_) => Ok(None),
        }
    })?;
    globals.set("list_files_by_extension", list_files_by_extension)?;

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

async fn copy_dir_iterative(src: &str, dest: &str , dir_ignore: &str) -> tokio::io::Result<()> {
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
                // Пропускаем директорию obfuscation
                if entry.file_name() != dir_ignore {
                    queue.push_back((
                        entry_path.to_string_lossy().to_string(),
                        dest_path.to_string_lossy().to_string(),
                    ));
                }
            } else {
                fs::copy(&entry_path, &dest_path).await?;
            }
        }
    }

    Ok(())
}

async fn list_files_recursive_impl(path: &str) -> tokio::io::Result<Vec<String>> {
    use std::collections::VecDeque;
    use tokio::fs;

    let mut files = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(path.to_string());

    while let Some(current_path) = queue.pop_front() {
        let mut entries = fs::read_dir(&current_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                files.push(entry_path.to_string_lossy().to_string());
            } else if metadata.is_dir() {
                queue.push_back(entry_path.to_string_lossy().to_string());
            }
        }
    }

    files.sort();
    Ok(files)
}