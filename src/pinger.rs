use crate::db::Database;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::time::interval;
use std::net::IpAddr;

pub async fn start_pinger(db: Arc<Database>, target: String) {
    info!("Starting pinger service for target: {}", target);

    let config = Config::default();

    let client = Client::new(&config).ok();
    if client.is_none() {
        warn!("Failed to create ping client. Pinger service will continue with resolution-only mode or recording failures. (This might be due to missing raw socket permissions)");
    }

    let mut interval = interval(Duration::from_secs(2));
    let mut sequence = 0u16;

    loop {
        interval.tick().await;

        // Resolve target to IP if it's a hostname
        let target_ip = match target.parse::<IpAddr>() {
            Ok(ip) => Some(ip),
            Err(_) => {
                match tokio::net::lookup_host(format!("{}:0", target)).await {
                    Ok(mut addrs) => addrs.next().map(|addr| addr.ip()),
                    Err(e) => {
                        warn!("DNS lookup failed for {}: {}", target, e);
                        None
                    }
                }
            }
        };

        let target_ip = match target_ip {
            Some(ip) => ip,
            None => {
                // If resolution fails, record a failure and wait for next interval
                if let Err(e) = db.record_ping(-1.0) {
                    error!("Failed to record DNS failure: {}", e);
                }
                continue;
            }
        };

        if let Some(ref client) = client {
            let mut pinger = client
                .pinger(target_ip, PingIdentifier(rand::random()))
                .await;
            pinger.timeout(Duration::from_secs(1));

            let payload = [0u8; 56];
            match pinger.ping(PingSequence(sequence), &payload).await {
                Ok((_, duration)) => {
                    let latency = duration.as_secs_f64() * 1000.0;
                    debug!("Ping {} success: {:.2}ms", target, latency);
                    if let Err(e) = db.record_ping(latency) {
                        error!("Failed to record ping: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Ping {} failed: {}", target, e);
                    // Record -1.0 for failure/timeout
                    if let Err(e) = db.record_ping(-1.0) {
                        error!("Failed to record ping failure: {}", e);
                    }
                }
            }
        } else {
            // If client is none (no permissions), just record -1.0 every interval
            if let Err(e) = db.record_ping(-1.0) {
                error!("Failed to record pinger failure (no client): {}", e);
            }
        }

        sequence = sequence.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_pinger_record() {
        let db_path = "test_pinger_record.db";
        // Ensure we start with a fresh DB for the test
        let _ = std::fs::remove_file(db_path);
        let db = Arc::new(Database::new(db_path).unwrap());

        let latency = 12.34;
        db.record_ping(latency).unwrap();
        let stats = db.get_pinger_stats().unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].0, latency);
        let _ = std::fs::remove_file(db_path);
    }
}
