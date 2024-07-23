use std::error::Error as StdError;
use std::process::{Command, Stdio};
use std::str;

use std::any::Any;
use std::fmt::Debug;

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


// pub fn zk0(command: &str){
//     let zk0_command = &format!("docker exec -it zk0 bash -c '{}'", command);
//     let _ = tty(&format!("{}",zk0_command));
//     // println!("{}", zk0_command);
//     // return cmd(zk0_command, None);
// }
pub fn zk0(last_arg: &str) -> Result<Option<String>, Box<dyn StdError>> {
    let args = ["exec", "-i", "zk0", "bash", "-c", last_arg];
    let output = Command::new("docker")
        .args(args)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(Some(stdout))
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        eprintln!("Error:\n{}", stderr);
        Ok(None)
    }

}


pub fn execute_fn<T: Debug + Any>(response: Result<T, Box<dyn StdError>>) {
    match response {
        Ok(result) => {
            if let Some(option) = (&result as &dyn Any).downcast_ref::<Option<String>>() {
                if let Some(value) = option {
                    println!("{}", value);
                }
            } else {
                println!("Operation successful: {:?}", result);
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch data: {}", err);
        }
    }
}