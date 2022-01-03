use std::process::Command;

pub fn launch_firefox(name: &str) {
    let _ = Command::new("setsid")
        .arg("-f")
        .arg("firefox")
        .arg("--no-remote")
        .arg("-P")
        .arg(name)
        .spawn();
}
