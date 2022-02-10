use anke_core::{
    async_trait,
    log, reqwest,
    serde_json::{self, Value},
    tokio::time,
    EntryBox, OutputFilter, OutputFilterFactory,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

pub struct DiscordWebhookFilter {
    webhook: String,
    dest: String,
}

impl fmt::Debug for DiscordWebhookFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DiscordWebhookFilter").finish()
    }
}

impl DiscordWebhookFilter {
    fn new(dest: String, webhook: String) -> Self {
        Self {
            webhook,
            dest
        }
    }

    async fn send(&self, entry: &EntryBox) -> reqwest::Result<reqwest::Response> {
        log::debug!("Sending {:?} into {}", entry, self.dest);

        let mut body = serde_json::json!({
            "embeds" : [
                {
                    "title": entry.title().map(|t| t.into()).unwrap_or(Value::Null),
                    "color": 10034204u32,
                    "url": entry.title_url().map(|t| t.into()).unwrap_or(Value::Null),
                    "image": {
                        "url": entry.image_url().map(|t| t.into()).unwrap_or(Value::Null),
                    }
                }
            ]
        });



        body["embeds"][0]["fields"] = Value::Array({
            let mut fields = Vec::new();
            for (name, body) in entry.build_extra_fields() {
                fields.push(serde_json::json!(
                    {
                        "name": name,
                        "value": body
                    }
                ))            
            }
            fields
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
                }
                _ => {
                    return res.error_for_status();
                }
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

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    webhooks: HashMap<String, String>,
}

impl OutputFilterFactory for DiscordWebhookFilter {
    type Config = DiscordConfig;

    const NAME: &'static str = "discord";

    fn build_filters(config: Self::Config) -> Vec<Box<dyn OutputFilter<Item = EntryBox>>> {
        let mut filters: Vec<Box<dyn OutputFilter<Item = EntryBox>>> = Vec::new();

        for (dest, url) in config.webhooks {

            filters.push(Box::new(DiscordWebhookFilter::new(dest, url)))
        }

        filters
    }
}
