use std::error::Error;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use addr::parse_domain_name;

use clap::{arg, value_parser, Command};

use console::Term;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use url::Url;

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const ABOUT: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RunConfig {
    url: Option<String>,           // = "https://www.google.com/",
    url_list: Option<Vec<String>>, // = "url,url"
    config: Config,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    same_domain: bool,     // = true,
    same_sub_domain: bool, // = true,
    depth: u16,            // = 1,
    repeat: u16,           // = 1,
    users: u16,            // = 1,
    wait_ms: u16,          // = 500
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig {
            url: None,
            url_list: None,
            config: Config::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            same_domain: true,
            same_sub_domain: true,
            depth: 1,
            repeat: 1,
            users: 1,
            wait_ms: 500,
        }
    }
}

fn main() -> Result<(), ::std::io::Error> {
    let term = Term::stdout();
    term.clear_screen().unwrap();

    let cfg: RunConfig = get_config();

    let started = Instant::now();
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}").unwrap();
    let msg_style = ProgressStyle::with_template("{wide_msg}").unwrap();

    let mut thread_handles = Vec::<Option<JoinHandle<()>>>::new();

    println!("⌛ Spawning threads ({})", cfg.config.users);

    let m = MultiProgress::new();

    for i in 0..cfg.config.users {
        let pb = m.add(ProgressBar::new(10000));
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!("[{}/{}", i + 1, cfg.config.users));
        pb.inc(1);

        let t_cfg: RunConfig = cfg.clone();

        let join_handle = thread::spawn(move || {
            let _ = browse(&pb, &t_cfg);

            pb.finish_with_message("done...");
        });
        thread_handles.push(Some(join_handle));
    }

    let bottom_pb = m.add(ProgressBar::new(10000));
    bottom_pb.set_style(msg_style.clone());
    bottom_pb.set_message("press CTRL+C or 'q' for exit ...");
    bottom_pb.inc(1);

    loop {
        let quit = wait_for_quitkey(&term);
        if quit {
            break;
        }
    }

    while thread_handles.len() > 0 {
        let cur_thread = thread_handles.remove(0); // moves it into cur_thread
        let r = cur_thread.unwrap().join();
        handle_thread_result(r);
    }

    bottom_pb.finish_and_clear();
    println!("{} Done in {}", "*", HumanDuration(started.elapsed()));
    Ok(())
}

fn wait_for_quitkey(mut term: &Term) -> bool {
    let char = term.read_char();
    let key = match char {
        Ok(k) => k,
        Err(e) => {
            let _ = writeln!(term, "{}", e);
            ' '
        }
    };

    if key == 'q' {
        return true;
    }
    return false;
}

fn handle_thread_result(r: thread::Result<()>) {
    match r {
        Ok(_) => (),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<&'static str>() {
                println!("Got an error: {}", e);
            } else {
                println!("Got an unknown error: {:?}", e);
            }
        }
    }
}

