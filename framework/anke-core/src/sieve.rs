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

pub struct AsyncSender<T> {
    sender: Sender<T>,
    timeout: Duration,
}

impl<T> std::clone::Clone for AsyncSender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            timeout: self.timeout.clone(),
        }
    }
}

impl<T> AsyncSender<T> {
    pub fn new(sender: Sender<T>, timeout: Option<Duration>) -> Self {
        Self {
            sender,
            timeout: timeout.unwrap_or(Duration::from_secs(5)),
        }
    }

    /// async wrapper for Sender::send
    pub async fn send(&mut self, mut item: T) {
        while let Err(e) = self.sender.try_send(item) {
            item = match e {
                TrySendError::Full(item) => item,
                TrySendError::Disconnected(_) => {
                    panic!("receiving channel got closed, please dont do that")
                }
            };

            time::sleep(self.timeout).await;
        }
    }
}

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

/// A content sieve, responsible for, given a (site, tag) pair
/// to scrape for content and sieve out new/relevant content, then send that
/// over the sender.
#[async_trait]
pub trait Sieve: Debug + Send + Sync {
    /// Decides how long the task will sleep after running.
    /// This can be overriden with custom logic for e.g. a ratelimit bucket.
    /// The default is to sleep for 15 minutes.
    fn sleep_duration(&self) -> Duration {
        Duration::from_secs(15 * 60)
    }

    /// The main interface, when called this should send all new content to the `ctx.pipeline` sender.
    async fn poll(&mut self, ctx: &mut SieveContext) -> Result<()>;
}
