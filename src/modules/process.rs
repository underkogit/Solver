use mlua::{Error, Lua, Value};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;

pub fn setup_globals_process(
    lua: Lua,
    script_path: String,
    target: &Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ================ Выполнение команд с callback ================

    // Выполняет команду с построчной обработкой вывода
    // task_run("cargo build", function(line) print(line) end)
    // Возвращает: boolean (true если команда завершилась успешно)
    let task_run = lua.create_async_function(|_, (command, callback): (String, mlua::Function)| async move {
        let mut child = create_command(&command).spawn();

        match child {
            Ok(mut process) => {
                let stdout = process.stdout.take().unwrap();
                let stderr = process.stderr.take().unwrap();

                let stdout_reader = BufReader::new(stdout);
                let stderr_reader = BufReader::new(stderr);

                let mut stdout_lines = stdout_reader.lines();
                let mut stderr_lines = stderr_reader.lines();

                loop {
                    tokio::select! {
                        line = stdout_lines.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    callback.call_async::<mlua::Value>(line).await
                                        .map_err(|e| mlua::Error::external(format!("Callback error: {}", e)))?;
                                },
                                Ok(None) => break,
                                Err(e) => {
                                    let error_line = format!("[UTF-8 ERROR] {}", e);
                                    callback.call_async::<mlua::Value>(error_line).await
                                        .map_err(|e| mlua::Error::external(format!("Callback error: {}", e)))?;
                                },
                            }
                        },
                        line = stderr_lines.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    let error_line = format!("{}", line);
                                    callback.call_async::<mlua::Value>(error_line).await
                                        .map_err(|e| mlua::Error::external(format!("Callback error: {}", e)))?;
                                },
                                Ok(None) => {},
                                Err(e) => {
                                    let error_line = format!("[STDERR UTF-8 ERROR] {}", e);
                                    callback.call_async::<mlua::Value>(error_line).await
                                        .map_err(|e| mlua::Error::external(format!("Callback error: {}", e)))?;
                                },
                            }
                        },
                    }
                }

                let status = process.wait().await;
                match status {
                    Ok(exit_status) => {
                        let exit_message = format!("[EXIT] Process finished with code: {}", exit_status.code().unwrap_or(-1));
                        callback.call_async::<mlua::Value>(exit_message).await
                            .map_err(|e| mlua::Error::external(format!("Callback error: {}", e)))?;
                        Ok(exit_status.success())
                    },
                    Err(e) => Err(mlua::Error::external(format!("Process wait error: {}", e))),
                }
            },
            Err(e) => Err(mlua::Error::external(format!("Failed to spawn process: {}", e))),
        }
    })?;
    globals.set("task_run", task_run)?;

    // Выполняет команду с детальной информацией о прогрессе
    // task_with_progress("long_command", function(progress)
    //   print("Lines: " .. progress.processed_lines .. ", Speed: " .. progress.lines_per_second)
    // end)
    // Callback получает таблицу: {line, processed_lines, elapsed_seconds, lines_per_second}
    // Возвращает: boolean (успех выполнения)
    let task_with_progress = lua.create_async_function(|lua, (command, progress_callback): (String, mlua::Function)| async move {
        let mut child = create_command(&command).spawn();

        match child {
            Ok(mut process) => {
                let stdout = process.stdout.take().unwrap();
                let stderr = process.stderr.take().unwrap();

                let stdout_reader = BufReader::new(stdout);
                let stderr_reader = BufReader::new(stderr);

                let mut stdout_lines = stdout_reader.lines();
                let mut stderr_lines = stderr_reader.lines();

                let start_time = std::time::Instant::now();
                let mut processed_lines = 0;
                let mut accumulated_output = Vec::new();

                loop {
                    tokio::select! {
                        line = stdout_lines.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    processed_lines += 1;
                                    accumulated_output.push(line.clone());

                                    let elapsed = start_time.elapsed().as_secs();
                                    let progress = lua.create_table()?;
                                    progress.set("line", line)?;
                                    progress.set("processed_lines", processed_lines)?;
                                    progress.set("elapsed_seconds", elapsed)?;
                                    progress.set("lines_per_second",
                                        if elapsed > 0 { processed_lines as f64 / elapsed as f64 }
                                        else { 0.0 })?;

                                    progress_callback.call_async::<mlua::Value>(progress).await
                                        .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))?;
                                },
                                Ok(None) => break,
                                Err(e) => {
                                    let error_progress = lua.create_table()?;
                                    error_progress.set("error", format!("UTF-8 decode error: {}", e))?;
                                    error_progress.set("processed_lines", processed_lines)?;

                                    progress_callback.call_async::<mlua::Value>(error_progress).await
                                        .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))?;
                                },
                            }
                        },
                        line = stderr_lines.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    let error_progress = lua.create_table()?;
                                    error_progress.set("error", line)?;
                                    error_progress.set("processed_lines", processed_lines)?;

                                    progress_callback.call_async::<mlua::Value>(error_progress).await
                                        .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))?;
                                },
                                Ok(None) => {},
                                Err(e) => {
                                    let error_progress = lua.create_table()?;
                                    error_progress.set("error", format!("Stderr UTF-8 decode error: {}", e))?;
                                    error_progress.set("processed_lines", processed_lines)?;

                                    progress_callback.call_async::<mlua::Value>(error_progress).await
                                        .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))?;
                                },
                            }
                        },
                    }
                }

                let status = process.wait().await;
                match status {
                    Ok(exit_status) => {
                        let total_time = start_time.elapsed().as_secs();
                        let final_result = lua.create_table()?;
                        final_result.set("success", exit_status.success())?;
                        final_result.set("exit_code", exit_status.code().unwrap_or(-1))?;
                        final_result.set("total_lines", processed_lines)?;
                        final_result.set("total_time", total_time)?;
                        final_result.set("final_output", accumulated_output.join("\n"))?;

                        progress_callback.call_async::<mlua::Value>(final_result).await
                            .map_err(|e| mlua::Error::external(format!("Final callback error: {}", e)))?;

                        Ok(exit_status.success())
                    },
                    Err(e) => Err(mlua::Error::external(format!("Process wait error: {}", e))),
                }
            },
            Err(e) => Err(mlua::Error::external(format!("Failed to spawn process: {}", e))),
        }
    })?;
    globals.set("task_with_progress", task_with_progress)?;

    // ================ Real-time выполнение команд ================

    // Выполняет команду в реальном времени (для ping, tail и т.д.)
    // task_realtime("ping google.com", function(data)
    //   if data.line then print(data.line) end
    // end)
    // Callback получает данные немедленно по мере поступления
    // Возвращает: boolean (успех выполнения)
    let task_realtime = lua.create_async_function(|lua, (command, progress_callback): (String, mlua::Function)| async move {
        let mut child = create_command(&command).spawn();

        match child {
            Ok(mut process) => {
                let mut stdout = process.stdout.take().unwrap();
                let mut stderr = process.stderr.take().unwrap();

                let start_time = std::time::Instant::now();
                let mut processed_lines = 0;
                let mut accumulated_output = Vec::new();
                let mut stdout_buffer = Vec::new();
                let mut stderr_buffer = Vec::new();
                let mut stdout_finished = false;
                let mut stderr_finished = false;

                loop {
                    let mut stdout_chunk = [0u8; 1024];
                    let mut stderr_chunk = [0u8; 1024];

                    tokio::select! {
                        result = stdout.read(&mut stdout_chunk), if !stdout_finished => {
                            match result {
                                Ok(0) => {
                                    stdout_finished = true;
                                    if !stdout_buffer.is_empty() {
                                        process_buffer_line(&lua, &mut stdout_buffer, &mut processed_lines,
                                                          &mut accumulated_output, start_time, &progress_callback).await?;
                                    }
                                },
                                Ok(n) => {
                                    stdout_buffer.extend_from_slice(&stdout_chunk[0..n]);
                                    process_buffer_lines(&lua, &mut stdout_buffer, &mut processed_lines,
                                                       &mut accumulated_output, start_time, &progress_callback).await?;
                                },
                                Err(e) => {
                                    send_error_progress(&lua, &progress_callback, format!("Stdout read error: {}", e), processed_lines).await?;
                                    stdout_finished = true;
                                },
                            }
                        },
                        result = stderr.read(&mut stderr_chunk), if !stderr_finished => {
                            match result {
                                Ok(0) => {
                                    stderr_finished = true;
                                    if !stderr_buffer.is_empty() {
                                        send_error_progress(&lua, &progress_callback,
                                                          String::from_utf8_lossy(&stderr_buffer).trim().to_string(),
                                                          processed_lines).await?;
                                        stderr_buffer.clear();
                                    }
                                },
                                Ok(n) => {
                                    stderr_buffer.extend_from_slice(&stderr_chunk[0..n]);
                                    process_stderr_lines(&lua, &mut stderr_buffer, &progress_callback, processed_lines).await?;
                                },
                                Err(e) => {
                                    send_error_progress(&lua, &progress_callback, format!("Stderr read error: {}", e), processed_lines).await?;
                                    stderr_finished = true;
                                },
                            }
                        },
                        else => break,
                    }

                    if stdout_finished && stderr_finished {
                        break;
                    }
                }

                finalize_process(&lua, process, &progress_callback, processed_lines,
                                 start_time, accumulated_output).await
            },
            Err(e) => Err(mlua::Error::external(format!("Failed to spawn process: {}", e))),
        }
    })?;
    globals.set("task_realtime", task_realtime)?;

    // ================ Переменные окружения ================

    // Получает значение переменной окружения
    // local path = get_env("PATH")
    // Возвращает: string | nil (значение переменной или nil)
    let get_env = lua.create_function(|_, var_name: String| Ok(std::env::var(var_name).ok()))?;
    globals.set("get_env", get_env)?;

    // Устанавливает переменную окружения
    // set_env("RUST_LOG", "debug")
    // Результат: устанавливает переменную для текущего процесса
    let set_env = lua.create_function(|_, (var_name, value): (String, String)| unsafe {
        std::env::set_var(var_name, value);
        Ok(())
    })?;
    globals.set("set_env", set_env)?;

    // ================ Рабочая директория ================

    // Получает текущую рабочую директорию
    // local current_dir = get_cwd()
    // Возвращает: string | nil (путь к текущей директории)
    let get_cwd = lua.create_function(|_, ()| match std::env::current_dir() {
        Ok(path) => Ok(Some(path.to_string_lossy().to_string())),
        Err(_) => Ok(None),
    })?;
    globals.set("get_cwd", get_cwd)?;

    // Изменяет текущую рабочую директорию
    // set_cwd("/home/user/project")
    // Возвращает: boolean (true если успешно)
    let set_cwd = lua.create_async_function(|_, path: String| async move {
        std::env::set_current_dir(&path)
            .map(|_| true)
            .map_err(|e| mlua::Error::external(format!("Failed to change directory: {}", e)))
    })?;
    globals.set("set_cwd", set_cwd)?;

    // ================ Определение платформы ================

    // Возвращает название операционной системы
    // local os = get_platform()
    // Возвращает: string ("windows", "macos", "linux", "unknown")
    let get_platform = lua.create_function(|_, ()| {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        };
        Ok(os.to_string())
    })?;
    globals.set("get_platform", get_platform)?;

    // Проверяет, запущено ли на Windows
    // if is_windows() then print("Running on Windows") end
    // Возвращает: boolean
    let is_windows = lua.create_function(|_, ()| Ok(cfg!(target_os = "windows")))?;
    globals.set("is_windows", is_windows)?;

    // Проверяет, запущено ли на Unix-системе
    // if is_unix() then print("Running on Unix") end
    // Возвращает: boolean
    let is_unix = lua.create_function(|_, ()| Ok(cfg!(unix)))?;
    globals.set("is_unix", is_unix)?;

    // ================ Cargo команды ================

    // Выполняет cargo build с опциональным release режимом
    // local result = cargo_build(true) -- release build
    // Возвращает: таблица {exit_code, stdout, stderr, success}
    let cargo_build = lua.create_async_function(|_, release_mode: Option<bool>| async move {
        let command = if release_mode.unwrap_or(false) {
            "cargo build --release"
        } else {
            "cargo build"
        };

        let output = create_command(command).output().await;

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout).to_string();
                let stderr = String::from_utf8_lossy(&result.stderr).to_string();

                let mut response = HashMap::new();
                response.insert("exit_code".to_string(), result.status.code().unwrap_or(-1));
                response.insert("stdout".to_string(), stdout.parse().unwrap());
                response.insert("stderr".to_string(), stderr.parse().unwrap());
                response.insert("success".to_string(), if result.status.success() { 1 } else { 0 });

                Ok(response)
            }
            Err(e) => Err(mlua::Error::external(format!("Cargo build failed: {}", e))),
        }
    })?;
    globals.set("cargo_build", cargo_build)?;

    Ok(())
}

