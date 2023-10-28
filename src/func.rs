use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use home::home_dir;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::{thread, time};

use crate::executor::{Executor, ExecutorExitCode};
use crate::sqlitedb::Commands;
use crate::sqlitedb::SqliteDB;
use crate::SQLITE_DB;
use crate::USER_QUIT_OP;

fn get_username() -> String {
    let linux_command = Command::new("whoami")
        .output()
        .expect("failed to execute process");
    let user = String::from_utf8_lossy(&linux_command.stdout);
    user.trim().to_string()
}

pub fn add(command: &str, executor: &str, before: i32, after: i32) -> Result<()> {
    let add_time = Utc::now().timestamp();
    let db = SqliteDB::new()?;
    let user = get_username();
    let command = if command.contains(".") {
        let command_split: Vec<&str> = command.split(" ").collect();
        let (script_file, new_command) = if command_split[0] != "python3"
            && command_split[0] != "python"
            && command_split[0] != "bash"
            && command_split[0] != "zsh"
            && command_split[0] != "fish"
        {
            (command_split[0], command.to_string())
        } else {
            if command_split.len() > 0 {
                (command_split[1], command_split[1..].join(" "))
            } else {
                panic!("wrong command: {}", command);
            }
        };
        let path = Path::new(script_file);
        if path.exists() {
            let current_dir = env::current_dir().unwrap();
            format!("{}/{}", current_dir.display(), new_command)
        } else {
            println!("Warning !!!");
            println!("File [{}] not exists!", script_file);
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
        db.move_jobs(&id_vec)?;
        db.insert_with_id(before, &user, &command, &executor, add_time)?;
    } else if before == -1 && after != -1 {
        let commands = db.select_after(after)?;
        let mut id_vec = Vec::new();
        for c in commands {
            id_vec.push(c.id);
        }
        db.move_jobs(&id_vec)?;
        db.insert_with_id(after + 1, &user, &command, &executor, add_time)?;
    } else {
        println!("Wrong parameters!")
    }
    Ok(())
}

pub fn delete(id_str: &str) -> Result<()> {
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

fn commands_show(rets: Vec<Commands>, noemoji: bool) {
    for r in rets {
        // status
        let status = if noemoji {
            if r.status == 0 {
                " x" // waitting
            } else if r.status == 1 {
                " o" // finish
            } else if r.status == 2 {
                " e" // error
            } else if r.status == 3 {
                " c" // cancel
            } else {
                " r" // running
            }
        } else {
            if r.status == 0 {
                // "x" // waitting
                "😐"
            } else if r.status == 1 {
                // "o" // finish
                "😁"
            } else if r.status == 2 {
                // "e" // error
                "😨"
            } else if r.status == 3 {
                // "c" // cancel
                "🤡"
            } else {
                // "r" // running
                "🥵"
            }
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
            let finish_time = DateTime::from_timestamp(r.finish_time, 0)
                .unwrap()
                .with_timezone(&Local);
            finish_time.format("%m-%d %H:%M").to_string()
        } else {
            String::from("00-00 00:00")
        };

        if r.executor != "null" {
            println!("{} | {} | {} | {}", status, r.id, r.command, r.executor,);
        } else {
            println!("{} | {} | {}", status, r.id, r.command,);
        }
        println!(
            "---| add({}) | start({}) | finish({}) | used({})",
            add_time_str, start_time_str, finish_time_str, used_time
        );
    }
}

pub fn list(noemoji: bool) -> Result<()> {
    let db = SqliteDB::new()?;
    let rets = db.select_all()?;
    // println!("S | Jobs");
    commands_show(rets, noemoji);
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

pub fn clean() -> Result<()> {
    let home = home_dir().unwrap();
    let sqlite_file_path = format!("{}/{}", home.to_string_lossy(), SQLITE_DB);
    match fs::remove_file(sqlite_file_path) {
        _ => (),
    }
    println!("Clean database finish!");
    Ok(())
}

pub fn align() -> Result<()> {
    let db = SqliteDB::new()?;
    // get all id
    let ret = db.select_all()?;
    let mut id_vec: Vec<i32> = Vec::new();
    for r in ret {
        id_vec.push(r.id);
    }
    db.align_id(&id_vec)?;
    Ok(())
}

pub fn grep(name: &str, noemoji: bool) -> Result<()> {
    let db = SqliteDB::new()?;
    let rets = db.select_grep(name)?;
    commands_show(rets, noemoji);
    Ok(())
}