fn get_config() -> RunConfig {
    let mut run_cfg: RunConfig = RunConfig::default();

    let matches = Command::new("Web Stress Tools")
        .version(VERSION.unwrap_or("unknown"))
        .author(AUTHOR.unwrap_or("Richard Straka <richard.straka@gmail.com>"))
        .about(ABOUT.unwrap_or("Generating synthetic web traffic for your app to help with benchmarking and debuging of performance issues."))
        // These two arguments are part of the "target" group which is required
        .arg(
            arg!(-u --"url" <URL> "starting URL for recursive browsing through extracted links on pages")
                .value_parser(value_parser!(String))
                .group("input").required(true),
        )
        .arg(
            arg!(-l --"url-list" <FILE> "file path to a list of URLs (one per line) to browse")
                .value_parser(value_parser!(PathBuf))
                .group("input").required(true),
        )
        .arg(
            arg!(-c --config <FILE> "file path to the TOML configuration, see Config.toml")
                .value_parser(value_parser!(PathBuf))
                .required(false)
        )
        .arg(
            arg!(--"same-domain" <VALUE> "filtering of extracted links from pages {true|false} (default: true)")
                .value_parser(value_parser!(bool))
                .required(false)
        )
        .arg(
            arg!(--"same-subdomain" <VALUE> "filtering of extracted links from pages {true|false} (default: true)")
                .value_parser(value_parser!(bool))
                .required(false)
        )
        .arg(
            arg!(--depth <VALUE> "how deep we want to go with recursive browsing (default: 1)")
                .value_parser(value_parser!(u16))
                .required(false)
        )
        .arg(
            arg!(--repeat <VALUE> "how many times we want to repeat browsing (default: 1)")
                .value_parser(value_parser!(u16))
                .required(false)
        )
        .arg(
            arg!(--users <VALUE> "number of simulated users (default: 1)")
                .value_parser(value_parser!(u16))
                .required(false)
        )
        .arg(
            arg!(--"wait-ms" <VALUE> "how many miliseconds we want to wait on each page (default: 500)")
                .value_parser(value_parser!(u16))
                .required(false)
        )
        .get_matches();

    if let Some(cfg_path) = matches.get_one::<PathBuf>("config") {
        run_cfg.config = confy::load_path(cfg_path).unwrap();
    }

    if let Some(url) = matches.get_one::<String>("url") {
        run_cfg.url = Some(url.to_string());
    } else if let Some(url_list_file) = matches.get_one::<PathBuf>("url-list") {
        let mut url_list = Vec::<String>::new();
        for line in read_to_string(url_list_file).unwrap().lines() {
            url_list.push(line.to_string())
        }
        run_cfg.url_list = Some(url_list);
    }

    if let Some(same_domain) = matches.get_one::<bool>("same-domain") {
        run_cfg.config.same_domain = same_domain.clone();
    }

    if let Some(same_sub_domain) = matches.get_one::<bool>("same-subdomain") {
        run_cfg.config.same_sub_domain = same_sub_domain.clone();
    }

    if let Some(depth) = matches.get_one::<u16>("depth") {
        run_cfg.config.depth = depth.clone();
    }

    if let Some(repeat) = matches.get_one::<u16>("repeat") {
        run_cfg.config.repeat = repeat.clone();
    }

    if let Some(users) = matches.get_one::<u16>("users") {
        run_cfg.config.users = users.clone();
    }

    if let Some(wait_ms) = matches.get_one::<u16>("wait-ms") {
        run_cfg.config.wait_ms = wait_ms.clone();
    }
    run_cfg.config.wait_ms = if run_cfg.config.wait_ms > 0 {
        run_cfg.config.wait_ms
    } else {
        1
    };

    //dbg!(&run_cfg);

    println!("🔍 Configuration loaded");
    if run_cfg.url.is_some() {
        println!(
            "Action:            Browsing pages recursively from {}",
            run_cfg.url.clone().unwrap()
        );
        println!("Same domain:       {}", run_cfg.config.same_domain);
        println!("Same sub-domain:   {}", run_cfg.config.same_sub_domain);
        println!("Depth:             {}", run_cfg.config.depth);
    } else if run_cfg.url_list.is_some() {
        println!(
            "Action:          Browse over list of {} URLs",
            run_cfg.url_list.clone().unwrap().len()
        );
    }
    println!("Repeat:            {} time(s)", run_cfg.config.repeat);
    println!("Simulated users:   {}", run_cfg.config.users);
    println!("Wait on each page: {} ms", run_cfg.config.wait_ms);

    return run_cfg;
}

fn extract_links(page: &no_browser::page::Page, cfg: &RunConfig) -> Vec<String> {
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

fn wait_with_random(ms: u16) {
    let rnd_ms = Duration::from_millis(rand::thread_rng().gen_range(0..u64::from(ms)));
    thread::sleep(Duration::from_millis(u64::from(ms)) + rnd_ms);
}

fn browse(pb: &ProgressBar, cfg: &RunConfig) -> Result<(), Box<dyn Error>> {
    pb.set_message("Loading chrome...");
    pb.inc(1);
    let browser = no_browser::Browser::builder().finish()?;
    pb.inc(1);

    let depth: u16 = 0;
    pb.inc(1);

    let mut index: u16 = 0;
    loop {
        index += 1;

        if let Some(url) = &cfg.url {
            browse_recursive(url, depth, &browser, &pb, &cfg);
        } else if let Some(url_list) = &cfg.url_list {
            browse_list(url_list, &browser, &pb, &cfg);
        }

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
    cfg: &RunConfig,
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

    let links = extract_links(&page.unwrap(), &cfg);

    pb.set_message(format!("found {} links", links.len()));
    pb.inc(1);
    wait_with_random(500);
    pb.inc(1);

    for link in links {
        browse_recursive(&link, next_depth, &browser, &pb, &cfg);
    }
}

fn browse_list(
    url_list: &Vec<String>,
    browser: &no_browser::browser::Browser,
    pb: &ProgressBar,
    cfg: &RunConfig,
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
