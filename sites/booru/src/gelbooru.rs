use anke_core::{
    reqwest, url::Url, Aggregator, Context, Entry, EntryBox, PipelineResult, State,
    TokenStorageConnection,
};

use regex::Regex;

use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct GelbooruId(usize);

impl From<GelbooruId> for String {
    fn from(s: GelbooruId) -> String {
        format!("{}", s.0)
    }
}

impl From<&str> for GelbooruId {
    fn from(s: &str) -> GelbooruId {
        GelbooruId(s.parse().unwrap())
    }
}

impl From<String> for GelbooruId {
    fn from(s: String) -> GelbooruId {
        GelbooruId(s.parse::<usize>().unwrap())
    }
}

const PAGE_URL_BASE: &str = "https://gelbooru.com/index.php?page=post&s=list";

#[derive(Debug)]
struct GelbooruPage {
    url: Url,
    post_cnt: usize,
    posts: Vec<GelbooruId>,
}

impl GelbooruPage {
    async fn tag(tag: &String) -> Self {
        let mut url = Url::from_string_with_query(PAGE_URL_BASE.to_owned());

        let headers = url.query_mut();
        headers.insert("tags".into(), tag.clone());
        headers.insert("pid".into(), "0".into());

        let mut this = Self {
            url,
            post_cnt: 0,
            posts: Vec::new(),
        };

        this.fetch_page().await;

        this
    }

    async fn fetch_page(&mut self) {
        lazy_static! {
            static ref ID_REGEX: Regex = Regex::new(r#"<a id="p(?P<id>\w+)""#).unwrap();
        };

        let raw = reqwest::get(self.url.into_url())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let mut v: Vec<GelbooruId> = ID_REGEX
            .captures_iter(&raw)
            .map(|c| c["id"].into())
            .collect();

        v.reverse();
        self.post_cnt = v.len();
        self.posts = v;
    }

    async fn next_page(&mut self) {
        let headers = self.url.query_mut();
        let offset: usize = headers.get("pid".into()).unwrap().parse().unwrap();
        let offset = offset + self.post_cnt;

        headers.insert("pid".into(), offset.to_string());

        self.fetch_page().await;
    }

    async fn next_post(&mut self) -> Option<GelbooruId> {
        match self.posts.pop() {
            None => {
                self.next_page().await;
                self.posts.pop()
            }
            post => post,
        }
    }
}

#[derive(Debug)]
pub struct GelbooruAggregator {
    pub(crate) tag: String,
    storage: TokenStorageConnection,
    fresh_poll_limit: isize,
    poll_limit: isize,
    tags_in_embed: bool,
}

impl GelbooruAggregator {
    pub(crate) fn new(
        tag: String,
        state: &State,
        fresh_poll_limit: isize,
        poll_limit: isize,
        tags_in_embed: bool,
    ) -> Box<dyn Aggregator<Item = EntryBox, PipelineState = State>> {
        let storage = state.storage_for("gelbooru", &tag);
        Box::new(Self {
            tag,
            storage,
            fresh_poll_limit,
            poll_limit,
            tags_in_embed
        })
    }

    async fn scrape_posts_from_page(&self, mut limit: isize, until: GelbooruId) -> Vec<GelbooruId> {
        let mut page = GelbooruPage::tag(&self.tag).await;

        let mut r = Vec::new();
        while let Some(id) = page.next_post().await {
            limit -= 1;
            debug!("[{}] limit({}) > 0 = {}, id({}) > until({}) = {}", self.tag, limit, limit > 0, id.0, until.0, id > until);
            if limit >= 0 && id > until {
                debug!("[{}] Pushing {}!", &self.tag, id.0);
                r.push(id)
            } else {
                break;
            }
        }

        r
    }
}

#[async_trait]
impl Aggregator for GelbooruAggregator {
    type Item = EntryBox;
    type PipelineState = State;

