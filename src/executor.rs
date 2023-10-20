use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use home::home_dir;
use std::env;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;
use std::{thread, time};

use crate::sqlitedb::SqliteDB;
use crate::SQLITE_DB;
use crate::USER_QUIT_OP;

enum CommandType {
    Unsupport,
    Shell,
    Python,
    Command,
    Binary,
}

enum ExecutorExitCode {
    Success,
    Error,
    Cancel,
    Unknown,
}

fn command_judge(command: &str) -> CommandType {
    let cs: Vec<&str> = command.split(" ").collect();
    let mut contain_dot = false;
    for c in cs {
        if c.contains(".") {
            contain_dot = true;
        }

        if c.contains(".sh") {
            return CommandType::Shell;
        } else if c.contains(".py") {
            return CommandType::Python;
        } else if c.contains(".o") | c.contains(".exe") {
            return CommandType::Binary;
        }
    }
    if contain_dot {
        CommandType::Unsupport
    } else {
        CommandType::Command
    }
}

fn get_username() -> String {
    let linux_command = Command::new("whoami")
        .output()
        .expect("failed to execute process");
    let user = String::from_utf8_lossy(&linux_command.stdout);
    user.trim().to_string()
}

fn get_exec_path(file_extension: &str) -> String {
    let linux_command = Command::new("which")
        .arg(file_extension)
        .output()
        .expect("failed to execute process");
    let user = String::from_utf8_lossy(&linux_command.stdout);
    user.trim().to_string()
}

struct Executor {
    command: String,
    executor: String,
}

impl Executor {
    fn new(command: &str, executor: &str) -> Executor {
        let command = command.to_string();
        let executor = executor.to_string();
        Executor { command, executor }
    }
    fn exec(&self) -> Result<ExecutorExitCode> {
        //           executor        file    parameters
        // example: /usr/bin/python3 test.py -a 1

        let command = &self.command;
        let executor = if self.executor == "null" {
            let command_type = command_judge(command);
            match command_type {
                CommandType::Shell => get_exec_path("bash"),
                CommandType::Python => get_exec_path("python3"),
                CommandType::Command | CommandType::Binary => command.to_string(),
                CommandType::Unsupport => command.to_string(),
            }
        } else {
            self.executor.to_string()
        };

        let mut command_split: Vec<&str> = command.split(" ").collect();
        let status = if command_split.len() > 0 {
            let file = command_split[0];
            let mut args = Vec::new();
            for c in &mut command_split[1..] {
                args.push(c.to_string());
            }
            let status = if executor != file {
                println!(">>> Run: {} {}", executor, command);
                Command::new(executor).arg(file).args(args).status()?
            } else {
                println!(">>> Run: {}", command);
                Command::new(file).args(args).status()?
            };
            Some(status)
        } else {
            None
        };

        let ret = match status {
            Some(status) => {
                if status.success() {
                    println!("<<<");
                    ExecutorExitCode::Success
                } else {
                    // exit status: 1 => program error
                    // signal: 2 (SIGINT) => user ctrl-c
                    let status_code = match status.code() {
                        Some(s) => s,
                        _ => match status.signal() {
                            Some(s) => s,
                            _ => 0,
                        },
                    };
                    if status_code == 1 {
                        println!("<<< Error");
                        ExecutorExitCode::Error
                    } else if status_code == 2 {
                        ExecutorExitCode::Cancel
                    } else {
                        ExecutorExitCode::Unknown
                    }
                }
            }
            _ => {
                println!("<<<");
                ExecutorExitCode::Unknown
            }
        };

        Ok(ret)
    }
}

pub fn clean() {
    let home = home_dir().unwrap();
    let sqlite_file_path = format!("{}/{}", home.to_string_lossy(), SQLITE_DB);
    match fs::remove_file(sqlite_file_path) {
        _ => (),
    }
    println!("Clean database finish!");
}

pub fn add(command: &str, executor: &str, before: i32, after: i32) -> Result<()> {
    let add_time = Utc::now().timestamp();
    let db = SqliteDB::new()?;
    let user = get_username();
    let command = if command.contains(".") {
        let command_split: Vec<&str> = command.split(" ").collect();
        let script_file = command_split[0];
        let path = Path::new(script_file);
        if path.exists() {
            let current_dir = env::current_dir().unwrap();
            format!("{}/{}", current_dir.display(), command)
        } else {
            println!("{} not exists!", command);
            return Ok(());
        }
    } else {
        command.to_string()
    };
    if before == -1 && after == -1 {
        db.insert(&user, &command, &executor, add_time)?;
    } else if before != -1 && after == -1 {
        let commands = db.select_after(before - 1)?;
        let mut id_vec = Vec::new();
        for c in commands {
            id_vec.push(c.id);
        }
        println!("aaa");
        db.move_jobs(id_vec)?;
        println!("bbb");
        db.insert_with_id(before, &user, &command, &executor, add_time)?;
    } else if before == -1 && after != -1 {
        let commands = db.select_after(after)?;
        let mut id_vec = Vec::new();
        for c in commands {
            id_vec.push(c.id);
        }
        db.move_jobs(id_vec)?;
        db.insert_with_id(after + 1, &user, &command, &executor, add_time)?;
    } else {
        println!("Wrong parameters!")
    }
    Ok(())
}

