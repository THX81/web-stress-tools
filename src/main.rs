use std::error::Error;
use std::io::Write;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use addr::parse_domain_name;

use console::Emoji;
use console::Term;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use serde_derive::{Deserialize, Serialize};

use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    start_url: String,          // = "https://www.google.com/",
    only_same_domain: bool,     // = true,
    only_same_sub_domain: bool, // = true,
    target_depth: u16,          // = 1,
    repeat: bool,               // = false,
    users: usize,               // = 5,
    wait_miliseconds: u16,      // = 5000
}

impl Default for Config {
    fn default() -> Self {
        Config {
            start_url: "".to_string(),
            only_same_domain: true,
            only_same_sub_domain: true,
            target_depth: 1,
            repeat: false,
            users: 1,
            wait_miliseconds: 1000,
        }
    }
}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");

fn main() -> Result<(), ::std::io::Error> {
    let term = Term::stdout();
    term.clear_screen().unwrap();

    println!("press 'q' for exit ...");

    println!("{} Loading configuration...", LOOKING_GLASS);

    let cfg: Config = get_config();

    let started = Instant::now();
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}").unwrap();

    let mut thread_handles = Vec::<Option<JoinHandle<()>>>::new();
    //let (send_finished_thread, receive_finished_thread) = std::sync::mpsc::channel();

    println!("⌛ Spawning threads ({})", cfg.users);

    let m = MultiProgress::new();
    for i in 0..cfg.users {
        //let send_finished_thread = send_finished_thread.clone();

        let pb = m.add(ProgressBar::new(10000));
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!("[{}/{}", i + 1, cfg.users));
        pb.inc(1);

        let t_cfg: Config = cfg.clone();

        let join_handle = thread::spawn(move || {
            let _ = browse(&pb, &t_cfg);

            pb.finish_with_message("done...");

            // Signal that we are finished.
            // This will wake up the main thread.
            //send_finished_thread.send(i).unwrap();
        });
        thread_handles.push(Some(join_handle));
    }

    /*loop {
        // Check if all threads are finished
        let num_left = threads.iter().filter(|th| th.is_some()).count();
        if num_left == 0 {
            break;
        }

        // Wait until a thread is finished, then join it
        let i = receive_finished_thread.recv().unwrap();
        let join_handle = std::mem::take(&mut threads[i]).unwrap();
        println!("Joining {} ...", i);
        join_handle.join().unwrap();
        println!("{} joined.", i);
    }*/

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
        Ok(r) => println!("All is well! {:?}", r),
        Err(e) => {
            if let Some(e) = e.downcast_ref::<&'static str>() {
                println!("Got an error: {}", e);
            } else {
                println!("Got an unknown error: {:?}", e);
            }
        }
    }
}

fn get_config() -> Config {
    let cfg: Config = confy::load_path("Config.toml").unwrap();
    //println!("{:#?}", cfg);
    //dbg!(&cfg);
    //let file = confy::get_configuration_file_path("web_stress_tools", "Config").unwrap();
    //println!("The configuration file path is: {:#?}", file);
    //let args: Vec<String> = env::args().collect();
    //println!("args: {:#?}", args);

    return cfg;
}

fn extract_links(page: &no_browser::page::Page, cfg: &Config) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(elems) = page.select("a[href]") {
        let start_url = Url::parse(cfg.start_url.as_str()).unwrap();
        let start_domain = parse_domain_name(start_url.host_str().unwrap()).unwrap();
        let start_domain_prefix = start_domain.prefix();
        let start_domain_root = start_domain.root().unwrap();

        for elem in elems {
            let link = elem.value().attr("href").get_or_insert("").to_string();
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
            let domain_root = domain.root().unwrap();

            if scheme != "http" && scheme != "https" {
                continue;
            }

            if cfg.only_same_domain && domain_root != start_domain_root {
                continue;
            }

            if cfg.only_same_sub_domain && domain_prefix != start_domain_prefix {
                continue;
            }

            result.push(link);
        }
    }

    result
}

fn browse(pb: &ProgressBar, cfg: &Config) -> Result<(), Box<dyn Error>> {
    pb.set_message("Loading chrome...");
    pb.inc(1);
    let browser = no_browser::Browser::builder().finish()?;
    pb.inc(1);

    let depth: u16 = 0;
    pb.inc(1);

    loop {
        browse_recursive(&cfg.start_url, depth, &browser, &pb, &cfg);
        pb.inc(1);
        if !cfg.repeat {
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
    cfg: &Config,
) {
    if depth > cfg.target_depth {
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

    thread::sleep(Duration::from_millis(u64::from(cfg.wait_miliseconds)));
    pb.inc(1);

    let next_depth: u16 = depth + 1;
    if next_depth > cfg.target_depth {
        return;
    }

    let links = extract_links(&page.unwrap(), &cfg);

    pb.set_message(format!("found {} links", links.len()));
    pb.inc(1);
    pb.inc(1);

    for link in links {
        browse_recursive(&link, next_depth, &browser, &pb, &cfg);
    }
}