    async fn poll(
        &mut self,
        ctx: &mut Context<Self::Item, Self::PipelineState>,
    ) -> PipelineResult<()> {
        info!("Polled {}", self.tag);

        let (limit, mut newest) = {
            match self.storage.fetch() {
                Some(a) => (self.poll_limit, a),
                None => (self.fresh_poll_limit, GelbooruId(0)),
            }
        };

        for post in self.scrape_posts_from_page(limit, newest).await {
            if post > newest {
                newest = post;
            }

            info!("Post: {}", post.0);

            ctx.sender
                .send(Box::new(GelbooruEntry::fetch_from_id(post, self.tags_in_embed).await?))
                .await;
        }

        self.storage.store(newest);

        Ok(())
    }
}

#[derive(Debug)]
struct GelbooruEntry {
    tags: HashSet<String>,
    image_url: Option<String>,
    post_url: String,
    artist: Option<String>,
    characters: HashSet<String>,
    tags_in_embed: bool,
}

impl GelbooruEntry {
    pub async fn fetch_from_id(id: GelbooruId, tags_in_embed: bool) -> reqwest::Result<Self> {
        let url = format!(
            "https://gelbooru.com/index.php?page=post&s=view&id={}",
            id.0
        );

        let raw = reqwest::get(&url).await?.text().await?;

        Ok(Self::extract_info(url, raw, tags_in_embed))
    }

    fn extract_info(post_url: String, raw: String, tags_in_embed: bool) -> Self {
        lazy_static! {
            static ref IMAGE_URL_REG: Regex = Regex::new(r#"image\.attr\('src','(?P<url>.+)'\);"#).unwrap();
            static ref TAGS_REG: Regex = Regex::new(r#"data-tags="(?P<tags>(\s?([^\s"])*\s?)*)"#).unwrap();
            static ref ARTIST_REG: Regex = Regex::new(r#"class="tag-type-artist"><span class="sm-hidden"><a href=".+?">\?</a> </span><a href=".+?">(?P<artist>.+?)</a>"#).unwrap();
            static ref CHARACTERS_REGEX: Regex = Regex::new(r#"class="tag-type-character"><span class="sm-hidden"><a href=".+?">\?</a> </span><a href=".+?">(?P<character>.+?)</a>"#).unwrap();
        }

        let image_url = IMAGE_URL_REG.captures(&raw).map(|c| c["url"].to_owned());

        let tags = match TAGS_REG.captures(&raw) {
            Some(cap) => cap["tags"].split_whitespace().map(str::to_string).collect(),
            None => HashSet::new(),
        };

        let artist = ARTIST_REG.captures(&raw).map(|c| c["artist"].to_owned());

        let characters: HashSet<String> = CHARACTERS_REGEX
            .captures_iter(&raw)
            .map(|c| c["character"].to_owned())
            .collect();

        Self {
            post_url,
            tags,
            image_url,
            artist,
            characters,
            tags_in_embed
        }
    }
}

impl Entry for GelbooruEntry {
    fn tags(&self) -> Option<&HashSet<String>> {
        Some(&self.tags)
    }

    fn title(&self) -> Option<String> {
        let mut chars = itertools::join(self.characters.iter().map(|c| format!("\"{}\"", c)), ",");
        if chars.is_empty() {
            chars = "untagged character(s)".into()
        }

        if let Some(artist) = &self.artist {
            return Some(format!("{} by \"{}\"", chars, artist));
        }
        else {
            return Some(format!("{} by untagged artist", chars))
        }
    }

    fn title_url(&self) -> Option<String> {
        Some(self.post_url.clone())
    }

    fn image_url(&self) -> Option<String> {
        self.image_url.clone()
    }

    fn build_extra_fields(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if self.tags_in_embed {
            map.insert(
                "Tags".into(),
                itertools::join(self.tags.iter(), ", "),
            );

            if let Some(artist) = &self.artist {
                map.insert("artist".into(), artist.clone());
            }
        }
        
        map
    }
}
