use std::fs::read_to_string;

use std::path::PathBuf;

use clap::{arg, value_parser, Command};

use serde_derive::{Deserialize, Serialize};

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const AUTHOR: Option<&str> = option_env!("CARGO_PKG_AUTHORS");
const ABOUT: Option<&str> = option_env!("CARGO_PKG_DESCRIPTION");

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct RunConfig {
    pub url: Option<String>,           // = "https://www.google.com/",
    pub url_list: Option<Vec<String>>, // = "url,url"
    pub config: Config,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub same_domain: bool,     // = true,
    pub same_sub_domain: bool, // = true,
    pub depth: u16,            // = 1,
    pub repeat: u16,           // = 1,
    pub users: u16,            // = 1,
    pub wait_ms: u16,          // = 500
}

impl Default for Config {
    fn default() -> Self {
        Self {
            same_domain: true,
            same_sub_domain: true,
            depth: 1,
            repeat: 1,
            users: 1,
            wait_ms: 500,
        }
    }
}

pub fn get_config() -> RunConfig {
    let mut run_cfg: RunConfig = RunConfig::default();

    let matches = get_matches();
    if let Some(cfg_path) = matches.get_one::<PathBuf>("config") {
        run_cfg.config = confy::load_path(cfg_path)
            .expect("Error loading configuration from the path {cfg_path}");
    }

    if let Some(url) = matches.get_one::<String>("url") {
        run_cfg.url = Some(url.to_string());
    } else if let Some(url_list_file) = matches.get_one::<PathBuf>("url-list") {
        let mut url_list = Vec::<String>::new();
        for line in read_to_string(url_list_file)
            .expect("Error loading URLs from the file {url_list_file}")
            .lines()
        {
            url_list.push(line.to_string())
        }
        run_cfg.url_list = Some(url_list);
    }

    if let Some(same_domain) = matches.get_one::<bool>("same-domain") {
        run_cfg.config.same_domain = *same_domain;
    }

    if let Some(same_sub_domain) = matches.get_one::<bool>("same-subdomain") {
        run_cfg.config.same_sub_domain = *same_sub_domain;
    }

    if let Some(depth) = matches.get_one::<u16>("depth") {
        run_cfg.config.depth = *depth;
    }

    if let Some(repeat) = matches.get_one::<u16>("repeat") {
        run_cfg.config.repeat = *repeat;
    }

    if let Some(users) = matches.get_one::<u16>("users") {
        run_cfg.config.users = *users;
    }

    if let Some(wait_ms) = matches.get_one::<u16>("wait-ms") {
        run_cfg.config.wait_ms = *wait_ms;
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
            run_cfg.url.clone().expect("Expected URL!")
        );
        println!("Same domain:       {}", run_cfg.config.same_domain);
        println!("Same sub-domain:   {}", run_cfg.config.same_sub_domain);
        println!("Depth:             {}", run_cfg.config.depth);
    } else if run_cfg.url_list.is_some() {
        println!(
            "Action:          Browse over list of {} URLs",
            run_cfg
                .url_list
                .clone()
                .expect("Expected list of URLs!")
                .len()
        );
    }
    println!("Repeat:            {} time(s)", run_cfg.config.repeat);
    println!("Simulated users:   {}", run_cfg.config.users);
    println!("Wait on each page: {} ms", run_cfg.config.wait_ms);

    run_cfg
}

fn get_matches() -> clap::ArgMatches {
    let mut cmd = get_default_args();
    cmd = get_url_arg(cmd);
    cmd = get_url_list_arg(cmd);
    cmd = get_config_arg(cmd);
    cmd = get_domain_arg(cmd);
    cmd = get_subdomain_arg(cmd);
    cmd = get_depth_arg(cmd);
    cmd = get_repeat_arg(cmd);
    cmd = get_users_arg(cmd);
    cmd = get_wait_arg(cmd);
    cmd.get_matches()
}

fn get_default_args() -> clap::Command {
    Command::new("Web Stress Tools")
        .version(VERSION.unwrap_or("unknown"))
        .author(AUTHOR.unwrap_or("Richard Straka <richard.straka@gmail.com>"))
        .about(ABOUT.unwrap_or("Generating synthetic web traffic for your app to help with benchmarking and debuging of performance issues."))
}

fn get_url_arg(cmd: clap::Command) -> clap::Command {
    cmd
        .arg(
            arg!(-u --"url" <URL> "starting URL for recursive browsing through extracted links on pages")
                .value_parser(value_parser!(String)).group("input").required(true))
}

fn get_url_list_arg(cmd: clap::Command) -> clap::Command {
    cmd.arg(
        arg!(-l --"url-list" <FILE> "file path to a list of URLs (one per line) to browse")
            .value_parser(value_parser!(PathBuf))
            .group("input")
            .required(true),
    )
}

fn get_config_arg(cmd: clap::Command) -> clap::Command {
    cmd.arg(
        arg!(-c --config <FILE> "file path to the TOML configuration, see Config.toml")
            .value_parser(value_parser!(PathBuf))
            .required(false),
    )
}

fn get_domain_arg(cmd: clap::Command) -> clap::Command {
    cmd
        .arg(
            arg!(--"same-domain" <VALUE> "filtering of extracted links from pages {true|false} (default: true)")
                .value_parser(value_parser!(bool)).required(false))
}

fn get_subdomain_arg(cmd: clap::Command) -> clap::Command {
    cmd
        .arg(
            arg!(--"same-subdomain" <VALUE> "filtering of extracted links from pages {true|false} (default: true)")
                .value_parser(value_parser!(bool)).required(false))
}

fn get_depth_arg(cmd: clap::Command) -> clap::Command {
    cmd.arg(
        arg!(--depth <VALUE> "how deep we want to go with recursive browsing (default: 1)")
            .value_parser(value_parser!(u16))
            .required(false),
    )
}
fn get_repeat_arg(cmd: clap::Command) -> clap::Command {
    cmd.arg(
        arg!(--repeat <VALUE> "how many times we want to repeat browsing (default: 1)")
            .value_parser(value_parser!(u16))
            .required(false),
    )
}
fn get_users_arg(cmd: clap::Command) -> clap::Command {
    cmd.arg(
        arg!(--users <VALUE> "number of simulated users (default: 1)")
            .value_parser(value_parser!(u16))
            .required(false),
    )
}
fn get_wait_arg(cmd: clap::Command) -> clap::Command {
    cmd
        .arg(
            arg!(--"wait-ms" <VALUE> "how many miliseconds we want to wait on each page (default: 500)")
                .value_parser(value_parser!(u16)).required(false))
}
