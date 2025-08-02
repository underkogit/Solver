use mlua::Lua;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::collections::HashMap;

pub fn setup_globals_process(lua: Lua, script_path: String, target: &Option<String>, verbose: bool) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ================ Process Execution ================
    let exec = lua.create_async_function(|_, command: String| async move {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .output()
                .await
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .output()
                .await
        };

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
            },
            Err(e) => Err(mlua::Error::external(format!("Failed to execute command: {}", e))),
        }
    })?;
    globals.set("exec", exec)?;

    let exec_silent = lua.create_async_function(|_, command: String| async move {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
        };

        match output {
            Ok(status) => Ok(status.success()),
            Err(e) => Err(mlua::Error::external(format!("Failed to execute command: {}", e))),
        }
    })?;
    globals.set("exec_silent", exec_silent)?;

    let exec_streaming = lua.create_async_function(|_, command: String| async move {
        let mut child = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        };

        match child {
            Ok(mut process) => {
                let stdout = process.stdout.take().unwrap();
                let stderr = process.stderr.take().unwrap();

                let stdout_reader = BufReader::new(stdout);
                let stderr_reader = BufReader::new(stderr);

                let mut stdout_lines = stdout_reader.lines();
                let mut stderr_lines = stderr_reader.lines();

                let mut all_output = Vec::new();

                loop {
                    tokio::select! {
                        line = stdout_lines.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    println!("{}", line);
                                    all_output.push(format!("[OUT] {}", line));
                                },
                                Ok(None) => break,
                                Err(e) => return Err(mlua::Error::external(format!("Stdout error: {}", e))),
                            }
                        },
                        line = stderr_lines.next_line() => {
                            match line {
                                Ok(Some(line)) => {
                                    eprintln!("{}", line);
                                    all_output.push(format!("[ERR] {}", line));
                                },
                                Ok(None) => break,
                                Err(e) => return Err(mlua::Error::external(format!("Stderr error: {}", e))),
                            }
                        },
                    }
                }

                let status = process.wait().await;
                match status {
                    Ok(exit_status) => {
                        let mut response = HashMap::new();
                        response.insert("exit_code".to_string(), exit_status.code().unwrap_or(-1));
                        response.insert("success".to_string(), if exit_status.success() { 1 } else { 0 });
                        response.insert("output".to_string(), all_output.join("\n").parse().unwrap());
                        Ok(response)
                    },
                    Err(e) => Err(mlua::Error::external(format!("Process wait error: {}", e))),
                }
            },
            Err(e) => Err(mlua::Error::external(format!("Failed to spawn process: {}", e))),
        }
    })?;
    globals.set("exec_streaming", exec_streaming)?;

    // ================ Utility Commands ================
    let which = lua.create_async_function(|_, program: String| async move {
        let command = if cfg!(target_os = "windows") {
            format!("where {}", program)
        } else {
            format!("which {}", program)
        };

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .output()
                .await
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .output()
                .await
        };

        match output {
            Ok(result) => {
                if result.status.success() {
                    let path = String::from_utf8_lossy(&result.stdout).trim().to_string();
                    Ok(if path.is_empty() { None } else { Some(path) })
                } else {
                    Ok(None)
                }
            },
            Err(_) => Ok(None),
        }
    })?;
    globals.set("which", which)?;

    let command_exists = lua.create_async_function(|_, program: String| async move {
        let command = if cfg!(target_os = "windows") {
            format!("where {}", program)
        } else {
            format!("which {}", program)
        };

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await
        };

        match output {
            Ok(status) => Ok(status.success()),
            Err(_) => Ok(false),
        }
    })?;
    globals.set("command_exists", command_exists)?;

    // ================ Environment Variables ================
    let get_env = lua.create_function(|_, var_name: String| {
        Ok(std::env::var(var_name).ok())
    })?;
    globals.set("get_env", get_env)?;

    let set_env = lua.create_function(|_, (var_name, value): (String, String)| unsafe {
        std::env::set_var(var_name, value);
        Ok(())
    })?;
    globals.set("set_env", set_env)?;

    // ================ Working Directory ================
    let get_cwd = lua.create_function(|_, ()| {
        match std::env::current_dir() {
            Ok(path) => Ok(Some(path.to_string_lossy().to_string())),
            Err(_) => Ok(None),
        }
    })?;
    globals.set("get_cwd", get_cwd)?;

    let set_cwd = lua.create_async_function(|_, path: String| async move {
        match std::env::set_current_dir(&path) {
            Ok(_) => Ok(true),
            Err(e) => Err(mlua::Error::external(format!("Failed to change directory: {}", e))),
        }
    })?;
    globals.set("set_cwd", set_cwd)?;

    // ================ Platform Detection ================
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

    let is_windows = lua.create_function(|_, ()| {
        Ok(cfg!(target_os = "windows"))
    })?;
    globals.set("is_windows", is_windows)?;

    let is_unix = lua.create_function(|_, ()| {
        Ok(cfg!(unix))
    })?;
    globals.set("is_unix", is_unix)?;

    // ================ Common Tools ================
    let git_clone = lua.create_async_function(|_, (repo_url, target_dir): (String, Option<String>)| async move {
        let command = if let Some(dir) = target_dir {
            format!("git clone {} {}", repo_url, dir)
        } else {
            format!("git clone {}", repo_url)
        };

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .output()
                .await
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .output()
                .await
        };

        match output {
            Ok(result) => Ok(result.status.success()),
            Err(e) => Err(mlua::Error::external(format!("Git clone failed: {}", e))),
        }
    })?;
    globals.set("git_clone", git_clone)?;

    let cargo_build = lua.create_async_function(|_, release_mode: Option<bool>| async move {
        let command = if release_mode.unwrap_or(false) {
            "cargo build --release"
        } else {
            "cargo build"
        };

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .output()
                .await
        } else {
            Command::new("sh")
                .args(["-c", command])
                .output()
                .await
        };

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
            },
            Err(e) => Err(mlua::Error::external(format!("Cargo build failed: {}", e))),
        }
    })?;
    globals.set("cargo_build", cargo_build)?;

    Ok(())
}