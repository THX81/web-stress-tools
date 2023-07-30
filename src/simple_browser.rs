use std::error::Error;

use std::thread;
use std::time::Duration;

use indicatif::ProgressBar;
use rand::Rng;

use crate::config;
use crate::html_helper;

pub fn browse(pb: &ProgressBar, cfg: &config::RunConfig) -> Result<(), Box<dyn Error>> {
    pb.set_message("Loading chrome...");
    pb.inc(1);
    let browser = no_browser::Browser::builder().finish()?;
    pb.inc(1);

    let depth: u16 = 0;
    pb.inc(1);

    let mut index: u16 = 0;
    loop {
        index += 1;

        cfg.url.as_ref().map_or_else(
            || {
                if let Some(url_list) = &cfg.url_list {
                    browse_list(url_list, &browser, pb, cfg);
                }
            },
            |url| {
                browse_recursive(url, depth, &browser, pb, cfg);
            },
        );

        pb.inc(1);

        if index >= cfg.config.repeat {
            break;
        }
    }

    Ok(())
}

fn browse_recursive(
    url: &str,
    depth: u16,
    browser: &no_browser::browser::Browser,
    pb: &ProgressBar,
    cfg: &config::RunConfig,
) {
    if depth > cfg.config.depth {
        return;
    }

    pb.set_message(format!("Loading {}", url));
    //println!("Loading {}", url);
    pb.inc(1);

    // Navigate to the url
    let page = browser.navigate_to(url, None);
    match page {
        Ok(_) => (),
        Err(e) => {
            println!("Err: {}", e);
            return;
        }
    }
    pb.inc(1);

    let next_depth: u16 = depth + 1;
    if next_depth > cfg.config.depth {
        return;
    }

    wait_with_random(cfg.config.wait_ms);

    let links = html_helper::extract_links(&page.expect("Expected page!"), cfg);

    pb.set_message(format!("found {} links", links.len()));
    pb.inc(1);
    wait_with_random(500);
    pb.inc(1);

    for link in links {
        browse_recursive(&link, next_depth, browser, pb, cfg);
    }
}

fn browse_list(
    url_list: &Vec<String>,
    browser: &no_browser::browser::Browser,
    pb: &ProgressBar,
    cfg: &config::RunConfig,
) {
    for url in url_list {
        pb.set_message(format!("Loading {}", url));
        //println!("Loading {}", url);
        pb.inc(1);

        // Navigate to the url
        let page = browser.navigate_to(url, None);
        match page {
            Ok(_) => (),
            Err(e) => {
                println!("Err: {}", e);
                return;
            }
        }

        wait_with_random(cfg.config.wait_ms);
        pb.inc(1);
    }
}

fn wait_with_random(ms: u16) {
    let rnd_ms = Duration::from_millis(rand::thread_rng().gen_range(0..u64::from(ms)));
    thread::sleep(Duration::from_millis(u64::from(ms)) + rnd_ms);
}
