use bsky_sdk::api::types::string::Did;
use bsky_sdk::BskyAgent;
use std::collections::HashMap;
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenvy::dotenv().is_ok() {
        println!("loaded .env");
    }
    let handle = env::var("BSKY_HANDLE")?;
    let password = env::var("BSKY_APP_PASSWORD")?;

    let agent = BskyAgent::builder().build().await?;
    agent.login(&handle, &password).await?;
    println!("logged in as {handle}");

    let mut handle_cache: HashMap<String, String> = HashMap::new();

    loop {
        let notifications = agent
            .api
            .app
            .bsky
            .notification
            .list_notifications(
                bsky_sdk::api::app::bsky::notification::list_notifications::ParametersData {
                    cursor: None,
                    limit: None,
                    priority: None,
                    seen_at: None,
                    reasons: None,
                }
                .into(),
            )
            .await;

        let unread = match notifications {
            Ok(output) => output
                .data
                .notifications
                .into_iter()
                .filter(|n| !n.data.is_read)
                .collect::<Vec<_>>(),
            Err(e) => {
                eprintln!("poll failed: {e}");
                sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        for notif in &unread {
            let name = match resolve_handle(&agent, &mut handle_cache, &notif.author.data.did).await
            {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("resolve handle failed: {e}");
                    "<unknown>".to_string()
                }
            };
            let time = notif.indexed_at.as_ref().format("%Y-%m-%d %H:%M");
            println!("[{:>7}] @{:<20} {}", notif.reason, name, time);
        }

        if !unread.is_empty() {
            let _ = agent
                .api
                .app
                .bsky
                .notification
                .update_seen(
                    bsky_sdk::api::app::bsky::notification::update_seen::InputData {
                        seen_at: bsky_sdk::api::types::string::Datetime::now(),
                    }
                    .into(),
                )
                .await;
        }

        sleep(Duration::from_secs(30)).await;
    }
}

async fn resolve_handle(
    agent: &BskyAgent,
    cache: &mut HashMap<String, String>,
    did: &Did,
) -> Result<String, Box<dyn std::error::Error>> {
    let key = did.as_str().to_owned();
    if let Some(handle) = cache.get(&key) {
        return Ok(handle.clone());
    }
    let profile = agent
        .api
        .app
        .bsky
        .actor
        .get_profile(
            bsky_sdk::api::app::bsky::actor::get_profile::ParametersData {
                actor: did.clone().into(),
            }
            .into(),
        )
        .await?;
    let handle = profile.data.handle.to_string();
    cache.insert(key, handle.clone());
    Ok(handle)
}
