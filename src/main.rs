use std::io::Write;

use std::thread::{self, JoinHandle};
use std::time::Instant;

use console::Term;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};

mod config;
mod html_helper;
mod simple_browser;

fn main() -> Result<(), ::std::io::Error> {
    let term = Term::stdout();
    term.clear_screen()?;

    let cfg: config::RunConfig = config::get_config();

    let started = Instant::now();
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}")
            .expect("Expected style!");
    let msg_style = ProgressStyle::with_template("{wide_msg}").expect("Expected style!");

    let mut thread_handles = Vec::<Option<JoinHandle<()>>>::new();

    println!("⌛ Spawning threads ({})", cfg.config.users);

    let m = MultiProgress::new();

    for i in 0..cfg.config.users {
        let pb = m.add(ProgressBar::new(10000));
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!("[{}/{}", i + 1, cfg.config.users));
        pb.inc(1);

        let t_cfg: config::RunConfig = cfg.clone();

        let join_handle = thread::spawn(move || {
            let _ = simple_browser::browse(&pb, &t_cfg);

            pb.finish_with_message("done...");
        });
        thread_handles.push(Some(join_handle));
    }

    let bottom_pb = m.add(ProgressBar::new(10000));
    bottom_pb.set_style(msg_style);
    bottom_pb.set_message("press CTRL+C or 'q' for exit ...");
    bottom_pb.inc(1);

    loop {
        let quit = wait_for_quitkey(&term);
        if quit {
            break;
        }
    }

    while !thread_handles.is_empty() {
        let cur_thread = thread_handles.remove(0); // moves it into cur_thread
        let r = cur_thread.expect("Expected thread!").join();
        handle_thread_result(r);
    }

    bottom_pb.finish_and_clear();
    println!("* Done in {}", HumanDuration(started.elapsed()));
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
    false
}

fn handle_thread_result(r: thread::Result<()>) {
    match r {
        Ok(_) => (),
        Err(e) => {
            e.downcast_ref::<&'static str>().map_or_else(
                || {
                    println!("Got an unknown error: {:?}", e);
                },
                |e| {
                    println!("Got an error: {}", e);
                },
            );
        }
    }
}
