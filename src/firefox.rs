use crate::config;
use crate::proc;
use crate::xwindow::XWindow;
use ini::Ini;
use libc::{sysconf, _SC_CLK_TCK};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use procfs::process;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FIREFOX_DIR: &str = ".mozilla/firefox";
const WARMUP_TIME: u64 = 300; // 300 secs to warm up

#[derive(Debug)]
struct Profile {
    name: String,
    path: String,
    is_relative: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
pub enum Mode {
    Auto,
    On,
    Off,
    Suspend,
    AsIs,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    StartingUp,
    ShuttingDown,
    Down,
    Warming,
    Warmed,
    Suspend,
}

#[derive(Debug)]
pub struct Firefox {
    name: String,
    pid: Option<i32>,
    state: State,
    is_top: bool,
    mode: Mode,
    pidlink: PathBuf,
}

impl Firefox {
    fn new(profile: Profile, mode: Mode) -> Self {
        let pidlink = if profile.is_relative {
            firefox_home().join(profile.path).join("lock")
        } else {
            PathBuf::from(profile.path).join("lock")
        };
        let name = profile.name;
        Firefox {
            name,
            pid: None,
            state: State::Down,
            is_top: false,
            mode,
            pidlink,
        }
    }

    pub fn update(&mut self, xwin: &XWindow) {
        let pid = get_pid(&self.pidlink);
        let (state, is_top) = match pid {
            Some(p) => (get_state(p, &self.name), Some(p) == xwin.top_pid),
            _ => (State::Down, false),
        };
        self.pid = pid;
        self.state = state;
        self.is_top = is_top
    }

    pub fn apply_mode(&mut self, xwin: &XWindow) {
        match self.state {
            State::Down => match self.mode {
                Mode::Auto | Mode::On | Mode::Suspend => {
                    self.launch();
                    self.state = State::StartingUp;
                }
                _ => (),
            },
            State::Warming => {
                if let Mode::Off = self.mode {
                    if let Some(pid) = self.pid {
                        xwin.close_pid(pid);
                        self.state = State::ShuttingDown;
                    }
                }
            }
            State::Warmed => {
                if !self.is_top && self.mode == Mode::Auto {
                    self.suspend();
                } else {
                    match self.mode {
                        Mode::Off => {
                            if let Some(pid) = self.pid {
                                xwin.close_pid(pid);
                                self.state = State::ShuttingDown;
                            }
                        }
                        Mode::Suspend => self.suspend(),
                        _ => (),
                    }
                }
            }
            State::Suspend => match self.mode {
                Mode::Auto if self.is_top => self.resume(),
                Mode::On => self.resume(),
                Mode::Off => {
                    self.resume();
                    if let Some(pid) = self.pid {
                        xwin.close_pid(pid);
                        self.state = State::ShuttingDown;
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn suspend(&self) {
        if let Some(pid) = self.pid {
            let _ = kill(Pid::from_raw(-pid), Signal::SIGSTOP);
        }
    }

    fn resume(&self) {
        if let Some(pid) = self.pid {
            let _ = kill(Pid::from_raw(-pid), Signal::SIGCONT);
        }
    }

    fn launch(&self) {
        proc::launch_firefox(&self.name);
    }
}

fn get_state(pid: i32, name: &str) -> State {
    let proc = process::Process::new(pid);
    match proc {
        Ok(p) if p.is_alive() => match p.cmdline() {
            Ok(v) if !v.is_empty() => {
                if v[0].ends_with("/firefox") && v.last().unwrap() == name {
                    let uptime = get_uptime(&p);
                    if uptime > WARMUP_TIME {
                        if let Ok(stat) = p.stat() {
                            if let Ok(process::ProcState::Stopped) = stat.state() {
                                State::Suspend
                            } else {
                                State::Warmed
                            }
                        } else {
                            State::Down
                        }
                    } else {
                        State::Warming
                    }
                } else {
                    State::Down
                }
            }
            _ => State::Down,
        },
        _ => State::Down,
    }
}

fn get_uptime(p: &process::Process) -> u64 {
    let nowsec = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    nowsec.as_secs()
        - p.stat.starttime / unsafe { sysconf(_SC_CLK_TCK) } as u64
        - procfs::boot_time_secs().unwrap()
}

fn get_pid(pidlink: &Path) -> Option<i32> {
    let link = fs::read_link(pidlink);
    match link {
        Ok(p) => Some(
            p.to_str()
                .unwrap()
                .to_string()
                .split('+')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .parse::<i32>()
                .unwrap(),
        ),
        _ => None,
    }
}

fn firefox_home() -> PathBuf {
    let homedir = env::var("HOME").unwrap();
    Path::new(&homedir).join(FIREFOX_DIR)
}

fn profiles() -> Vec<Profile> {
    let firefox_profile = firefox_home().join("profiles.ini");
    let i = Ini::load_from_file(firefox_profile).unwrap();
    let mut profiles: Vec<Profile> = vec![];
    for (sec, prop) in i.iter() {
        match sec {
            Some(s) if s.starts_with("Profile") => {
                let mut p: Profile = Profile {
                    name: String::from(""),
                    path: String::from(""),
                    is_relative: true,
                };
                for (k, v) in prop.iter() {
                    match k {
                        "Name" => p.name = v.to_string(),
                        "Path" => p.path = v.to_string(),
                        "IsRelative" => p.is_relative = matches!(v, "1"),
                        _ => continue,
                    }
                }
                profiles.push(p);
            }
            _ => continue,
        }
    }
    profiles
}

pub fn firefoxes(xwin: &XWindow) -> HashMap<String, Firefox> {
    let mut firefoxes = HashMap::<String, Firefox>::new();
    let configs = config::configs();
    for p in profiles() {
        let mode = match configs.iter().find(|&x| x.Name == p.name) {
            Some(c) => match c.Mode {
                Mode::None => Mode::AsIs,
                _ => c.Mode,
            },
            _ => Mode::AsIs,
        };
        let name = p.name.clone();
        let mut firefox = Firefox::new(p, mode);
        firefox.update(xwin);
        firefoxes.insert(name, firefox);
    }
    firefoxes
}