pub fn remove(id_str: &str) -> Result<()> {
    let db = SqliteDB::new()?;
    let mut success = true;
    if id_str.contains("-") {
        let id_split: Vec<&str> = id_str.split("-").collect();
        if id_split.len() == 2 {
            let start: i32 = id_split[0].parse().unwrap();
            let end: i32 = id_split[1].parse().unwrap();
            if start < end {
                for id in start..=end {
                    db.remove_by_id(id)?;
                }
            } else {
                success = false;
            }
        } else {
            success = false;
        }
    } else {
        let id: i32 = id_str.parse().unwrap();
        db.remove_by_id(id)?;
    }

    if success == false {
        println!("Please use a-b format!");
    }
    Ok(())
}

pub fn cancel(id_str: &str) -> Result<()> {
    let db = SqliteDB::new()?;
    let mut success = true;
    if id_str.contains("-") {
        let id_split: Vec<&str> = id_str.split("-").collect();
        if id_split.len() == 2 {
            let start: i32 = id_split[0].parse().unwrap();
            let end: i32 = id_split[1].parse().unwrap();
            if start < end {
                for id in start..=end {
                    db.update_status_cancel(id)?;
                }
            } else {
                success = false;
            }
        } else {
            success = false;
        }
    } else {
        let id: i32 = id_str.parse().unwrap();
        db.update_status_cancel(id)?;
    }

    if success == false {
        println!("Please use a-b format!");
    }
    Ok(())
}

pub fn list() -> Result<()> {
    let db = SqliteDB::new()?;
    let rets = db.select_all()?;
    println!("S | Jobs");
    for r in rets {
        // status
        let status = if r.status == 0 {
            "x"
        } else if r.status == 1 {
            "o"
        } else if r.status == 2 {
            "e"
        } else if r.status == 3 {
            "c"
        } else {
            "r"
        };

        // used time format
        let used_time = if r.start_time != -1 {
            let used_time = if r.finish_time != -1 {
                r.finish_time - r.start_time
            } else {
                Utc::now().timestamp() - r.start_time
            };

            let seconds = used_time % 60;
            let minutes = (used_time / 60) % 60;
            let hours = (used_time / 60) / 60;

            let convert = |input: i64| -> String {
                let result_str = if input < 10 {
                    format!("0{}", input)
                } else {
                    format!("{}", input)
                };
                result_str
            };

            let seconds_str = convert(seconds);
            let minutes_str = convert(minutes);
            let hours_str = convert(hours);
            let used_time = format!("{}:{}:{}", hours_str, minutes_str, seconds_str);
            used_time
        } else {
            String::from("00:00:00")
        };

        // add time convert
        let add_time_str = if r.add_time != -1 {
            let add_time = DateTime::from_timestamp(r.add_time, 0)
                .unwrap()
                .with_timezone(&Local);
            add_time.format("%m-%d %H:%M").to_string()
        } else {
            String::from("00-00 00:00")
        };

        let start_time_str = if r.start_time != -1 {
            let start_time = DateTime::from_timestamp(r.start_time, 0)
                .unwrap()
                .with_timezone(&Local);
            start_time.format("%m-%d %H:%M").to_string()
        } else {
            String::from("00-00 00:00")
        };

        let finish_time_str = if r.finish_time != -1 {
            let finish_time = DateTime::from_timestamp(r.add_time, 0)
                .unwrap()
                .with_timezone(&Local);
            finish_time.format("%m-%d %H:%M").to_string()
        } else {
            String::from("00-00 00:00")
        };

        println!(
            "{} | id[{}], add[{}], start[{}], finish[{}], used[{}]",
            status, r.id, add_time_str, start_time_str, finish_time_str, used_time,
        );
        if r.executor != "null" {
            println!("  | >>> command[{}], executor[{}]", r.command, r.executor);
        } else {
            println!("  | >>> command[{}]", r.command);
        }
    }
    Ok(())
}

pub fn exec() -> Result<()> {
    let db = SqliteDB::new()?;
    let duration = time::Duration::from_secs_f32(1.0);
    loop {
        let user_quit_op = *USER_QUIT_OP.lock().unwrap();

        // When user_quit_op is true,
        // mean the user is deciding quit the program or not,
        // so we do not run the job.
        if user_quit_op != true {
            let rets = db.select_not_finish()?;
            // rets == 1 if have job, == 0 if no job
            for r in rets {
                let executor = Executor::new(&r.command, &r.executor);
                db.update_status_running(r.id)?;
                let start_time = Utc::now().timestamp();
                db.update_start_time(r.id, start_time)?;
                match executor.exec() {
                    Ok(exit_code) => match exit_code {
                        ExecutorExitCode::Success | ExecutorExitCode::Unknown => {
                            db.update_status_finish(r.id)?
                        }
                        ExecutorExitCode::Error => db.update_status_error(r.id)?,
                        ExecutorExitCode::Cancel => db.update_status_cancel(r.id)?,
                    },
                    Err(e) => {
                        println!("Program error: {}", e);
                        db.update_status_error(r.id)?;
                    }
                }
                let finish_time = Utc::now().timestamp();
                db.update_finish_time(r.id, finish_time)?;
            }
        }
        thread::sleep(duration);
    }
}
