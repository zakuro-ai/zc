use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::str;

pub fn command(command: &str) {

    // Run command interactively with a TTY
    let mut child = Command::new("/bin/bash")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn();

    // Check if the command execution was successful
    match child {
        Ok(mut child) => {
            // Wait for the child process to finish (you can remove this if you want to keep it running)
            let status = child.wait();
        }
        Err(e) => eprintln!("Error spawning command: {}", e),
    }

}

pub fn exec(command: &str, print_command: Option<bool>) -> String {
    let process = match Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn {}: {}", command, why),
        Ok(process) => process,
    };

    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(why) => panic!("couldn't read wc stdout: {}", why),
        Ok(_) => {
            if print_command.unwrap_or(false) {
                print!("{}", &s);
            }
        }
    }

    return s;
}

pub fn dist() -> String {
    return exec(&format!("echo $(uname -m)"), Some(false)).replace("\n", "");
}
