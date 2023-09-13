use crate::common;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

pub static CONFIG_FILE: &str = "/opt/zakuro/etc/profile.d/zk0.sh";
pub static CONFIG_DIR: &str = "/opt/zakuro/etc/profile.d";

pub fn docker() -> String {
    let docker;
    if Path::new("/usr/bin/docker").exists() {
        docker = "/usr/bin/docker";
    } else {
        docker = "/usr/local/bin/docker";
    }
    return String::from(docker);
}

pub fn nmap() -> String {
    if Path::new("/opt/homebrew/bin/nmap").exists() {
        return String::from("/opt/homebrew/bin/nmap");
    } else {
        return String::from("/usr/bin/nmap");
    }
}

pub fn vars() -> Result<HashMap<String, String>, std::io::Error> {
    let mut variables = HashMap::new();

    if !Path::new(CONFIG_FILE).exists() {
        let command = &format!("sudo mkdir -p {}", CONFIG_DIR);
        common::exec(command, Some(true));
        let command = &format!("wget -q 'http://get.zakuro.ai/env' -O {}", CONFIG_FILE);
        common::exec(command, Some(true));
    }

    let command = &format!("source {} && env", CONFIG_FILE);
    let lines = common::exec(command, Some(false));

    for line in lines.split("\n") {
        if let Some((key, value)) = parse_exported_variable(line) {
            variables.insert(key.to_string(), value.to_string());
        }
    }
    Ok(variables)
}

fn parse_exported_variable(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '=');
    match (parts.next(), parts.next()) {
        (Some(key), Some(value)) => Some((key.trim(), value.trim())),
        _ => None,
    }
}
