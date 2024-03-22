use crate::common;
use nix::fcntl::{Flock, FlockArg};
use std::fs::File;

pub struct Lockfile {
    #[allow(dead_code)]
    file: Option<Flock<File>>,
    pub is_single: bool,
}

impl Lockfile {
    pub fn new() -> Self {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(common::XDG_PREFIX).unwrap();
        let path = xdg_dirs.place_runtime_file("lock");
        match path {
            Ok(p) => {
                let file = if p.exists() {
                    File::open(p)
                } else {
                    File::create(p)
                };
                match file {
                    Ok(f) => {
                        let result = Flock::lock(f, FlockArg::LockExclusiveNonblock);
                        match result {
                            Ok(l) => Self {
                                file: Some(l),
                                is_single: true,
                            },
                            Err(_) => Self {
                                file: None,
                                is_single: false,
                            },
                        }
                    }
                    _ => Self {
                        file: None,
                        is_single: false,
                    },
                }
            }
            _ => Self {
                file: None,
                is_single: false,
            },
        }
    }
}
