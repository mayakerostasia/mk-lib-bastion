use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};

pub struct MakoBattery {
    sem: Arc<Semaphore>,
    jh: tokio::task::JoinHandle<()>,
}

impl MakoBattery {
    pub fn new(duration: Duration, capacity: usize) -> Self {
        let sem = Arc::new(Semaphore::new(capacity));

        let jh = tokio::spawn({
            let sem = sem.clone();
            let mut interval = interval(duration);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            async move {
                loop {
                    interval.tick().await;
                    if sem.available_permits() < capacity {
                        sem.add_permits(1);
                    }
                }
            }
        });

        Self { jh, sem }
    }

    pub async fn acquire(&self) {
        // Acquire the next permit
        let permit = self.sem.acquire().await.unwrap();
        // Forget the permit so it doesn't get released
        permit.forget();
    }
}

impl Drop for MakoBattery {
    fn drop(&mut self) {
        self.jh.abort();
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Error;
    // use chrono::Duration;
    use super::*;

    #[tokio::test]
    async fn test_battery_capacity() -> Result<(), Error> {
        let capacity = 5;
        let update_interval = tokio::time::Duration::from_secs_f32(1.0 / capacity as f32);
        let bucket = MakoBattery::new(update_interval, capacity);
        for _ in 0..10 {
            bucket.acquire().await;
            println!("Acquired!")
        }
        Ok(())
    }
}