// ================ Вспомогательные функции ================

fn create_command(command: &str) -> Command {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }
}

async fn process_buffer_line(
    lua: &Lua,
    buffer: &mut Vec<u8>,
    processed_lines: &mut i32,
    accumulated_output: &mut Vec<String>,
    start_time: std::time::Instant,
    progress_callback: &mlua::Function,
) -> Result<(), mlua::Error> {
    let line = String::from_utf8_lossy(buffer).trim().to_string();
    if !line.is_empty() {
        *processed_lines += 1;
        accumulated_output.push(line.clone());

        let elapsed = start_time.elapsed().as_secs();
        let progress = lua.create_table()?;
        progress.set("line", line)?;
        progress.set("processed_lines", *processed_lines)?;
        progress.set("elapsed_seconds", elapsed)?;
        progress.set("lines_per_second",
                     if elapsed > 0 { *processed_lines as f64 / elapsed as f64 } else { 0.0 })?;

        progress_callback.call_async::<mlua::Value>(progress).await
            .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))?;
    }
    buffer.clear();
    Ok(())
}

async fn process_buffer_lines(
    lua: &Lua,
    buffer: &mut Vec<u8>,
    processed_lines: &mut i32,
    accumulated_output: &mut Vec<String>,
    start_time: std::time::Instant,
    progress_callback: &mlua::Function,
) -> Result<(), mlua::Error> {
    while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n' || b == b'\r') {
        let line_bytes = buffer.drain(0..=newline_pos).collect::<Vec<u8>>();
        let line = String::from_utf8_lossy(&line_bytes[0..line_bytes.len()-1]).trim().to_string();

        if !line.is_empty() {
            *processed_lines += 1;
            accumulated_output.push(line.clone());

            let elapsed = start_time.elapsed().as_secs();
            let progress = lua.create_table()?;
            progress.set("line", line)?;
            progress.set("processed_lines", *processed_lines)?;
            progress.set("elapsed_seconds", elapsed)?;
            progress.set("lines_per_second",
                         if elapsed > 0 { *processed_lines as f64 / elapsed as f64 } else { 0.0 })?;

            progress_callback.call_async::<mlua::Value>(progress).await
                .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))?;
        }
    }
    Ok(())
}

