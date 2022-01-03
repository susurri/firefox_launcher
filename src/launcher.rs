use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::firefox;
use crate::xwindow;

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
        let data = cmd_rx.try_recv();
        if let Ok(s) = data {
            println!("{} received", s)
        }
        prev_top_pid = top_pid;
    }
}
