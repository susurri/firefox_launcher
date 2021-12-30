use crate::config;
use ini::Ini;
use libc::{sysconf, _SC_CLK_TCK};
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

#[derive(Debug, Clone, Copy, Deserialize)]
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
}

#[derive(Debug)]
pub struct Firefox {
    name: String,
    pid: i32,
    state: State,
    is_top: bool,
    mode: Mode,
    pidlink: PathBuf,
}

impl Firefox {
    fn new(profile: Profile, mode: Mode) -> Firefox {
        let pidlink = if profile.is_relative {
            firefox_home().join(profile.path).join("lock")
        } else {
            PathBuf::from(profile.path).join("lock")
        };
        Firefox {
            name: profile.name,
            pid: 0,
            state: State::Down,
            is_top: false,
            mode,
            pidlink,
        }
        .update()
    }

    fn update(&self) -> Self {
        let pid = get_pid(&self.pidlink);
        let state = get_state(pid, &self.name);
        let is_top = is_top(pid);
        Firefox {
            name: self.name.clone(),
            pid,
            state,
            is_top,
            mode: self.mode,
            pidlink: self.pidlink.clone(),
        }
    }
}

fn is_top(pid: i32) -> bool {
    false
}

fn get_state(pid: i32, name: &str) -> State {
    let proc = process::Process::new(pid);
    match proc {
        Ok(p) if p.is_alive() => match p.cmdline() {
            Ok(v) if !v.is_empty() => {
                if v[0].ends_with("/firefox") && v.last().unwrap() == name {
                    let uptime = get_uptime(p);
                    if uptime > WARMUP_TIME {
                        State::Warmed
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

fn get_uptime(p: process::Process) -> u64 {
    let nowsec = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    println!(
        "{} - {} - {}",
        nowsec.as_secs(),
        p.stat.starttime,
        procfs::boot_time_secs().unwrap()
    );
    nowsec.as_secs()
        - p.stat.starttime / unsafe { sysconf(_SC_CLK_TCK) } as u64
        - procfs::boot_time_secs().unwrap()
}

fn get_pid(pidlink: &Path) -> i32 {
    let link = fs::read_link(pidlink);
    match link {
        Ok(p) => p
            .to_str()
            .unwrap()
            .to_string()
            .split('+')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .parse::<i32>()
            .unwrap(),
        _ => 0,
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

pub fn firefoxes() -> HashMap<String, Firefox> {
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
        firefoxes.insert(p.name.clone(), Firefox::new(p, mode));
    }
    firefoxes
}
