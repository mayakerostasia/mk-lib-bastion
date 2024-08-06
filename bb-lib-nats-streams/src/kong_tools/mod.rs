mod kingkong;
mod kong;
mod monkey;

pub use kingkong::KingKong;
pub use kong::Kong;
pub use monkey::Monkey;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use tokio::time::Duration;

    const NATS_ADDR: &str = "nats://10.2.4.106:4222";
    const HEALTH_BIND: &str = "127.0.0.1:4202";

    #[tokio::test]
    async fn test_king_kong_init() -> Result<(), Error> {
        let kk = KingKong::new("test", NATS_ADDR, HEALTH_BIND).await;
        assert_eq!("test", kk.subject);
        Ok(())
    }

    #[tokio::test]
    async fn test_kkong_wait() -> Result<(), Error> {
        let kk = KingKong::new("test", NATS_ADDR, HEALTH_BIND).await;
        assert_eq!("test", kk.subject);

        let wait_handle = tokio::task::spawn(async move { kk.wait().await });
        tokio::time::sleep(Duration::from_secs(20)).await;
        wait_handle.abort();
        Ok(())
    }
}
