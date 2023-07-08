use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use addr::{parse_dns_name, parse_domain_name};

use console::Style;
use console::Term;
use console::{style, Emoji};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

//use headless_chrome::browser::Tab;
//use headless_chrome::Browser;

use url::{Host, Position, Url};

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
            //let term_inner = Term::stdout();
            //let _ = term_inner.write_line(&format!("Thread {} started.", i));

            /*for x in 0..count {
                thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(25..200)));
                pb.set_message(format!("{x}: {count}"));
                //pb.inc(1);
            }*/

            let _ = browse(&pb, &t_cfg);

            //thread::sleep(Duration::from_millis(1000 - i as u64 * 100));

            if i == 3 {
                //println!("Thread {} panic.", i);
                //panic!();
            }

            pb.finish_with_message("done...");

            //println!("Thread {} finished.", i);

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

    //let cyan = Style::new().cyan();
    //println!("This is {} neat", cyan.apply_to("quite"));

    //term.write_line("Hello World!");
    //thread::sleep(Duration::from_millis(2000));
    //term.clear_line();

    //let _ = browse_domain();
    loop {
        //thread::sleep(Duration::from_millis(5000));
        let quit = wait_for_quitkey(&term);
        if quit {
            break;
        }
    }

    //for tj in thread_handles.into_iter() {
    //    let _ = tj.unwrap().join();
    //}

    while thread_handles.len() > 0 {
        let cur_thread = thread_handles.remove(0); // moves it into cur_thread
        let r = cur_thread.unwrap().join();
        handle_thread_result(r);
    }

    //while let Some(cur_thread) = thread_handles.pop() {
    //    let _ = cur_thread.unwrap().join();
    //}
    //m.clear().unwrap();
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

/*fn get_attr(elt: &headless_chrome::Element, attr: &str) -> String {
    let attributes = elt.get_attributes();

    let mut is_match = false;

    if let Ok(Some(array)) = attributes {
        for item in array.iter() {
            if is_match {
                return item.to_string();
            }
            if item == attr {
                is_match = true;
            }
        }
    }

    return "".to_string();
}*/

/*fn extract_links(tab: &Tab, cfg: &Config) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(elems) = tab.wait_for_elements("a[href]") {
        let start_url = Url::parse(cfg.start_url.as_str()).unwrap();
        let start_domain = parse_domain_name(start_url.host_str().unwrap()).unwrap();
        let start_domain_prefix = start_domain.prefix();
        let start_domain_root = start_domain.root().unwrap();

        for elem in elems {
            let link = get_attr(&elem, "href");
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
}*/

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
    //let browser = Browser::default()?;
    //let _ = browser.wait_for_initial_tab().unwrap();
    pb.inc(1);

    //let tab = browser.new_tab()?;
    let depth: u16 = 0;
    pb.inc(1);

    loop {
        browse_recursive(&cfg.start_url, depth, &browser /*&tab*/, &pb, &cfg);
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
    browser: &no_browser::browser::Browser, /*&Browser*/
    /*tab: &Tab*/
    pb: &ProgressBar,
    cfg: &Config,
) {
    if depth > cfg.target_depth {
        return;
    }

    // Navigate to the url
    pb.set_message(format!("Loading {}", url));
    //println!("Loading {}", url);

    pb.inc(1);
    //let tab = browser.new_tab().unwrap();

    //let tab = browser.new_tab().unwrap();
    let page = browser.navigate_to(url, None);
    match page {
        Ok(_) => (),
        Err(e) => {
            println!("Err: {}", e);
            return;
        }
    }

    //dbg!(page);
    //if let Err(e) = tab.navigate_to(url) {
    //    dbg!(e);
    //}

    //if let Err(e) = tab.wait_until_navigated() {
    //    dbg!(e);
    //}

    //let nav = tab.navigate_to(url);
    //let _ = tab.wait_for_element("body");
    //match nav {
    //    Ok(n) => match n.get_title() {
    //        Ok(t) => println!("title: {}", t),
    //        Err(e) => println!("title err: {}", e),
    //    },
    //    Err(e) => println!("nav err: {}", e),
    //}
    //println!("title: {}", title);
    //println!("{}", nav.unwrap().get_document().unwrap());

    thread::sleep(Duration::from_millis(u64::from(cfg.wait_miliseconds)));
    pb.inc(1);

    //TODO: we can do something on the page...
    // Wait for network/javascript/dom to make the search-box available
    // and click it.
    //tab.wait_for_element("input#searchInput")?.click()?;

    // Type in a query and press `Enter`
    //tab.type_str("WebKit")?.press_key("Enter")?;

    //println!("{:#?}", elems);
    //assert!(tab.get_url().ends_with("WebKit"));

    // Take a screenshot of the entire browser window
    //let _jpeg_data =
    //    tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Jpeg, None, None, true)?;

    // Take a screenshot of just the WebKit-Infobox
    //let _png_data = tab
    //    .wait_for_element("#mw-content-text > div > table.infobox.vevent")?
    //    .capture_screenshot(Page::CaptureScreenshotFormatOption::Png)?;

    // Run JavaScript in the page
    //let remote_object = elem.call_js_fn(
    //    r#"
    //    function getIdTwice () {
    //        // `this` is always the element that you called `call_js_fn` on
    //        const id = this.id;
    //        return id + id;
    //    }
    //"#,
    //    vec![],
    //    false,
    //)?;
    //match remote_object.value {
    //    Some(returned_string) => {
    //        dbg!(&returned_string);
    //        assert_eq!(returned_string, "firstHeadingfirstHeading".to_string());
    //    }
    //    _ => unreachable!(),
    //};

    let next_depth: u16 = depth + 1;
    if next_depth > cfg.target_depth {
        return;
    }

    let links = extract_links(&page.unwrap(), &cfg);
    //let links = extract_links(&tab, &cfg);

    //println!("{}", links.len());
    pb.set_message(format!("found {} links", links.len()));
    pb.inc(1);
    pb.inc(1);

    //if let Err(e) = tab.close(false) {
    //    dbg!(e);
    //}

    for link in links {
        //pb.set_message(format!("{}", link));
        //pb.inc(1);
        //println!();
        //thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(25..100)));
        browse_recursive(&link, next_depth, &browser /*&tab*/, &pb, &cfg);
    }
}
