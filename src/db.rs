use home::home_dir;
use rusqlite::{Connection, Result};

use crate::SQLITE_DB;

#[derive(Debug)]
pub struct Commands {
    pub id: i32,
    pub user: String,
    pub command: String,
    pub add_time: i64, // UTC timestamp
    pub status: i32,   // 1 finish, 0 not finish, 9 running
    pub used_time: i64,
}

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Database> {
        let home = home_dir().unwrap();
        let sqlite_file_path = format!("{}/{}", home.to_string_lossy(), SQLITE_DB);
        // println!("{}", sqlite_file_path);
        let conn = Connection::open(sqlite_file_path)?;
        // let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                    id          INTEGER PRIMARY KEY,
                    user        TEXT NOT NULL,
                    command     TEXT NOT NULL,
                    add_time    INTEGER,
                    status      INTEGER,
                    used_time   INTEGER
                )",
            (), // empty list of parameters.
        )?;
        Ok(Database { conn })
    }
    pub fn insert(&self, user: &str, command: &str, add_time: i64) -> Result<()> {
        let cm = Commands {
            id: 0,
            user: user.to_string(),
            command: command.to_string(),
            add_time,
            status: 0,
            used_time: -1,
        };
        self.conn.execute(
            "INSERT INTO commands (user, command, add_time, status, used_time) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&cm.user, &cm.command, &cm.add_time, &cm.status, &cm.used_time),
        )?;
        Ok(())
    }
    pub fn remove(&self, id: i32) -> Result<()> {
        self.conn
            .execute(&format!("DELETE FROM commands WHERE id={}", id), ())?;
        Ok(())
    }
    pub fn select(&self) -> Result<Vec<Commands>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, user, command, add_time, status, used_time FROM commands")?;

        let commands_iter = stmt.query_map([], |row| {
            Ok(Commands {
                id: row.get(0)?,
                user: row.get(1)?,
                command: row.get(2)?,
                add_time: row.get(3)?,
                status: row.get(4)?,
                used_time: row.get(5)?,
            })
        })?;

        let mut ret: Vec<Commands> = Vec::new();
        for command in commands_iter {
            match command {
                Ok(c) => ret.push(c),
                Err(e) => return Err(e),
            }
        }

        Ok(ret)
    }
    pub fn select_not_finish(&self) -> Result<Vec<Commands>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user, command, add_time, status, used_time FROM commands WHERE status=0",
        )?;

        let commands_iter = stmt.query_map([], |row| {
            Ok(Commands {
                id: row.get(0)?,
                user: row.get(1)?,
                command: row.get(2)?,
                add_time: row.get(3)?,
                status: row.get(4)?,
                used_time: row.get(5)?,
            })
        })?;

        let mut ret: Vec<Commands> = Vec::new();
        for command in commands_iter {
            match command {
                Ok(c) => ret.push(c),
                Err(e) => return Err(e),
            }
        }

        Ok(ret)
    }
    pub fn update_command_running(&self, id: i32) -> Result<()> {
        let stmt = format!("UPDATE commands SET status=9 WHERE id={}", id);
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_command_finish(&self, id: i32) -> Result<()> {
        let stmt = format!("UPDATE commands SET status=1 WHERE id={}", id);
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_command_used_time(&self, id: i32, used_time: i64) -> Result<()> {
        let stmt = format!(
            "UPDATE commands SET used_time={} WHERE id={}",
            used_time, id
        );
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
}