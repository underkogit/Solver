use mlua::{Lua, Value};
use std::path::Path;

pub fn setup_globals_utility(
    lua: Lua,
    script_path: String,
    target: &Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    let globals = lua.globals();

    // Получаем базовую директорию скрипта для локальных включений
    let base_path = Path::new(&script_path)
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf();

    // ================ Утилиты преобразования ================

    // Преобразует любое Lua значение в строку
    // local str = to_string(42)
    // local str2 = to_string(true)
    // local str3 = to_string({key = "value"})
    // Возвращает: string (строковое представление значения)
    let to_string = lua.create_function(|_, value: mlua::Value| {
        match value {
            mlua::Value::String(s) => s.to_str().map(|s| s.to_string()).map_err(mlua::Error::from),
            mlua::Value::Integer(i) => Ok(i.to_string()),
            mlua::Value::Number(n) => Ok(n.to_string()),
            mlua::Value::Boolean(b) => Ok(b.to_string()),
            mlua::Value::Nil => Ok("nil".to_string()),
            mlua::Value::Table(_) => Ok("[table]".to_string()),
            mlua::Value::Function(_) => Ok("[function]".to_string()),
            mlua::Value::Thread(_) => Ok("[thread]".to_string()),
            mlua::Value::UserData(_) => Ok("[userdata]".to_string()),
            mlua::Value::LightUserData(_) => Ok("[lightuserdata]".to_string()),
            mlua::Value::Error(e) => Ok(format!("[error: {}]", e)),
            _ => Ok(format!("{}", "<null>"))
        }
    })?;
    globals.set("to_string", to_string)?;

    // ================ Система включения файлов ================

    // Выполняет Lua скрипт по абсолютному пути
    // include("/path/to/script.lua")
    // Результат: выполняет код из указанного файла в текущем контексте
    let include = lua.create_async_function(|lua, path: String| async move {
        let script_content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                return Err(mlua::Error::external(format!(
                    "Failed to read file '{}': {}",
                    path, e
                )));
            }
        };

        // Выполняем скрипт в текущем контексте
        lua.load(&script_content)
            .set_name(&path)
            .exec_async()
            .await
    })?;
    globals.set("include", include)?;

    // Выполняет Lua скрипт относительно директории текущего скрипта
    // include_local("modules/helper.lua")
    // include_local("../common/utils.lua")
    // Результат: выполняет код из файла относительно папки со скриптом
    let include_local = lua.create_async_function(move |lua, path: String| {
        let base_path = base_path.clone();
        async move {
            let full_path = base_path.join(&path);
            let script_content = match tokio::fs::read_to_string(&full_path).await {
                Ok(content) => content,
                Err(e) => {
                    return Err(mlua::Error::external(format!(
                        "Failed to read local file '{}' (resolved to '{}'): {}",
                        path,
                        full_path.display(),
                        e
                    )));
                }
            };

            // Выполняем скрипт в текущем контексте с информативным именем
            lua.load(&script_content)
                .set_name(&format!("@{}", full_path.display()))
                .exec_async()
                .await
        }
    })?;
    globals.set("include_local", include_local)?;

    // ================ Вспомогательные функции для отладки ================

    // Выводит детальную информацию о Lua значении (для отладки)
    // debug_print({name = "test", value = 42})
    // Вывод: подробная информация о структуре таблицы
    let debug_print = lua.create_function(|lua, value: mlua::Value| {
        fn format_value(lua: &Lua, value: &mlua::Value, indent: usize) -> Result<String, mlua::Error> {
            let spaces = "  ".repeat(indent);
            match value {
                mlua::Value::Nil => Ok("nil".to_string()),
                mlua::Value::Boolean(b) => Ok(b.to_string()),
                mlua::Value::Integer(i) => Ok(i.to_string()),
                mlua::Value::Number(n) => Ok(n.to_string()),
                mlua::Value::String(s) => Ok(format!("\"{}\"", s.to_str()?)),
                mlua::Value::Table(table) => {
                    let mut result = "{\n".to_string();
                    for pair in table.pairs::<mlua::Value, mlua::Value>() {
                        let (key, val) = pair?;
                        let key_str = format_value(lua, &key, 0)?;
                        let val_str = format_value(lua, &val, indent + 1)?;
                        result.push_str(&format!("{}  [{}] = {}\n", spaces, key_str, val_str));
                    }
                    result.push_str(&format!("{}}}", spaces));
                    Ok(result)
                },
                mlua::Value::Function(_) => Ok("[function]".to_string()),
                mlua::Value::Thread(_) => Ok("[thread]".to_string()),
                mlua::Value::UserData(_) => Ok("[userdata]".to_string()),
                mlua::Value::LightUserData(_) => Ok("[lightuserdata]".to_string()),
                mlua::Value::Error(e) => Ok(format!("[error: {}]", e)),

                _ => Ok(format!("{}", "<null>"))
            }
        }

        match format_value(&lua, &value, 0) {
            Ok(formatted) => {
                println!("[DEBUG] {}", formatted);
                Ok(())
            },
            Err(e) => {
                println!("[DEBUG ERROR] Failed to format value: {}", e);
                Ok(())
            }
        }
    })?;
    globals.set("debug_print", debug_print)?;

    // Проверяет тип Lua значения
    // local t = get_type(42)        -- "integer"
    // local t2 = get_type("hello")  -- "string"
    // local t3 = get_type({})       -- "table"
    // Возвращает: string (название типа)
    let get_type = lua.create_function(|_, value: mlua::Value| {
        let type_name = match value {
            mlua::Value::Nil => "nil",
            mlua::Value::Boolean(_) => "boolean",
            mlua::Value::Integer(_) => "integer",
            mlua::Value::Number(_) => "number",
            mlua::Value::String(_) => "string",
            mlua::Value::Table(_) => "table",
            mlua::Value::Function(_) => "function",
            mlua::Value::Thread(_) => "thread",
            mlua::Value::UserData(_) => "userdata",
            mlua::Value::LightUserData(_) => "lightuserdata",
            mlua::Value::Error(_) => "error",
            _ => "<null>"
        };
        Ok(type_name.to_string())
    })?;
    globals.set("get_type", get_type)?;

    // ================ Измерение времени выполнения ================

    // Создает объект для измерения времени
    // local timer = create_timer()
    // timer:start()
    // -- выполняем операции
    // local elapsed = timer:stop()
    // Возвращает: таблица с методами start(), stop(), elapsed()
    let create_timer = lua.create_function(|lua, ()| {
        let timer_table = lua.create_table()?;

        // Внутреннее состояние таймера
        timer_table.set("_start_time", mlua::Value::Nil)?;
        timer_table.set("_elapsed", 0.0)?;

        // Метод start() - запускает таймер
        let start_fn = lua.create_function(|_, timer: mlua::Table| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            timer.set("_start_time", now)?;
            Ok(())
        })?;
        timer_table.set("start", start_fn)?;

        // Метод stop() - останавливает таймер и записывает время
        let stop_fn = lua.create_function(|_, timer: mlua::Table| {
            let start_time: Option<f64> = timer.get("_start_time")?;
            if let Some(start) = start_time {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                let elapsed = now - start;
                timer.set("_elapsed", elapsed)?;
                timer.set("_start_time", mlua::Value::Nil)?;
                Ok(elapsed)
            } else {
                Err(mlua::Error::external("Timer was not started"))
            }
        })?;
        timer_table.set("stop", stop_fn)?;

        // Метод elapsed() - возвращает прошедшее время
        let elapsed_fn = lua.create_function(|_, timer: mlua::Table| {
            let start_time: Option<f64> = timer.get("_start_time")?;
            let stored_elapsed: f64 = timer.get("_elapsed")?;

            if let Some(start) = start_time {
                // Таймер запущен, возвращаем текущее время
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                Ok(now - start + stored_elapsed)
            } else {
                // Таймер остановлен, возвращаем сохраненное время
                Ok(stored_elapsed)
            }
        })?;
        timer_table.set("elapsed", elapsed_fn)?;

        Ok(timer_table)
    })?;
    globals.set("create_timer", create_timer)?;

    // ================ Работа с массивами/таблицами ================

    // Возвращает количество элементов в таблице
    // local count = table_length({a = 1, b = 2, c = 3})
    // Возвращает: number (количество элементов)
    let table_length = lua.create_function(|_, table: mlua::Table| {
        let mut count = 0;
        for _ in table.pairs::<mlua::Value, mlua::Value>() {
            count += 1;
        }
        Ok(count)
    })?;
    globals.set("table_length", table_length)?;

    // Проверяет пустая ли таблица
    // local empty = table_is_empty({})
    // Возвращает: boolean (true если таблица пустая)
    let table_is_empty = lua.create_function(|_, table: mlua::Table| {
        for _ in table.pairs::<mlua::Value, mlua::Value>() {
            return Ok(false);
        }
        Ok(true)
    })?;
    globals.set("table_is_empty", table_is_empty)?;

    // Объединяет две таблицы (вторая перезаписывает первую)
    // local merged = table_merge({a = 1}, {b = 2, a = 3})
    // Возвращает: table {a = 3, b = 2}
    let table_merge = lua.create_function(|lua, (table1, table2): (mlua::Table, mlua::Table)| {
        let result = lua.create_table()?;

        // Копируем элементы из первой таблицы
        for pair in table1.pairs::<mlua::Value, mlua::Value>() {
            let (key, value) = pair?;
            result.set(key, value)?;
        }

        // Копируем элементы из второй таблицы (с перезаписью)
        for pair in table2.pairs::<mlua::Value, mlua::Value>() {
            let (key, value) = pair?;
            result.set(key, value)?;
        }

        Ok(result)
    })?;
    globals.set("table_merge", table_merge)?;

    // ================ Математические утилиты ================

    // Ограничивает число в заданном диапазоне
    // local clamped = clamp(15, 0, 10)
    // Возвращает: number (10 - ограничено максимумом)
    let clamp = lua.create_function(|_, (value, min, max): (f64, f64, f64)| {
        Ok(value.max(min).min(max))
    })?;
    globals.set("clamp", clamp)?;

    // Округляет число до заданного количества знаков после запятой
    // local rounded = round(3.14159, 2)
    // Возвращает: number (3.14)
    let round = lua.create_function(|_, (value, precision): (f64, Option<i32>)| {
        let precision = precision.unwrap_or(0);
        if precision <= 0 {
            Ok(value.round())
        } else {
            let multiplier = 10_f64.powi(precision);
            Ok((value * multiplier).round() / multiplier)
        }
    })?;
    globals.set("round", round)?;

    // ================ Генерация случайных данных ================

    // Генерирует случайную строку заданной длины
    // local random_str = random_string(8)
    // Возвращает: string (случайная строка из букв и цифр)
    let random_string = lua.create_function(|_, length: usize| {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        let random_string: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Ok(random_string)
    })?;
    globals.set("random_string", random_string)?;

    // Генерирует случайное число в диапазоне
    // local num = random_number(1, 100)
    // Возвращает: number (случайное число от min до max включительно)
    let random_number = lua.create_function(|_, (min, max): (f64, f64)| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(min..=max))
    })?;
    globals.set("random_number", random_number)?;

    // ================ Утилиты для работы с JSON ================

    // Парсит JSON строку в Lua таблицу
    // local data = json_parse('{"name": "test", "value": 42}')
    // Возвращает: table | nil (Lua таблица или nil при ошибке)
    let json_parse = lua.create_function(|lua, json_str: String| {
        use serde_json::Value;

        fn json_to_lua(lua: &Lua, value: &Value) -> mlua::Result<mlua::Value> {
            match value {
                Value::Null => Ok(mlua::Value::Nil),
                Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(mlua::Value::Integer(i))
                    } else if let Some(f) = n.as_f64() {
                        Ok(mlua::Value::Number(f))
                    } else {
                        Ok(mlua::Value::Nil)
                    }
                },
                Value::String(s) => Ok(mlua::Value::String(lua.create_string(s)?)),
                Value::Array(arr) => {
                    let table = lua.create_table()?;
                    for (i, item) in arr.iter().enumerate() {
                        table.set(i + 1, json_to_lua(lua, item)?)?;
                    }
                    Ok(mlua::Value::Table(table))
                },
                Value::Object(obj) => {
                    let table = lua.create_table()?;
                    for (key, value) in obj {
                        table.set(key.clone(), json_to_lua(lua, value)?)?;
                    }
                    Ok(mlua::Value::Table(table))
                }
            }
        }

        match serde_json::from_str::<Value>(&json_str) {
            Ok(value) => json_to_lua(&lua, &value),
            Err(_) => Ok(mlua::Value::Nil),
        }
    })?;
    globals.set("json_parse", json_parse)?;

    // Конвертирует Lua таблицу в JSON строку
    // local json_str = json_stringify({name = "test", value = 42})
    // Возвращает: string | nil (JSON строка или nil при ошибке)
    let json_stringify = lua.create_function(|_, value: mlua::Value| {
        use serde_json::Value;

        fn lua_to_json(value: &mlua::Value) -> Option<Value> {
            match value {
                mlua::Value::Nil => Some(Value::Null),
                mlua::Value::Boolean(b) => Some(Value::Bool(*b)),
                mlua::Value::Integer(i) => Some(Value::Number((*i).into())),
                mlua::Value::Number(n) => {
                    serde_json::Number::from_f64(*n).map(Value::Number)
                },
                mlua::Value::String(s) => {
                    s.to_str().ok().map(|s| Value::String(s.to_string()))
                },
                mlua::Value::Table(table) => {
                    let mut map = serde_json::Map::new();
                    let mut is_array = true;
                    let mut max_index = 0;

                    // Проверяем является ли таблица массивом
                    for pair in table.pairs::<mlua::Value, mlua::Value>() {
                        if let Ok((key, _)) = pair {
                            if let mlua::Value::Integer(i) = key {
                                if i > 0 {
                                    max_index = max_index.max(i as usize);
                                    continue;
                                }
                            }
                            is_array = false;
                            break;
                        }
                    }

                    if is_array && max_index > 0 {
                        let mut vec = Vec::new();
                        vec.resize(max_index, Value::Null);

                        for pair in table.pairs::<mlua::Value, mlua::Value>() {
                            if let Ok((key, val)) = pair {
                                if let mlua::Value::Integer(i) = key {
                                    if let Some(json_val) = lua_to_json(&val) {
                                        if i > 0 && (i as usize) <= max_index {
                                            vec[i as usize - 1] = json_val;
                                        }
                                    }
                                }
                            }
                        }
                        Some(Value::Array(vec))
                    } else {
                        for pair in table.pairs::<mlua::Value, mlua::Value>() {
                            if let Ok((key, val)) = pair {
                                if let Some(key_str) = match key {
                                    mlua::Value::String(s) => s.to_str().ok().map(|s| s.to_string()),
                                    mlua::Value::Integer(i) => Some(i.to_string()),
                                    mlua::Value::Number(n) => Some(n.to_string()),
                                    _ => None,
                                } {
                                    if let Some(json_val) = lua_to_json(&val) {
                                        map.insert(key_str, json_val);
                                    }
                                }
                            }
                        }
                        Some(Value::Object(map))
                    }
                },
                _ => None,
            }
        }

        match lua_to_json(&value) {
            Some(json_value) => match serde_json::to_string(&json_value) {
                Ok(json_str) => Ok(Some(json_str)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    })?;
    globals.set("json_stringify", json_stringify)?;

    Ok(())
}