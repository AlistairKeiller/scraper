use anyhow::Result;
use csv::Writer;
use futures::StreamExt;
use reqwest::Url;
use std::collections::HashSet;
use voyager::scraper::Selector;
use voyager::{Collector, Crawler, CrawlerConfig, Response, Scraper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    struct Explorer {
        visited: HashSet<Url>,
        link_selector: Selector,
        paragraph_selector: Selector,
    }
    impl Default for Explorer {
        fn default() -> Self {
            Self {
                visited: Default::default(),
                link_selector: Selector::parse("a").unwrap(),
                paragraph_selector: Selector::parse("p").unwrap(),
            }
        }
    }

    impl Scraper for Explorer {
        type Output = (usize, Url);
        type State = bool; // empty

        fn scrape(
            &mut self,
            response: Response<Self::State>,
            crawler: &mut Crawler<Self>,
        ) -> Result<Option<Self::Output>> {
            std::fs::create_dir_all(["data/", response.request_url.path()].concat())?;
            let mut wtr =
                Writer::from_path(["data/", response.request_url.path(), "data.csv"].concat())?;
            for paragraph in response.html().select(&self.paragraph_selector) {
                wtr.write_record(&[paragraph.inner_html()])?;
            }
            wtr.flush()?;
            for link in response.html().select(&self.link_selector) {
                if let Some(href) = link.value().attr("href") {
                    if let Ok(url) = response.response_url.join(href) {
                        if self.visited.contains(&url) {
                            crawler.visit(url);
                        } else {
                            self.visited.insert(url);
                        }
                    }
                }
            }

            Ok(Some((response.depth, response.response_url)))
        }
    }

    let config = CrawlerConfig::default().allow_domain("ivypanda.com");
    let mut collector = Collector::new(Explorer::default(), config);

    collector
        .crawler_mut()
        .visit("https://ivypanda.com/essays/all/");

    while let Some(output) = collector.next().await {
        if let Ok((depth, url)) = output {
            println!("Visited {} at depth: {}", url, depth);
        }
    }

    Ok(())
}
