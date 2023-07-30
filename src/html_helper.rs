use addr::parse_domain_name;

use rand::seq::SliceRandom;
use rand::thread_rng;

use url::Url;

use crate::config;

pub fn extract_links(page: &no_browser::page::Page, cfg: &config::RunConfig) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(elems) = page.select("a[href]") {
        let start_url = Url::parse(cfg.url.as_ref().expect("Expected URL reference!").as_str())
            .expect("URL parse failed!");
        let start_url_scheme = start_url.scheme();
        let start_domain = parse_domain_name(start_url.host_str().expect("Expected URL host!"))
            .expect("Domain parse failed!");
        let start_domain_prefix = start_domain.prefix();
        let start_domain_root = start_domain.root().unwrap_or(start_domain.as_str());

        for elem in elems {
            let href = elem.value().attr("href").get_or_insert("").to_string();

            let link: String = if href.starts_with('/') {
                start_url
                    .join(href.as_str())
                    .expect("Expected full URL!")
                    .to_string()
            } else {
                href
            };

            let url_result = Url::parse(link.as_str());
            match url_result {
                Ok(_) => (),
                Err(_) => continue,
            }

            let url = url_result.expect("Expected URL value!");
            let scheme = url.scheme();
            let host = url.host_str();
            match host {
                Some(_) => (),
                None => continue,
            }
            let domain =
                parse_domain_name(host.expect("No host value!")).expect("Domain not parsed!");
            let domain_prefix = domain.prefix();
            let domain_root = domain.root().unwrap_or(domain.as_str());

            if scheme != "http" && scheme != "https" {
                continue;
            }

            if scheme != start_url_scheme {
                continue;
            }

            if cfg.config.same_domain && domain_root != start_domain_root {
                continue;
            }

            if cfg.config.same_sub_domain && domain_prefix != start_domain_prefix {
                continue;
            }

            result.push(link);
        }
    }

    result.shuffle(&mut thread_rng());

    result
}
