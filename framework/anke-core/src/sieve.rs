use rusqlite::{Connection, params};

use crate::async_trait;
use crate::crossbeam_channel::{Sender, TrySendError};
use crate::tokio::time;
use crate::EntryBox;
use crate::Result;
use crate::log;
use std::fmt::{self, Debug};
use std::sync::{Arc, Mutex};
use std::time::Duration;


pub struct SieveContext {
    pub pipeline: AsyncSender<EntryBox>,
    pub site: String,
    pub tag: String,
    pub task_number: usize,
    pub token_db: TokenStorageConnection,
    pub max_new_scrape_limit: usize,
}

impl fmt::Display for SieveContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} [{}::{}]", self.task_number, self.site, self.tag)
    }
}

pub type Token = String;

pub struct TokenStorageConnection {
    group: String,
    id: String,
    db: Arc<Mutex<Connection>>,
}

impl TokenStorageConnection {
    pub(crate) fn new(db: Arc<Mutex<Connection>>, group: &str, id: &str) -> Self {
        Self {
            group: group.to_owned(),
            id: id.to_owned(),
            db,
        }
    }

    pub fn fetch<T: Debug + From<String>>(&self) -> Option<T> {
        if let Ok(db) = self.db.lock() {
            let res: Option<String> = db.query_row(
                "SELECT token FROM tokens WHERE (token_group = ?1 AND token_id = ?2)",
                params![self.group, self.id],
                |row| {
                    row.get(0)
                }
            ).ok();

            log::debug!("Read {:?} from {}::{}", res, self.group, self.id);    
            res.map(|r| r.into())
        } else {
            None
        }
    }

    pub fn store<T: Into<Token> + Debug>(&mut self, new: T) {
        log::debug!("Storing {:?} at {}::{}", new, self.group, self.id);
        if let Ok(db) = self.db.lock() {
            db.execute(
                "INSERT INTO tokens (token_group, token_id, token) VALUES (?1, ?2, ?3)",
                params![self.group, self.id, new.into()]
            ).ok();
        }
    }
}
