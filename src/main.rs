use anyhow::Result;
use clap::{ArgAction, Parser};
use once_cell::sync::Lazy;
use std::process;
use std::sync::Mutex;
use std::{thread, time};

pub mod executor;
pub mod func;
pub mod sqlitedb;
use func::{add, align, cancel, clean, exec, grep, list, remove};

static SQLITE_DB: &str = "lucq.sql";
static USER_QUIT_OP: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Linux user command queue
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run mode (cli or exec)
    #[arg(short, long, value_name = "mode", default_value = "cli")]
    mode: String,

    /// Add one command
    #[arg(short, long, value_name = "job", default_value = "null")]
    add: String,

    /// Add one command before <id>
    #[arg(long, value_name = "id", default_value_t = -1)]
    before: i32,

    /// Add one command after <id>
    #[arg(long, value_name = "id", default_value_t = -1)]
    after: i32,

    /// Remove command(s) (example: 1 or 1-5)
    #[arg(short, long, value_name = "id(s)", default_value = "null")]
    remove: String,

    /// Cancel command(s) (keep it in history but not run, example: 1 or 1-5)
    #[arg(long, value_name = "id(s)", default_value = "null")]
    cancel: String,

    /// Executor path (example: /usr/bin/python3)
    #[arg(short, long, value_name = "path", default_value = "null")]
    executor: String,

    /// Search and show
    #[arg(short, long, value_name = "name", default_value = "null")]
    grep: String,

    /// List all commands
    #[arg(short, long, action(ArgAction::SetTrue))]
    list: bool,

    /// Clean database
    #[arg(short, long, action(ArgAction::SetTrue))]
    clean: bool,

    /// Align database
    #[arg(long, action(ArgAction::SetTrue))]
    align: bool,

    /// Do not use emoji
    #[arg(long, action(ArgAction::SetTrue))]
    noemoji: bool,
}

fn user_quit() -> bool {
    let dur = time::Duration::from_secs_f32(0.5);
    thread::sleep(dur);
    println!("<<< Quit? [y/n]");
    let mut user_input = String::new();
    let _ = std::io::stdin().read_line(&mut user_input).unwrap();
    let ui = user_input.trim().to_string();

    match ui.as_str() {
        "Y" | "y" | "Yes" | "YES" | "yes" | "Q" | "q" => true,
        _ => false,
    }
}

fn main() -> Result<()> {
    // handle ctrl-c
    ctrlc::set_handler(move || {
        *USER_QUIT_OP.lock().unwrap() = true;
        match user_quit() {
            true => process::exit(0),
            _ => {
                println!(">>> Continue running...");
                *USER_QUIT_OP.lock().unwrap() = false;
            }
        }
    })
    .expect("error setting Ctrl-C handler");

    let args = Args::parse();
    if args.clean {
        clean()?;
    } else if args.mode == "cli" {
        if args.add != "null" {
            add(&args.add, &args.executor, args.before, args.after)?;
        } else if args.remove != "null" {
            remove(&args.remove)?;
        } else if args.cancel != "null" {
            cancel(&args.cancel)?;
        } else if args.grep != "null" {
            grep(&args.grep, args.noemoji)?;
        } else if args.list {
            list(args.noemoji)?;
        } else if args.align {
            align()?;
        }
    } else if args.mode == "exec" {
        println!(">>> Running...");
        exec()?;
    }
    Ok(())
}
