use anke_core::{async_trait, async_bucket::AsyncBucket, log, reqwest, url::Url, Entry, Result, Sieve, SieveContext};
use lazy_static::lazy_static;
use regex::Regex;
use std::time::Duration;

lazy_static! {
    static ref BUCKET: AsyncBucket = AsyncBucket::new(Duration::from_secs_f32(0.5), 50).init(50);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GelbooruId(usize);

impl From<String> for GelbooruId {
    fn from(s: String) -> Self {
        Self(s.parse().unwrap())
    }
}

struct GelbooruPage {
    url: Url,
}

impl GelbooruPage {
    fn from(site: &str, tag: &str) -> Self {
        let mut url =
            Url::from_string_with_query(format!("https://{}/index.php?page=post&s=list", site));
        url.query_mut().insert("tags".into(), tag.into());

        Self { url }
    }

    async fn fetch(self) -> Result<Vec<GelbooruId>> {
        lazy_static! {
            static ref ARTICLE_REGEX: Regex =
                Regex::new(r#"<article class=".+?">\n\s*<a id="p(?P<id>\w*)""#).unwrap();
        };

        BUCKET.take(1).await;
        let raw = reqwest::get(self.url.into_url()).await?.text().await?;

        let mut entries = Vec::new();
        for id in ARTICLE_REGEX
            .captures_iter(&raw)
            .map(|c| c["id"].parse().map(GelbooruId))
        {
            entries.push(id?);
        }

        Ok(entries)
    }
}

#[derive(Clone, Debug)]
pub struct GelbooruSieve {}

impl GelbooruSieve {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct GelbooruEntry {
    post_url: String,
    content_url: Option<String>,
    artist: Option<String>,
    id: usize,
    tags: Vec<String>,
    characters: Vec<String>,
}

impl GelbooruEntry {
    pub async fn fetch_from_id(id: GelbooruId) -> Result<Self> {
        lazy_static! {
            static ref CONTENT_REGEX: Regex = Regex::new(r#"(?:image\.attr\('src','(.+?)'\))"#).unwrap();
            static ref ARTIST_REGEX: Regex =
        Regex::new(r#"<li\sclass="tag-type-artist">(?:<.+?>){3}(.+?)</a>"#).unwrap();
            static ref TAGS_REGEX: Regex = Regex::new(r#""#).unwrap();
            static ref CHARS_REGEX: Regex = Regex::new(r#""#).unwrap();
        };

        BUCKET.take(1).await;
        let url = format!("https://gelbooru.com/index.php?page=post&s=view&id={}", id.0);
        let raw = reqwest::get(&url).await?.text().await?;

        let content_url = CONTENT_REGEX.captures(&raw).map(|s| s.get(1)).flatten().map(|m| m.as_str().to_owned());
        let artist = ARTIST_REGEX.captures(&raw).map(|s| s.get(1)).flatten().map(|m| m.as_str().to_owned());
        

        Ok(Self{
            post_url: url,
            content_url,
            artist,
            id: id.0,
            tags: Vec::new(),
            characters: Vec::new(),
        })
    }
}

#[async_trait]
impl Entry for GelbooruEntry {
    async fn title_url(&self) -> Option<String> {
        Some(self.post_url.clone())
    }

    async fn image_url(&self) -> Option<String> {
        self.content_url.clone()
    }

    async fn title(&self) -> Option<String> {
        Some(
            format!("[{}] {} drawn by {}",
                self.id,
                self.characters.iter().nth(0).unwrap_or(&"<unknown character>".into()),
                self.artist.as_ref().unwrap_or(&"<unknown artist>".into()),
            )
        )
    }
}

#[async_trait]
impl Sieve for GelbooruSieve {
    async fn poll(&mut self, ctx: &mut SieveContext) -> Result<()> {
        let newest = ctx.token_db.fetch();
        let mut scraped_ids = Vec::new();
        let mut limit = 0;
        for post in itertools::sorted(GelbooruPage::from(&ctx.site, &ctx.tag)
            .fetch()
            .await?)
            .filter(|id| {
                if let Some(newest) = newest { 
                    log::debug!("{}::{} @ {:?} > {:?} == {}", ctx.site, ctx.tag, *id, newest, *id > newest);
                    *id > newest
                } else { 
                    limit += 1; limit <= ctx.max_new_scrape_limit
                }
            })
        {
            log::debug!("Pushing {:?}", post);
            scraped_ids.push(post.0);

            ctx.pipeline
                .send(Box::new(GelbooruEntry::fetch_from_id(post).await?))
                .await;
        }

        let highest_of_scraped = scraped_ids.into_iter().max().unwrap_or(newest.map(|n| n.0).unwrap_or(0));
        
        Ok(ctx.token_db.store(highest_of_scraped.to_string()))
    }

    fn sleep_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60)
    }
}
