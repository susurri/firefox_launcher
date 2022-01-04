use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::process::exit;
use std::sync::mpsc;
use std::thread;

mod config;
mod firefox;
mod launcher;
mod lock;
mod proc;
mod xwindow;

fn main() {
    let lockfile = lock::Lockfile::new();
    if !lockfile.is_single {
        eprintln!("Another firefox-launcher is running");
        exit(1);
    }
    let (cmd_tx, cmd_rx) = mpsc::channel();
    thread::spawn(move || {
        launcher::run(&cmd_rx);
    });
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                cmd_tx.send(line.clone()).unwrap();
                rl.add_history_entry(line.as_str());
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
