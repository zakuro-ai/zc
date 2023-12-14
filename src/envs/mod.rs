use crate::common;
use crate::exec;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::env;

pub static CONFIG_FILE: &str = "/opt/zakuro/etc/profile.d/zk0.sh";
pub static CONFIG_DIR: &str = "/opt/zakuro/etc/profile.d";

pub fn eval(input: &str) -> String {
    let result = env::vars().fold(input.to_string(), |acc, (key, value)| {
        acc.replace(&format!("${}", key), &value)
    });
    if result.contains('$') {
        return eval(&result)
    }
    result
}

pub fn update(){

    if !Path::new(CONFIG_FILE).exists() {
        for command in vec![
            &format!("sudo mkdir -p {}", CONFIG_DIR),
            &format!("sudo chown -R $USER {}", CONFIG_DIR),
            &format!("wget -q 'http://get.zakuro.ai/env' -O {}", CONFIG_FILE),
        ] {
            let _ = exec::tty(command);
        }
    }


    // Read the content of the shell script
    let script_content = match std::fs::read_to_string(CONFIG_FILE) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading script: {}", e);
            return;
        }
    };

    // Parse the script content to extract variable assignments
    for line in script_content.lines() {
        // Check if the line is a variable assignment
        if let Some((key, value)) = line.split_once('=') {
            // Trim whitespace and remove quotes if present
            let key = key.trim();
            let value = value.trim_matches('"').trim();

            // Set the environment variable
            env::set_var(key, value);
        }
    }
}

// pub fn vars() -> Result<HashMap<String, String>, std::io::Error> {
//     let mut variables = HashMap::new();

//     if !Path::new(CONFIG_FILE).exists() {
//         for command in vec![
//             &format!("sudo mkdir -p {}", CONFIG_DIR),
//             &format!("sudo chown -R $UID {}", CONFIG_DIR),
//             &format!("wget -q 'http://get.zakuro.ai/env' -O {}", CONFIG_FILE),
//         ] {
//             common::exec(command, Some(true));
//         }
//     }

//     let command = &format!("source {} && env", CONFIG_FILE);
//     let lines = common::exec(command, Some(false));

//     for line in lines.split("\n") {
//         if let Some((key, value)) = parse_exported_variable(line) {
//             variables.insert(key.to_string(), value.to_string());
//         }
//     }
//     Ok(variables)
// }

fn parse_exported_variable(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '=');
    match (parts.next(), parts.next()) {
        (Some(key), Some(value)) => Some((key.trim(), value.trim())),
        _ => None,
    }
}
