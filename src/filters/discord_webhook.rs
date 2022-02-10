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
    override_discord_ratelimit: Option<f64>,
}

impl fmt::Debug for DiscordWebhookFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DiscordWebhookFilter").finish()
    }
}

impl DiscordWebhookFilter {
    fn new(dest: String, webhook: String, override_discord_ratelimit: Option<f64>) -> Self {
        Self {
            webhook,
            dest,
            override_discord_ratelimit
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
                    let mut int = 60.0;

                    if let Some(delay) = self.override_discord_ratelimit {
                        int = delay;
                    }
                    else {
                        let js = &res.json::<Value>().await?["retry_after"];
                        if let Some(delay) = js.as_f64() {
                            // so this is complete bullshit, if you actually do what this recommends
                            // you have to wait for like 3-10 minutes, just to get ratelimited after 5 requests again;
                            // for that reason you can override whether or not to listen to discord or just wait a certain amount of seconds
                            int = delay;
                            
                        }
                    }
                    log::warn!("Hit discord ratelimit, sleeping for {} seconds", int);
                    time::sleep(Duration::from_secs_f64(int)).await;
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
    override_discord_ratelimit: Option<f64>,
}

impl OutputFilterFactory for DiscordWebhookFilter {
    type Config = DiscordConfig;

    const NAME: &'static str = "discord";

    fn build_filters(config: Self::Config) -> Vec<Box<dyn OutputFilter<Item = EntryBox>>> {
        let mut filters: Vec<Box<dyn OutputFilter<Item = EntryBox>>> = Vec::new();

        for (dest, url) in config.webhooks {

            filters.push(Box::new(DiscordWebhookFilter::new(dest, url, config.override_discord_ratelimit)))
        }

        filters
    }
}
