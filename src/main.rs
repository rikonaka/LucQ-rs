use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use home::home_dir;
use std::fs;
use std::process::Command;
use std::{thread, time};

pub mod db;
use db::Database;

static SQLITE_DB: &str = "lucq.sql";

/// Linux user command queue
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run mode (cli or exec)
    #[arg(short, long, default_value = "cli")]
    mode: String,

    /// Add one command
    #[arg(short, long, default_value = "null")]
    add: String,

    /// Remove one command
    #[arg(short, long, default_value = "null")]
    remove: String,

    /// Executor path (example: /usr/bin/python3)
    #[arg(short, long, default_value = "null")]
    executor: String,

    /// List all commands
    #[arg(short, long, action)]
    list: bool,

    /// Clean database
    #[arg(short, long, action)]
    clean: bool,
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

enum CommandType {
    Unsupport,
    Shell,
    Python,
    ShellCommand,
    Binary,
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
    fn exec(&self) -> Result<()> {
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
            // println!("{} {}", executor, file);
            let status = if executor != file {
                println!(">>> exec: {} {}", executor, command);
                Command::new(executor).arg(file).args(args).status()?
            } else {
                println!(">>> exec: {}", command);
                Command::new(file).args(args).status()?
            };
            Some(status)
        } else {
            None
        };

        match status {
            Some(status) => {
                if status.success() {
                    println!("<<<");
                } else {
                    println!("<<< error")
                }
            }
            _ => println!("<<<"),
        }

        Ok(())
    }
}

fn clean() {
    let home = home_dir().unwrap();
    let sqlite_file_path = format!("{}/{}", home.to_string_lossy(), SQLITE_DB);
    match fs::remove_file(sqlite_file_path) {
        _ => (),
    }
    println!("Clean database finish!");
}

fn add(command: &str, executor: &str) -> Result<()> {
    let add_time = Utc::now().timestamp();
    let db = Database::new()?;
    let user = get_username();
    db.insert(&user, &command, &executor, add_time)?;
    Ok(())
}

fn remove(remove_str: &str) -> Result<()> {
    let db = Database::new()?;
    let id: i32 = remove_str.parse().unwrap();
    db.remove(id)?;
    Ok(())
}

fn list() -> Result<()> {
    let db = Database::new()?;
    let rets = db.select()?;
    println!("S | Jobs");
    for r in rets {
        // add time convert
        let add_time = DateTime::from_timestamp(r.add_time, 0)
            .unwrap()
            .with_timezone(&Local);

        // status
        let status = if r.status == 0 {
            "x"
        } else if r.status == 1 {
            "o"
        } else if r.status == 2 {
            "e"
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
                "{} | id[{}], user[{}], add_time[{}], used_time[{}], command[{}], executor[{}]",
                status,
                r.id,
                r.user,
                add_time.format("%Y-%m-%d %H:%M:%S"),
                used_time,
                r.command,
                r.executor
            )
        } else {
            println!(
                "{} | id[{}], user[{}], add_time[{}], used_time[{}], command[{}]",
                status,
                r.id,
                r.user,
                add_time.format("%Y-%m-%d %H:%M:%S"),
                used_time,
                r.command
            )
        }
    }
    Ok(())
}

fn exec() -> Result<()> {
    let db = Database::new()?;
    let duration = time::Duration::from_secs_f32(1.0);
    loop {
        let rets = db.select_not_finish()?;
        for r in rets {
            // println!("{}", &r.command);
            let executor = Executor::new(&r.command, &r.executor);
            db.update_command_running(r.id)?;
            let start_time = Utc::now().timestamp();
            db.update_command_start_time(r.id, start_time)?;
            match executor.exec() {
                Ok(_) => db.update_command_finish(r.id)?,
                Err(e) => {
                    println!("{}", e);
                    db.update_command_error(r.id)?;
                }
            }
            let finish_time = Utc::now().timestamp();
            db.update_command_finish_time(r.id, finish_time)?;
        }
        thread::sleep(duration);
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.clean {
        clean();
    } else if args.mode == "cli" {
        if args.add != "null" {
            add(&args.add, &args.executor)?;
        } else if args.remove != "null" {
            remove(&args.remove)?;
        } else if args.list {
            list()?;
        }
    } else if args.mode == "exec" {
        exec()?;
    }
    Ok(())
}
