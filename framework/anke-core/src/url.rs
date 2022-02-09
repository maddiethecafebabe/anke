use crate::reqwest;

use linked_hash_map::LinkedHashMap as HashMap;

#[derive(Debug)]
pub struct Url {
    base: String,
    query: HashMap<String, String>,
}

impl Url {
    pub fn from_string_base(s: String) -> Self {
        Self {
            base: s,
            query: HashMap::new(),
        }
    }

    pub fn from_string_with_query(s: String) -> Self {
        let (base, query_raw) = s.split_at(s.find('?').unwrap_or(s.len()));

        let query = query_raw[1..]
            .split('&')
            .map(|e| e.split_at(e.find('=').unwrap_or(e.len())))
            .map(|(k, v)| (Self::decode_part(k), Self::decode_part(&v[1..])))
            .collect();

        Self {
            base: base.into(),
            query,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}{}{}",
            self.base,
            if self.query.is_empty() { "" } else { "?" },
            self.query
                .iter()
                .map(|(k, v)| format!("{}={}", Self::encode_part(k), Self::encode_part(v)))
                .intersperse(String::from("&"))
                .collect::<String>()
        )
    }

    fn decode_part(s: &str) -> String {
        s.into() // TODO
    }

    fn encode_part(s: &str) -> String {
        s.into() // TODO
    }

    pub fn query(&self) -> &HashMap<String, String> {
        &self.query
    }

    pub fn query_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.query
    }

    pub fn into_url(&self) -> reqwest::Url {
        self.into()
    }
}

impl From<&Url> for reqwest::Url {
    fn from(url: &Url) -> Self {
        Self::parse(&url.to_string()).unwrap()
    }
}

impl From<Url> for reqwest::Url {
    fn from(url: Url) -> Self {
        Self::from(&url)
    }
}
