use anke_core::{log, FilterNet, OutputFilter, Pipeline, Sieve};

use super::HostsConfig;

pub struct App {
    mapping: Vec<(String, String, Box<dyn Sieve>)>,
    result_filters: Vec<Box<dyn OutputFilter>>,
    hosts: HostsConfig,
}

impl App {
    pub fn new(hosts: HostsConfig) -> Self {
        Self {
            hosts: hosts,
            mapping: Vec::new(),
            result_filters: Vec::new(),
        }
    }

    pub fn map_patterns_to_sieve<FN: Into<FilterNet>, F: Sieve + Clone + 'static>(
        mut self,
        net: FN,
        sieve: F,
    ) -> Self {
        let net = net.into();
        if let Some((site, tags)) = self
            .hosts
            .sites
            .iter()
            .find(|(site, _)| net.any_matches_on(site))
        {
            let sites = vec![site.clone(); tags.len()];
            let tags = tags.clone();
            let sieves = vec![Box::new(sieve); tags.len()];

            for (site, (tag, sieve)) in sites
                .into_iter()
                .zip(tags.into_iter().zip(sieves.into_iter()))
            {
                self.mapping.push((site, tag, sieve));
            }
        } else {
            log::warn!("Couldn't find a site for mapping");
        }
        self
    }

    pub fn add_result_filter<F: OutputFilter + 'static>(mut self, filter: F) -> Self {
        self.result_filters.push(Box::new(filter));
        self
    }

    pub async fn run_forever(self) -> ! {
        Pipeline::new(self.result_filters)
            .spawn_receiver_task()
            .await
            .spawn_sender_tasks(self.mapping)
            .await
            .loop_endless()
            .await
    }
}
