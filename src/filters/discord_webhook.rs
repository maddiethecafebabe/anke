use anke_core::{
    async_trait, log, reqwest,
    serde_json::{self, Value},
    EntryBox, OutputFilter,
    tokio::time,
};
use std::env;
use std::fmt;
use std::time::Duration;

pub struct DiscordWebhookFilter {
    webhook: String,
}

impl fmt::Debug for DiscordWebhookFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DiscordWebhookFilter").finish()
    }
}

impl DiscordWebhookFilter {

    async fn send(&self, entry: &EntryBox) -> reqwest::Result<reqwest::Response> {
        let body = serde_json::json!({
            "embeds" : [
                {
                    "title": entry.title().await.map(|t| t.into()).unwrap_or(Value::Null),
                    "color": 10034204u32,
                    "url": entry.title_url().await.map(|t| t.into()).unwrap_or(Value::Null),
                    "image": {
                        "url": entry.image_url().await.map(|t| t.into()).unwrap_or(Value::Null),
                    }
                }
            ]
        });

        loop {
            use reqwest::StatusCode;

            let res = reqwest::Client::new()
                .post(&self.webhook)
                .json(&body)
                .send()
                .await?;

            match res.status() {
                StatusCode::OK | StatusCode::NO_CONTENT => return Ok(res),
                StatusCode::TOO_MANY_REQUESTS => {
                    let js = &res.json::<Value>().await?["retry_after"];
                    if let Some(int) = js.as_f64() {
                        log::debug!("Hit discord ratelimit, sleeping for {} seconds", int);
                        time::sleep(Duration::from_secs_f64(int)).await;
                    }
                },
                _ => { return res.error_for_status(); }
            }
    
        }
    }
}

#[async_trait]
impl OutputFilter for DiscordWebhookFilter {
    type Item = EntryBox;

    async fn filter(&mut self, entry: EntryBox) -> Option<EntryBox> {
        if let Err(e) = self.send(&entry).await {
            log::error!("Error during webhook POST: {:?}", e);
        }

        Some(entry)
    }
}
