use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

pub static CONFIG_FILE: &str = "/opt/zakuro/etc/profile.d/zk0.sh";

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
    let mut command = Command::new("bash");
    command
        .arg("-c")
        .arg(format!("source {} && env", CONFIG_FILE));

    // Ensure we capture the command's stdout
    command.stdout(Stdio::piped());

    let output = command.spawn()?.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut variables = HashMap::new();

    for line in stdout.lines() {
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
