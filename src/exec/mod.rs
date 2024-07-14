
use std::process::{Command, Stdio};
use std::str;


use crate::envs;
use crate::common::print_debug;

pub fn tty(command: &str) {
    print_debug(&command);
    // Run command interactively with a TTY
    let child = Command::new("bash")
        .arg("-c")
        // .arg(envs::eval(command))
        .arg(command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn();

    // Check if the command execution was successful
    match child {
        Ok(mut child) => {
            // Wait for the child process to finish (you can remove this if you want to keep it running)
            let _status = child.wait();
        }
        Err(e) => eprintln!("Error spawning command: {}", e),
    }

}



pub fn cmd(command: &str, evaluate: Option<bool>) -> Result<String, String> {
    // Spawn a subprocess to execute the command
    let _command = if evaluate.unwrap_or(false) {
        envs::eval(command)
    } else {
        command.to_string()
    };

    let output = Command::new("bash")
        .arg("-c")
        .arg(_command)
        .output();

    // Check if the subprocess was executed successfully
    match output {
        Ok(result) => {
            if result.status.success() {
                // Print the output of the command
                if let Ok(output_string) = String::from_utf8(result.stdout) {
                    Ok(output_string.trim().to_string())
                } else {
                    Err("Error converting output to string".to_string())
                }
            } else {
                // Print the error message if the command failed
                Err(format!("Command failed with error: {:?}", result.status))
            }
        }
        Err(err) => {
            // Print the error message if there was an issue spawning the subprocess
            Err(format!("Error spawning subprocess: {}", err))
        }
    }
}

pub fn stdout(command: &str) -> Result<String, String> {
    return cmd(command, None);
}


pub fn zk0(command: &str) -> Result<String, String> {
    return cmd(&format!("docker exec -it zk0 bash -c '{}'", command), None);
}
