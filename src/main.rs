use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use home::home_dir;
use std::fs;
use std::process::Command;

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

enum ScriptType {
    Unsupport,
    Bash,
    Python,
    ShellCommand,
}

fn command_judge(command: &str) -> ScriptType {
    let cs: Vec<&str> = command.split(" ").collect();
    let mut contain_dot = false;
    for c in cs {
        if c.contains(".") {
            contain_dot = true;
        }

        if c.contains(".sh") {
            return ScriptType::Bash;
        } else if c.contains(".py") {
            return ScriptType::Python;
        }
    }
    if contain_dot {
        ScriptType::Unsupport
    } else {
        ScriptType::ShellCommand
    }
}

fn executor(command: &str) -> Result<()> {
    let st = command_judge(command);
    let exec = match st {
        ScriptType::Bash => get_exec_path("bash"),
        ScriptType::Python => get_exec_path("python3"),
        ScriptType::ShellCommand => command.to_string(),
        ScriptType::Unsupport => command.to_string(),
    };

    let status = match st {
        ScriptType::Bash | ScriptType::Python => {
            let mut cs: Vec<&str> = command.split(" ").collect();
            let mut args = Vec::new();
            if cs.len() > 0 {
                for c in &mut cs[1..] {
                    args.push(c.to_string());
                }
            }
            println!(">>> exec: {exec} {command}");
            let status = if cs.len() > 1 {
                Command::new(exec).arg(cs[0]).args(args).status()?
            } else {
                Command::new(exec).arg(cs[0]).status()?
            };
            Some(status)
        }
        ScriptType::ShellCommand => {
            let mut cs: Vec<&str> = command.split(" ").collect();
            let mut args = Vec::new();
            if cs.len() > 0 {
                for c in &mut cs[1..] {
                    args.push(c.to_string());
                }
            }
            println!(">>> exec: {command}");
            let status = if args.len() > 0 {
                Command::new(cs[0]).args(args).status()?
            } else {
                Command::new(cs[0]).status()?
            };
            Some(status)
        }
        ScriptType::Unsupport => {
            println!(">>> Unsupport file: {}", exec);
            None
        }
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

fn main() -> Result<()> {
    let args = Args::parse();
    if args.clean {
        let home = home_dir().unwrap();
        let sqlite_file_path = format!("{}/{}", home.to_string_lossy(), SQLITE_DB);
        match fs::remove_file(sqlite_file_path) {
            _ => (),
        }
        println!("Clean database finish!");
    } else if args.mode == "cli" {
        if args.add != "null" {
            // add new command
            let add_time = Utc::now().timestamp();
            let command = args.add.to_string();
            let db = Database::new()?;
            let user = get_username();
            db.insert(&user, &command, add_time)?;
        } else if args.remove != "null" {
            let db = Database::new()?;
            let id: i32 = args.remove.parse().unwrap();
            db.remove(id)?;
        } else if args.list {
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
                } else {
                    "r"
                };

                // used time format
                let used_time = if r.used_time != -1 {
                    let seconds = r.used_time % 60;
                    let minutes = (r.used_time / 60) % 60;
                    let hours = (r.used_time / 60) / 60;
                    let used_time = format!("{}:{}:{}", hours, minutes, seconds);
                    used_time
                } else {
                    String::from("0:0:0")
                };

                println!(
                    "{} | id[{}], user[{}], add_time[{}], used_time[{}], command[{}]",
                    status, r.id, r.user, add_time, used_time, r.command
                )
            }
        }
    } else if args.mode == "exec" {
        let db = Database::new()?;
        loop {
            let rets = db.select_not_finish()?;
            for r in rets {
                // println!("{}", &r.command);
                db.update_command_running(r.id)?;
                let start_time = Utc::now().timestamp();
                match executor(&r.command) {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                }
                let end_time = Utc::now().timestamp();
                db.update_command_used_time(r.id, end_time - start_time)?;
                db.update_command_finish(r.id)?;
            }
        }
    }
    Ok(())
}
