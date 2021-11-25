use crossbeam_channel::Receiver;

use crate::{
    log,
    sieve::TokenStorageConnection,
    tokio::{self, time},
    AsyncSender, EntryBox, OutputFilter, Sieve, SieveContext,
};

use std::env;
use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::{self, Connection};

pub struct Pipeline {
    filters: Vec<Box<dyn OutputFilter>>,
    sender: AsyncSender<EntryBox>,
    receiver: Option<Receiver<EntryBox>>,
    task_number: Option<usize>,
    database: Arc<Mutex<Connection>>,
}

impl Pipeline {
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn new(filters: Vec<Box<dyn OutputFilter>>) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let conn = Connection::open(PathBuf::from(env::var("APP_ROOT").unwrap()).join("pipeline.db")).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS tokens ( token_group TEXT NOT NULL, token_id TEXT NOT NULL, token TEXT, UNIQUE(token_group, token_id) ON CONFLICT REPLACE  );", []).unwrap();

        Self {
            filters: filters,
            sender: AsyncSender::new(sender, None),
            receiver: Some(receiver),
            task_number: None,
            database: Arc::new(Mutex::new(conn)),
        }
    }

    pub async fn spawn_receiver_task(mut self) -> Self {
        let receiver = self
            .receiver
            .take()
            .expect("Attempted to spawn receiver task twice (no receiver left)");
        let mut filters = mem::take(&mut self.filters);

        tokio::spawn(async move {
            log::debug!("Spawned receiver task!");

            loop {
                if let Some(mut entry) = receiver.try_recv().ok() {
                    log::debug!("We found a match!! {:?}", entry);

                    'l: for result_filter in filters.iter_mut() {
                        entry = match result_filter.filter(entry).await {
                            Some(e) => e,
                            None => break 'l,
                        };
                    }
                } else {
                    log::trace!("Nothing in the queue, sleeping for 3 secs...");
                    time::sleep(Duration::from_secs(3)).await;
                }
            }
        });

        self
    }

    pub async fn spawn_sender_tasks(
        mut self,
        mapping: Vec<(String, String, Box<dyn Sieve>)>,
    ) -> Self {
        let mut task_number = self.task_number.map(|n| n + 1).unwrap_or(0);

        // spawn a task for every tag
        for (site, tag, mut sieve) in mapping {
            let sender = self.sender.clone();
            let token_db = TokenStorageConnection::new(Arc::clone(&self.database), &site, &tag);

            tokio::spawn(async move {
                let mut ctx = SieveContext {
                    pipeline: sender,
                    site,
                    tag,
                    task_number,
                    token_db,
                    max_new_scrape_limit: 50,
                };

                log::info!("Spawned task {}", ctx);

                loop {
                    if let Err(e) = sieve.poll(&mut ctx).await {
                        log::error!("{}", e);
                        // TODO: for some reason this is never called, even if an error happens
                    }

                    time::sleep(sieve.sleep_duration()).await;
                }
            });
            task_number += 1;
        }

        self.task_number = Some(task_number);
        self
    }

    pub async fn loop_endless(self) -> ! {
        loop {
            time::sleep(Duration::from_secs(0xffffffff)).await;
        }
    }
}
