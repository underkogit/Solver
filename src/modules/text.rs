use mlua::Lua;
use regex::Regex;
use std::collections::HashMap;

pub fn setup_globals_text(lua: Lua, script_path: String, target: &Option<String>, verbose: bool) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ================ String Formatting ================
    let trim = lua.create_function(|_, text: String| {
        Ok(text.trim().to_string())
    })?;
    globals.set("trim", trim)?;

    let trim_start = lua.create_function(|_, text: String| {
        Ok(text.trim_start().to_string())
    })?;
    globals.set("trim_start", trim_start)?;

    let trim_end = lua.create_function(|_, text: String| {
        Ok(text.trim_end().to_string())
    })?;
    globals.set("trim_end", trim_end)?;

    let to_upper = lua.create_function(|_, text: String| {
        Ok(text.to_uppercase())
    })?;
    globals.set("to_upper", to_upper)?;

    let to_lower = lua.create_function(|_, text: String| {
        Ok(text.to_lowercase())
    })?;
    globals.set("to_lower", to_lower)?;

    let capitalize = lua.create_function(|_, text: String| {
        let mut chars: Vec<char> = text.chars().collect();
        if let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
        Ok(chars.into_iter().collect::<String>())
    })?;
    globals.set("capitalize", capitalize)?;

    let split = lua.create_function(|_, (text, delimiter): (String, String)| {
        let parts: Vec<String> = text.split(&delimiter).map(|s| s.to_string()).collect();
        Ok(parts)
    })?;
    globals.set("split", split)?;

    let join = lua.create_function(|_, (parts, delimiter): (Vec<String>, String)| {
        Ok(parts.join(&delimiter))
    })?;
    globals.set("join", join)?;

    let pad_left = lua.create_function(|_, (text, width, ch): (String, usize, Option<String>)| {
        let pad_char = ch.unwrap_or_else(|| " ".to_string()).chars().next().unwrap_or(' ');
        Ok(format!("{:>width$}", text, width = width).replace(' ', &pad_char.to_string()))
    })?;
    globals.set("pad_left", pad_left)?;

    let pad_right = lua.create_function(|_, (text, width, ch): (String, usize, Option<String>)| {
        let pad_char = ch.unwrap_or_else(|| " ".to_string()).chars().next().unwrap_or(' ');
        Ok(format!("{:<width$}", text, width = width).replace(' ', &pad_char.to_string()))
    })?;
    globals.set("pad_right", pad_right)?;

    // ================ Regular Expressions ================
    let regex_match = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.is_match(&text)),
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_match", regex_match)?;

    let regex_find = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => {
                if let Some(m) = re.find(&text) {
                    Ok(Some(m.as_str().to_string()))
                } else {
                    Ok(None)
                }
            },
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_find", regex_find)?;

    let regex_find_all = lua.create_function(|_, (text, pattern): (String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => {
                let matches: Vec<String> = re.find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect();
                Ok(matches)
            },
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_find_all", regex_find_all)?;

    let regex_replace = lua.create_function(|_, (text, pattern, replacement): (String, String, String)| {
        match Regex::new(&pattern) {
            Ok(re) => Ok(re.replace_all(&text, replacement.as_str()).to_string()),
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_replace", regex_replace)?;

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
            },
            Err(e) => Err(mlua::Error::external(format!("Invalid regex: {}", e))),
        }
    })?;
    globals.set("regex_capture", regex_capture)?;

    // ================ Text Analysis ================
    let word_count = lua.create_function(|_, text: String| {
        let count = text.split_whitespace().count();
        Ok(count)
    })?;
    globals.set("word_count", word_count)?;

    let line_count = lua.create_function(|_, text: String| {
        let count = text.lines().count();
        Ok(count)
    })?;
    globals.set("line_count", line_count)?;

    let char_count = lua.create_function(|_, text: String| {
        Ok(text.chars().count())
    })?;
    globals.set("char_count", char_count)?;

    let contains = lua.create_function(|_, (text, substring): (String, String)| {
        Ok(text.contains(&substring))
    })?;
    globals.set("contains", contains)?;

    let starts_with = lua.create_function(|_, (text, prefix): (String, String)| {
        Ok(text.starts_with(&prefix))
    })?;
    globals.set("starts_with", starts_with)?;

    let ends_with = lua.create_function(|_, (text, suffix): (String, String)| {
        Ok(text.ends_with(&suffix))
    })?;
    globals.set("ends_with", ends_with)?;

    // ================ Text Transformation ================
    let reverse = lua.create_function(|_, text: String| {
        Ok(text.chars().rev().collect::<String>())
    })?;
    globals.set("reverse", reverse)?;

    let repeat_text = lua.create_function(|_, (text, count): (String, usize)| {
        Ok(text.repeat(count))
    })?;
    globals.set("repeat_text", repeat_text)?;

    let remove_whitespace = lua.create_function(|_, text: String| {
        Ok(text.chars().filter(|c| !c.is_whitespace()).collect::<Vec<_>>())
    })?;
    globals.set("remove_whitespace", remove_whitespace)?;

    let normalize_whitespace = lua.create_function(|_, text: String| {
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(normalized)
    })?;
    globals.set("normalize_whitespace", normalize_whitespace)?;

    // ================ String Manipulation ================
    let substring = lua.create_function(|_, (text, start, length): (String, usize, Option<usize>)| {
        let chars: Vec<char> = text.chars().collect();
        let end = length.map(|len| (start + len).min(chars.len())).unwrap_or(chars.len());

        if start >= chars.len() {
            return Ok(String::new());
        }

        Ok(chars[start..end].iter().collect())
    })?;
    globals.set("substring", substring)?;

    let find_index = lua.create_function(|_, (text, substring): (String, String)| {
        match text.find(&substring) {
            Some(index) => Ok(Some(index)),
            None => Ok(None),
        }
    })?;
    globals.set("find_index", find_index)?;

    let replace_text = lua.create_function(|_, (text, from, to): (String, String, String)| {
        Ok(text.replace(&from, &to))
    })?;
    globals.set("replace_text", replace_text)?;

    Ok(())
}