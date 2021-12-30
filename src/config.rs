use crate::firefox;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

const CONF_FILENAME: &str = "config.json";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub Name: String,
    pub Mode: firefox::Mode,
}

// fn configs() -> std::collections::HashMap<String, Config> {
pub fn configs() -> Vec<Config> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("firefox-launcher").unwrap();
    let path = xdg_dirs.find_config_file(CONF_FILENAME);
    match path {
        Some(p) => {
            let file = File::open(p).unwrap();
            let reader = BufReader::new(file);
            // Read the JSON contents of the file as an instance of `User`.
            serde_json::from_reader(reader).unwrap()
        }
        _ => vec![],
    }
}