async fn process_stderr_lines(
    lua: &Lua,
    buffer: &mut Vec<u8>,
    progress_callback: &mlua::Function,
    processed_lines: i32,
) -> Result<(), mlua::Error> {
    while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n' || b == b'\r') {
        let line_bytes = buffer.drain(0..=newline_pos).collect::<Vec<u8>>();
        let line = String::from_utf8_lossy(&line_bytes[0..line_bytes.len()-1]).trim().to_string();

        if !line.is_empty() {
            send_error_progress(lua, progress_callback, line, processed_lines).await?;
        }
    }
    Ok(())
}

async fn send_error_progress(
    lua: &Lua,
    progress_callback: &mlua::Function,
    error_msg: String,
    processed_lines: i32,
) -> Result<Value, Error> {
    let error_progress = lua.create_table()?;
    error_progress.set("error", error_msg)?;
    error_progress.set("processed_lines", processed_lines)?;

    progress_callback.call_async::<mlua::Value>(error_progress).await
        .map_err(|e| mlua::Error::external(format!("Progress callback error: {}", e)))
}

async fn finalize_process(
    lua: &Lua,
    mut process: tokio::process::Child,
    progress_callback: &mlua::Function,
    processed_lines: i32,
    start_time: std::time::Instant,
    accumulated_output: Vec<String>,
) -> Result<bool, mlua::Error> {
    let status = process.wait().await;
    match status {
        Ok(exit_status) => {
            let total_time = start_time.elapsed().as_secs();
            let final_result = lua.create_table()?;
            final_result.set("success", exit_status.success())?;
            final_result.set("exit_code", exit_status.code().unwrap_or(-1))?;
            final_result.set("total_lines", processed_lines)?;
            final_result.set("total_time", total_time)?;
            final_result.set("final_output", accumulated_output.join("\n"))?;

            progress_callback.call_async::<mlua::Value>(final_result).await
                .map_err(|e| mlua::Error::external(format!("Final callback error: {}", e)))?;

            Ok(exit_status.success())
        },
        Err(e) => Err(mlua::Error::external(format!("Process wait error: {}", e))),
    }
}