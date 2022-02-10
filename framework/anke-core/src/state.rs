use rusqlite::{params, Connection};

use crate::log;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct State {
    db: Arc<Mutex<Connection>>,
}

impl State {
    pub fn new(path: String) -> Self {
        let conn = Connection::open(path).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS tokens ( token_group TEXT NOT NULL, token_id TEXT NOT NULL, token TEXT, UNIQUE(token_group, token_id) ON CONFLICT REPLACE  );", []).unwrap();

        Self {
            db: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn storage_for(
        &self,
        group: impl Into<String>,
        id: impl Into<String>,
    ) -> TokenStorageConnection {
        TokenStorageConnection::new(Arc::clone(&self.db), group.into(), id.into())
    }
}

#[derive(Debug)]
pub struct TokenStorageConnection {
    group: String,
    id: String,
    db: Arc<Mutex<Connection>>,
}

impl TokenStorageConnection {
    pub(crate) fn new(db: Arc<Mutex<Connection>>, group: String, id: String) -> Self {
        Self { group, id, db }
    }

    pub fn fetch<T: fmt::Debug + From<String>>(&self) -> Option<T> {
        if let Ok(db) = self.db.lock() {
            let res: Option<String> = db
                .query_row(
                    "SELECT token FROM tokens WHERE (token_group = ?1 AND token_id = ?2)",
                    params![self.group, self.id],
                    |row| row.get(0),
                )
                .ok();

            log::debug!("Read {:?} from {}::{}", res, self.group, self.id);
            res.map(|r| r.into())
        } else {
            None
        }
    }

    pub fn store<T: Into<String> + fmt::Debug>(&mut self, new: T) {
        log::debug!("Storing {:?} at {}::{}", new, self.group, self.id);
        if let Ok(db) = self.db.lock() {
            db.execute(
                "INSERT INTO tokens (token_group, token_id, token) VALUES (?1, ?2, ?3)",
                params![self.group, self.id, new.into()],
            )
            .ok();
        }
    }
}
