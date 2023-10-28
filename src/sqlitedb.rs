use home::home_dir;
use rusqlite::{Connection, Result};
use std::iter::zip;

use crate::SQLITE_DB;

#[derive(Debug)]
pub struct Commands {
    pub id: i32,
    pub user: String,
    pub command: String,
    pub executor: String,
    pub add_time: i64, // UTC timestamp
    pub status: i32,   // 1 finish, 0 not finish, 2 error, 3 cancel, 9 running
    pub start_time: i64,
    pub finish_time: i64,
}

pub struct SqliteDB {
    pub conn: Connection,
}

impl SqliteDB {
    pub fn new() -> Result<SqliteDB> {
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
                    executor    TEXT NOT NULL,
                    add_time    INTEGER,
                    status      INTEGER,
                    start_time  INTEGER,
                    finish_time INTEGER
                )",
            (), // empty list of parameters.
        )?;
        Ok(SqliteDB { conn })
    }
    pub fn insert(&self, user: &str, command: &str, executor: &str, add_time: i64) -> Result<()> {
        let cm = Commands {
            id: 0,
            user: user.to_string(),
            command: command.to_string(),
            executor: executor.to_string(),
            add_time,
            status: 0,
            start_time: -1,
            finish_time: -1,
        };
        self.conn.execute(
            "INSERT INTO commands (user, command, executor, add_time, status, start_time, finish_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (&cm.user, &cm.command, &cm.executor, &cm.add_time, &cm.status, &cm.start_time, &cm.finish_time),
        )?;
        Ok(())
    }
    pub fn insert_with_id(
        &self,
        id: i32,
        user: &str,
        command: &str,
        executor: &str,
        add_time: i64,
    ) -> Result<()> {
        let cm = Commands {
            id,
            user: user.to_string(),
            command: command.to_string(),
            executor: executor.to_string(),
            add_time,
            status: 0,
            start_time: -1,
            finish_time: -1,
        };
        self.conn.execute(
            "INSERT INTO commands (id, user, command, executor, add_time, status, start_time, finish_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (&cm.id, &cm.user, &cm.command, &cm.executor, &cm.add_time, &cm.status, &cm.start_time, &cm.finish_time),
        )?;
        Ok(())
    }
    pub fn remove_by_id(&self, id: i32) -> Result<()> {
        self.conn
            .execute(&format!("DELETE FROM commands WHERE id={}", id), ())?;
        Ok(())
    }
    pub fn select_all(&self) -> Result<Vec<Commands>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user, command, executor, add_time, status, start_time, finish_time FROM commands ORDER BY id ASC",
        )?;

        let commands_iter = stmt.query_map([], |row| {
            Ok(Commands {
                id: row.get(0)?,
                user: row.get(1)?,
                command: row.get(2)?,
                executor: row.get(3)?,
                add_time: row.get(4)?,
                status: row.get(5)?,
                start_time: row.get(6)?,
                finish_time: row.get(7)?,
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
    pub fn select_after(&self, id: i32) -> Result<Vec<Commands>> {
        let s = format!("SELECT id, user, command, executor, add_time, status, start_time, finish_time FROM commands WHERE id>{} ORDER BY id ASC", id);
        let mut stmt = self.conn.prepare(&s)?;

        let commands_iter = stmt.query_map([], |row| {
            Ok(Commands {
                id: row.get(0)?,
                user: row.get(1)?,
                command: row.get(2)?,
                executor: row.get(3)?,
                add_time: row.get(4)?,
                status: row.get(5)?,
                start_time: row.get(6)?,
                finish_time: row.get(7)?,
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
            "SELECT id, user, command, executor, add_time, status, start_time, finish_time FROM commands WHERE status=0 ORDER BY id ASC LIMIT 1",
        )?;

        let commands_iter = stmt.query_map([], |row| {
            Ok(Commands {
                id: row.get(0)?,
                user: row.get(1)?,
                command: row.get(2)?,
                executor: row.get(3)?,
                add_time: row.get(4)?,
                status: row.get(5)?,
                start_time: row.get(7)?,
                finish_time: row.get(6)?,
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
    pub fn select_grep(&self, name: &str) -> Result<Vec<Commands>> {
        let s = format!("SELECT id, user, command, executor, add_time, status, start_time, finish_time FROM commands WHERE command LIKE '%{}%' ORDER BY id ASC", name);
        let mut stmt = self.conn.prepare(&s)?;

        let commands_iter = stmt.query_map([], |row| {
            Ok(Commands {
                id: row.get(0)?,
                user: row.get(1)?,
                command: row.get(2)?,
                executor: row.get(3)?,
                add_time: row.get(4)?,
                status: row.get(5)?,
                start_time: row.get(7)?,
                finish_time: row.get(6)?,
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
    pub fn update_status_running(&self, id: i32) -> Result<()> {
        let stmt = format!("UPDATE commands SET status=9 WHERE id={}", id);
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_status_finish(&self, id: i32) -> Result<()> {
        let stmt = format!("UPDATE commands SET status=1 WHERE id={}", id);
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_status_error(&self, id: i32) -> Result<()> {
        let stmt = format!("UPDATE commands SET status=2 WHERE id={}", id);
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_status_cancel(&self, id: i32) -> Result<()> {
        let stmt = format!("UPDATE commands SET status=3 WHERE id={}", id);
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_start_time(&self, id: i32, start_time: i64) -> Result<()> {
        let stmt = format!(
            "UPDATE commands SET start_time={} WHERE id={}",
            start_time, id
        );
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    pub fn update_finish_time(&self, id: i32, finish_time: i64) -> Result<()> {
        let stmt = format!(
            "UPDATE commands SET finish_time={} WHERE id={}",
            finish_time, id
        );
        self.conn.execute(&stmt, ())?;
        Ok(())
    }
    fn gen_move_vec(id_vec: &[i32]) -> (Vec<i32>, Vec<i32>) {
        if id_vec.len() > 0 {
            let mut id_vec_ret = Vec::new();
            let id_vec_plus: Vec<i32> = id_vec.iter().map(|x| x + 1).collect();
            for id in &id_vec_plus {
                id_vec_ret.push(*id);
                if !id_vec_plus.contains(&(id + 1)) {
                    break;
                }
            }
            (id_vec[..id_vec_ret.len()].to_vec(), id_vec_ret)
        } else {
            (vec![], vec![])
        }
    }
    pub fn move_jobs(&self, id_vec: &[i32]) -> Result<()> {
        let (id_vec_1, id_vec_2) = SqliteDB::gen_move_vec(id_vec);
        for (id, new_id) in zip(id_vec_1.iter().rev(), id_vec_2.iter().rev()) {
            let stmt = format!("UPDATE commands SET id={} WHERE id={}", new_id, id);
            self.conn.execute(&stmt, ())?;
        }
        Ok(())
    }
    fn gen_align_vec(id_vec: &[i32]) -> (Vec<i32>, Vec<i32>) {
        let mut new_id_vec: Vec<i32> = Vec::new();
        for i in 0..id_vec.len() {
            new_id_vec.push(i as i32 + 1);
        }
        let mut same_len = 0;
        for i in 0..id_vec.len() {
            if id_vec[i] != new_id_vec[i] {
                same_len = i;
                break;
            }
        }
        (
            id_vec[same_len..id_vec.len()].to_vec(),
            new_id_vec[same_len..id_vec.len()].to_vec(),
        )
    }
    pub fn align_id(&self, id_vec: &[i32]) -> Result<()> {
        let (id_vec_1, id_vec_2) = SqliteDB::gen_align_vec(id_vec);
        for (id, new_id) in zip(id_vec_1, id_vec_2) {
            let stmt = format!("UPDATE commands SET id={} WHERE id={}", new_id, id);
            self.conn.execute(&stmt, ())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_move_jobs() {
        let id_vec = vec![7, 8, 10, 12];
        let (id_vec_1, id_vec_2) = SqliteDB::gen_move_vec(&id_vec);
        println!("{:?}", id_vec_1);
        println!("{:?}", id_vec_2);
    }
}
