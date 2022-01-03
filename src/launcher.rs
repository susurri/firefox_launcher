use std::collections::HashMap;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::firefox;
use crate::xwindow;

fn cmd_exec(cmd: &str, firefoxes: &mut HashMap<String, firefox::Firefox>, xwin: &xwindow::XWindow) {
    match cmd.split_whitespace().collect::<Vec<&str>>()[..] {
        ["set", name, mode] => match firefoxes.get_mut(name) {
            Some(f) => match firefox::Mode::from_str(mode) {
                Ok(m) => {
                    f.mode = m;
                    f.apply_mode(xwin)
                }
                _ => println!("No such mode {}", mode),
            },
            _ => println!("No profile name {} found", name),
        },
        ["list"] => firefoxes
            .iter()
            .for_each(|(name, firefox)| println!("{}  {:?}", name, firefox.mode)),
        _ => println!("Unknown command"),
    }
}

pub fn run(cmd_rx: &Receiver<String>) {
    let mut xwin = xwindow::XWindow::new();
    let mut firefoxes = firefox::firefoxes(&xwin);
    for (_, firefox) in firefoxes.iter_mut() {
        firefox.apply_mode(&xwin);
    }
    let mut prev_top_pid = xwin.top_pid();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let top_pid = xwin.top_pid();
        if top_pid != prev_top_pid {
            xwin.update();
            for (_, firefox) in firefoxes.iter_mut() {
                firefox.update(&xwin);
                firefox.apply_mode(&xwin);
            }
        }
        while let Ok(s) = cmd_rx.try_recv() {
            println!("{} received", s);
            cmd_exec(&s, &mut firefoxes, &xwin);
        }
        prev_top_pid = top_pid;
    }
}
