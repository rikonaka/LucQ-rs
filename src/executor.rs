use anyhow::Result;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

enum CommandType {
    Unsupport,
    Shell,
    Python,
    Command,
    Binary,
}

pub enum ExecutorExitCode {
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

fn get_exec_path(file_extension: &str) -> String {
    let linux_command = Command::new("which")
        .arg(file_extension)
        .output()
        .expect("failed to execute process");
    let user = String::from_utf8_lossy(&linux_command.stdout);
    user.trim().to_string()
}

pub struct Executor {
    command: String,
    executor: String,
}

impl Executor {
    pub fn new(command: &str, executor: &str) -> Executor {
        let command = command.to_string();
        let executor = executor.to_string();
        Executor { command, executor }
    }
    pub fn exec(&self) -> Result<ExecutorExitCode> {
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
