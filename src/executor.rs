use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use home::home_dir;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::{thread, time};

use crate::sqlitedb::SqliteDB;
use crate::SQLITE_DB;
use crate::USER_QUIT_OP;

enum CommandType {
    Unsupport,
    Shell,
    Python,
    ShellCommand,
    Binary,
}

enum ExecutorExitCode {
    Success,
    Error,
    Cancel,
    Unknown,
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
            CommandType::ShellCommand
        }
    }
    fn exec(&self) -> Result<ExecutorExitCode> {
        //           executor        file    parameters
        // example: /usr/bin/python3 test.py -a 1

        let command = &self.command;
        let executor = if self.executor == "null" {
            let ct = Executor::command_judge(command);
            match ct {
                CommandType::Shell => get_exec_path("bash"),
                CommandType::Python => get_exec_path("python3"),
                CommandType::ShellCommand | CommandType::Binary => command.to_string(),
                CommandType::Unsupport => command.to_string(),
            }
        } else {
            self.executor.to_string()
        };

        let mut cs: Vec<&str> = command.split(" ").collect();
        let status = if cs.len() > 0 {
            let file = cs[0];
            let mut args = Vec::new();
            for c in &mut cs[1..] {
                args.push(c.to_string());
            }
            let status = if executor != file {
                println!(">>> Run: {} {}", executor, command);
                Command::new(executor)
                    .arg(file)
                    .args(args)
                    .stderr(Stdio::null()) // do not show the ctrl-c error message
                    .status()?
            } else {
                println!(">>> Run: {}", command);
                Command::new(file)
                    .args(args)
                    .stderr(Stdio::null()) // do not show the ctrl-c error message
                    .status()?
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
                    // signal: 2 (SIGINT) => user ctrl-c
                    // exit status: 1 => program error
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
                        // println!("<<< Quit? [y/n]");
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

pub fn clean() {
    let home = home_dir().unwrap();
    let sqlite_file_path = format!("{}/{}", home.to_string_lossy(), SQLITE_DB);
    match fs::remove_file(sqlite_file_path) {
        _ => (),
    }
    println!("Clean database finish!");
}

pub fn add(command: &str, executor: &str) -> Result<()> {
    let add_time = Utc::now().timestamp();
    let db = SqliteDB::new()?;
    let user = get_username();
    db.insert(&user, &command, &executor, add_time)?;
    Ok(())
}

pub fn remove(remove_str: &str) -> Result<()> {
    let db = SqliteDB::new()?;
    let id: i32 = remove_str.parse().unwrap();
    db.remove(id)?;
    Ok(())
}

pub fn list() -> Result<()> {
    let db = SqliteDB::new()?;
    let rets = db.select()?;
    println!("S | Jobs");
    for r in rets {
        // add time convert
        let add_time = DateTime::from_timestamp(r.add_time, 0)
            .unwrap()
            .with_timezone(&Local);
        let start_time = DateTime::from_timestamp(r.start_time, 0)
            .unwrap()
            .with_timezone(&Local);

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

        if r.executor != "null" {
            println!(
                "{} | id[{}], add[{}], start[{}], used[{}], command[{}], executor[{}]",
                status,
                r.id,
                add_time.format("%m-%d %H:%M"),
                start_time.format("%m-%d %H:%M"),
                used_time,
                r.command,
                r.executor
            )
        } else {
            println!(
                "{} | id[{}], add[{}], start[{}], used[{}], command[{}]",
                status,
                r.id,
                add_time.format("%m-%d %H:%M"),
                start_time.format("%m-%d %H:%M"),
                used_time,
                r.command
            )
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
                db.update_command_running(r.id)?;
                let start_time = Utc::now().timestamp();
                db.update_command_start_time(r.id, start_time)?;
                match executor.exec() {
                    Ok(exit_code) => match exit_code {
                        ExecutorExitCode::Success | ExecutorExitCode::Unknown => {
                            db.update_command_finish(r.id)?
                        }
                        ExecutorExitCode::Error => db.update_command_error(r.id)?,
                        ExecutorExitCode::Cancel => db.update_command_cancel(r.id)?,
                    },
                    Err(e) => {
                        println!("Program error: {}", e);
                        db.update_command_error(r.id)?;
                    }
                }
                let finish_time = Utc::now().timestamp();
                db.update_command_finish_time(r.id, finish_time)?;
            }
        }
        thread::sleep(duration);
    }
}
