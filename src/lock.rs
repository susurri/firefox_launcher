use nix::fcntl::{flock, FlockArg};
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub struct Lockfile {
    file: Option<File>,
    pub is_single: bool,
}

impl Lockfile {
    pub fn new() -> Self {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("firefox-launcher").unwrap();
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
                        let fd = f.as_raw_fd();
                        let result = flock(fd, FlockArg::LockExclusiveNonblock);
                        Self {
                            file: Some(f),
                            is_single: matches!(result, Ok(_)),
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