use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::firefox;
use crate::xwindow;

pub fn run(cmd_rx: &Receiver<String>) {
    let firefoxes = firefox::firefoxes();
    let xwin = xwindow::XWindow::new();
    println!("{:?}", xwin.clients());
    println!("firefoxes = {:?}", firefoxes);
    let mut prev_top_pid = xwin.top_pid();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let top_pid = xwin.top_pid();
        if top_pid != prev_top_pid {
            println!("top pid changed from {:?} to {:?}", prev_top_pid, top_pid);
        }
        let data = cmd_rx.try_recv();
        if let Ok(s) = data {
            println!("{} received", s)
        }
        prev_top_pid = top_pid;
    }
}
