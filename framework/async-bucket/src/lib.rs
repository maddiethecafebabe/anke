use std::cell::Cell;
use std::cmp;
use std::mem;
use std::sync::{Mutex, MutexGuard};
use tokio::time::{self, Duration, Instant};

pub struct AsyncBucket {
    rate: Duration,
    ctr_limit: usize,
    ctr: Mutex<Cell<usize>>,
    last_decrement: Mutex<Instant>,
}

impl AsyncBucket {
    pub fn new(rate: Duration, limit: usize) -> Self {
        Self {
            rate,
            ctr_limit: limit,
            ctr: Mutex::new(Cell::new(0)),
            last_decrement: Mutex::new(Instant::now()),
        }
    }

    pub fn init(self, init_ctr: usize) -> Self {
        Self {
            ctr: Mutex::new(Cell::new(init_ctr)),
            last_decrement: Mutex::new(Instant::now()),
            ..self
        }
    }

    pub fn update(&self) -> MutexGuard<Cell<usize>> {
        tracing::debug!("Updating bucket counter");
        let mut lock = self.last_decrement.lock().unwrap();

        let now = Instant::now();
        let elapsed = now.duration_since(*lock);

        let cur_lock = self.ctr.lock().unwrap();
        let cur = cur_lock.get();
        let inc_by = elapsed.as_millis() as usize / self.rate.as_millis() as usize;

        if inc_by == 0 {
            return cur_lock;
        }

        *lock = now;
        cur_lock.set(cmp::min(self.ctr_limit, cur + inc_by));

        tracing::debug!(
            "Old: {}, New: {}, inc_by: {}, elapsed: {:?}",
            cur,
            cur_lock.get(),
            inc_by,
            elapsed
        );

        mem::drop(lock);
        cur_lock
    }

    pub fn try_take(&self, cnt: usize) -> Result<(), usize> {
        let lock = self.update();

        // we hold a mutex guard so we know the current data cant get modified
        let cur = lock.get();

        if cur < cnt {
            return Err(cnt - cur);
        } else {
            lock.set(cur - cnt);
            mem::drop(lock);
            return Ok(());
        }
    }

    pub async fn take(&self, cnt: usize) -> () {
        assert!(cnt <= self.ctr_limit);

        loop {
            match self.try_take(cnt) {
                Ok(_) => break,
                Err(delta) => {
                    let sleep_for = self.rate * delta as u32;

                    tracing::trace!(
                        "Got {} too few in bucket, sleeping for {:?}",
                        delta,
                        self.rate
                    );

                    time::sleep(sleep_for).await
                }
            };
        }
    }
}
