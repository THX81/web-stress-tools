use addr::parse_domain_name;

use rand::seq::SliceRandom;
use rand::thread_rng;

use url::Url;

use crate::config;

pub fn extract_links(page: &no_browser::page::Page, cfg: &config::RunConfig) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(elems) = page.select("a[href]") {
        let start_url = Url::parse(cfg.url.as_ref().unwrap().as_str()).unwrap();
        let start_url_scheme = start_url.scheme();
        let start_domain = parse_domain_name(start_url.host_str().unwrap()).unwrap();
        let start_domain_prefix = start_domain.prefix();
        let start_domain_root = start_domain.root().unwrap_or(start_domain.as_str());

        for elem in elems {
            let link: String;
            let href = elem.value().attr("href").get_or_insert("").to_string();

            if href.starts_with("/") {
                link = start_url.join(href.as_str()).unwrap().to_string();
            } else {
                link = href;
            }

            let url_result = Url::parse(link.as_str());
            match url_result {
                Ok(_) => (),
                Err(_) => continue,
            }

            let url = url_result.unwrap();
            let scheme = url.scheme();
            let host = url.host_str();
            match host {
                Some(_) => (),
                None => continue,
            }
            let domain = parse_domain_name(host.unwrap()).unwrap();
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
