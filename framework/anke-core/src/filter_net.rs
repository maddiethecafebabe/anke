#[derive(Debug, Clone)]
pub struct FilterNet {
    pub hosts: Vec<String>,
}

impl FilterNet {
    pub fn new<I, S>(hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            hosts: hosts.into_iter().map(|s| s.into()).collect(),
        }
    }

    fn matches(filter_pattern: &str, host: &str) -> bool {
        filter_pattern == host
    }

    pub fn any_matches_on(&self, site: &str) -> bool {
        self.hosts
            .iter()
            .any(|pattern| Self::matches(pattern, site))
    }
}

impl<I, S> From<I> for FilterNet
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    fn from(v: I) -> FilterNet {
        FilterNet::new(v)
    }
}
