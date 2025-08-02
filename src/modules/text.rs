use mlua::Lua;
use regex::Regex;
use std::collections::HashMap;

pub fn setup_globals_text(
    lua: Lua,
    script_path: String,
    target: &Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ================ Форматирование строк ================

    // Удаляет пробелы в начале и конце строки
    // local clean = trim("  hello world  ")
    // Возвращает: string ("hello world")
    let trim = lua.create_function(|_, text: String| Ok(text.trim().to_string()))?;
    globals.set("trim", trim)?;

    // Удаляет пробелы только в начале строки
    // local result = trim_start("  hello")
    // Возвращает: string ("hello")
    let trim_start = lua.create_function(|_, text: String| Ok(text.trim_start().to_string()))?;
    globals.set("trim_start", trim_start)?;

    // Удаляет пробелы только в конце строки
    // local result = trim_end("hello  ")
    // Возвращает: string ("hello")
    let trim_end = lua.create_function(|_, text: String| Ok(text.trim_end().to_string()))?;
    globals.set("trim_end", trim_end)?;

    // Преобразует строку в верхний регистр
    // local upper = to_upper("Hello World")
    // Возвращает: string ("HELLO WORLD")
    let to_upper = lua.create_function(|_, text: String| Ok(text.to_uppercase()))?;
    globals.set("to_upper", to_upper)?;

    // Преобразует строку в нижний регистр
    // local lower = to_lower("Hello World")
    // Возвращает: string ("hello world")
    let to_lower = lua.create_function(|_, text: String| Ok(text.to_lowercase()))?;
    globals.set("to_lower", to_lower)?;

    // Делает первую букву заглавной
    // local cap = capitalize("hello world")
    // Возвращает: string ("Hello world")
    let capitalize = lua.create_function(|_, text: String| {
        let mut chars: Vec<char> = text.chars().collect();
        if let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
        Ok(chars.into_iter().collect::<String>())
    })?;
    globals.set("capitalize", capitalize)?;

    // Разделяет строку по разделителю
    // local parts = split("a,b,c", ",")
    // Возвращает: table {"a", "b", "c"}
    let split = lua.create_function(|_, (text, delimiter): (String, String)| {
        let parts: Vec<String> = text.split(&delimiter).map(|s| s.to_string()).collect();
        Ok(parts)
    })?;
    globals.set("split", split)?;

    // Объединяет массив строк с разделителем
    // local result = join({"a", "b", "c"}, ",")
    // Возвращает: string ("a,b,c")
    let join = lua.create_function(|_, (parts, delimiter): (Vec<String>, String)| {
        Ok(parts.join(&delimiter))
    })?;
    globals.set("join", join)?;

    // Добавляет символы слева до нужной длины
    // local padded = pad_left("42", 5, "0")
    // Возвращает: string ("00042")
    let pad_left = lua.create_function(|_, (text, width, ch): (String, usize, Option<String>)| {
        let pad_char = ch.unwrap_or_else(|| " ".to_string()).chars().next().unwrap_or(' ');
        let current_len = text.chars().count();

        if current_len >= width {
            Ok(text)
        } else {
            let padding = pad_char.to_string().repeat(width - current_len);
            Ok(format!("{}{}", padding, text))
        }
    })?;
    globals.set("pad_left", pad_left)?;

    // Добавляет символы справа до нужной длины
    // local padded = pad_right("42", 5, "0")
    // Возвращает: string ("42000")
    let pad_right = lua.create_function(|_, (text, width, ch): (String, usize, Option<String>)| {
        let pad_char = ch.unwrap_or_else(|| " ".to_string()).chars().next().unwrap_or(' ');
        let current_len = text.chars().count();

        if current_len >= width {
            Ok(text)
        } else {
            let padding = pad_char.to_string().repeat(width - current_len);
            Ok(format!("{}{}", text, padding))
        }
    })?;
    globals.set("pad_right", pad_right)?;

    // ================ Регулярные выражения ================

    // Проверяет соответствие строки регулярному выражению
    // local matches = regex_match("hello123", "\\d+")
    // Возвращает: boolean (true если найдено соответствие)
    let regex_match = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.is_match(&text)),
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_match", regex_match)?;

    // Находит первое соответствие регулярному выражению
    // local found = regex_find("hello123world", "\\d+")
    // Возвращает: string | nil ("123" или nil если не найдено)
    let regex_find = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => {
                if let Some(m) = re.find(&text) {
                    Ok(Some(m.as_str().to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_find", regex_find)?;

    // Находит все соответствия регулярному выражению
    // local all_matches = regex_find_all("hello123world456", "\\d+")
    // Возвращает: table {"123", "456"}
    let regex_find_all = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => {
                let matches: Vec<String> = re
                    .find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect();
                Ok(matches)
            }
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_find_all", regex_find_all)?;

    // Заменяет все соответствия регулярному выражению
    // local result = regex_replace("hello123world456", "\\d+", "XXX")
    // Возвращает: string ("helloXXXworldXXX")
    let regex_replace = lua.create_function(|_, (text, pattern, replacement): (String, String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.replace_all(&text, replacement.as_str()).to_string()),
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_replace", regex_replace)?;

    // Извлекает группы захвата из регулярного выражения
    // local captures = regex_capture("name: John, age: 25", "name: (\\w+), age: (\\d+)")
    // Возвращает: table | nil {["0"] = "name: John, age: 25", ["1"] = "John", ["2"] = "25"}
    let regex_capture = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => {
                if let Some(caps) = re.captures(&text) {
                    let mut result = HashMap::new();
                    for (i, cap) in caps.iter().enumerate() {
                        if let Some(m) = cap {
                            result.insert(i.to_string(), m.as_str().to_string());
                        }
                    }
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_capture", regex_capture)?;

    // ================ Анализ текста ================

    // Подсчитывает количество слов в тексте
    // local count = word_count("Hello world from Rust")
    // Возвращает: number (4)
    let word_count = lua.create_function(|_, text: String| {
        let count = text.split_whitespace().count();
        Ok(count)
    })?;
    globals.set("word_count", word_count)?;

    // Подсчитывает количество строк в тексте
    // local count = line_count("line1\nline2\nline3")
    // Возвращает: number (3)
    let line_count = lua.create_function(|_, text: String| {
        let count = text.lines().count();
        Ok(count)
    })?;
    globals.set("line_count", line_count)?;

    // Подсчитывает количество символов (Unicode-aware)
    // local count = char_count("привет мир")
    // Возвращает: number (10)
    let char_count = lua.create_function(|_, text: String| Ok(text.chars().count()))?;
    globals.set("char_count", char_count)?;

    // Проверяет содержит ли строка подстроку
    // local has_substr = contains("hello world", "world")
    // Возвращает: boolean (true)
    let contains = lua.create_function(|_, (text, substring): (String, String)| {
        Ok(text.contains(&substring))
    })?;
    globals.set("contains", contains)?;

    // Проверяет начинается ли строка с префикса
    // local starts = starts_with("hello world", "hello")
    // Возвращает: boolean (true)
    let starts_with = lua.create_function(|_, (text, prefix): (String, String)| {
        Ok(text.starts_with(&prefix))
    })?;
    globals.set("starts_with", starts_with)?;

    // Проверяет заканчивается ли строка суффиксом
    // local ends = ends_with("hello world", "world")
    // Возвращает: boolean (true)
    let ends_with = lua.create_function(|_, (text, suffix): (String, String)| {
        Ok(text.ends_with(&suffix))
    })?;
    globals.set("ends_with", ends_with)?;

    // ================ Преобразование текста ================

    // Переворачивает строку задом наперед
    // local reversed = reverse("hello")
    // Возвращает: string ("olleh")
    let reverse = lua.create_function(|_, text: String| {
        Ok(text.chars().rev().collect::<String>())
    })?;
    globals.set("reverse", reverse)?;

    // Повторяет строку заданное количество раз
    // local repeated = repeat_text("abc", 3)
    // Возвращает: string ("abcabcabc")
    let repeat_text = lua.create_function(|_, (text, count): (String, usize)| {
        Ok(text.repeat(count))
    })?;
    globals.set("repeat_text", repeat_text)?;

    // Удаляет все пробельные символы из строки
    // local no_spaces = remove_whitespace("h e l l o")
    // Возвращает: string ("hello")
    let remove_whitespace = lua.create_function(|_, text: String| {
        Ok(text.chars().filter(|c| !c.is_whitespace()).collect::<String>())
    })?;
    globals.set("remove_whitespace", remove_whitespace)?;

    // Нормализует пробелы (схлопывает множественные пробелы в один)
    // local normalized = normalize_whitespace("hello    world  \n\t test")
    // Возвращает: string ("hello world test")
    let normalize_whitespace = lua.create_function(|_, text: String| {
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(normalized)
    })?;
    globals.set("normalize_whitespace", normalize_whitespace)?;

    // ================ Манипуляции со строками ================

    // Извлекает подстроку начиная с позиции (0-индексация)
    // local substr = substring("hello world", 6, 5)
    // Возвращает: string ("world")
    let substring = lua.create_function(|_, (text, start, length): (String, usize, Option<usize>)| {
        let chars: Vec<char> = text.chars().collect();
        let end = length
            .map(|len| (start + len).min(chars.len()))
            .unwrap_or(chars.len());

        if start >= chars.len() {
            return Ok(String::new());
        }

        Ok(chars[start..end].iter().collect())
    })?;
    globals.set("substring", substring)?;

    // Находит индекс первого вхождения подстроки
    // local index = find_index("hello world", "world")
    // Возвращает: number | nil (6 или nil если не найдено)
    let find_index = lua.create_function(|_, (text, substring): (String, String)| {
        match text.find(&substring) {
            Some(index) => Ok(Some(index)),
            None => Ok(None),
        }
    })?;
    globals.set("find_index", find_index)?;

    // Заменяет все вхождения одной подстроки на другую
    // local result = replace_text("hello world world", "world", "Rust")
    // Возвращает: string ("hello Rust Rust")
    let replace_text = lua.create_function(|_, (text, from, to): (String, String, String)| {
        Ok(text.replace(&from, &to))
    })?;
    globals.set("replace_text", replace_text)?;

    Ok(())
}