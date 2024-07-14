
use crate::exec;

use std::path::Path;

use std::str;
use std::env;

pub static CONFIG_FILE: &str = "/opt/zakuro/etc/profile.d/zk0.sh";
pub static CONFIG_DIR: &str = "/opt/zakuro/etc/profile.d";
use std::collections::HashSet;
use std::collections::HashMap;
pub fn eval(s1: &str) -> String{
    evaluate_expr(s1)
}
use crate::common::print_debug;

fn skip_first_char_if_slash(output: &str) -> Result<String, String> {
    // Check if the string is long enough to slice
    if !output.is_empty() && output.starts_with('/') {
        let result: String = output[1..].to_string();
        return Ok(result);
    } else if !output.is_empty() {
        return Ok(output.to_string());
    }
    Err(String::from("String is empty!"))
}


pub fn evaluate_expr(s1: &str) -> String{
    let mut output: String = String::from("");
    if !s1.starts_with("'"){
        // Extract path x:x
        for pathx in s1.split(':'){
            // For each x split by /
            let splits: Vec<&str> = pathx.split("/").collect();
            let condition:bool = splits.len()>1;
            for varpath in splits{
                if let Some('$') = varpath.chars().next() {
                    let _varpath = &varpath[1..];
                    if let Ok(s) = env::var(_varpath)
                    {
                        // println!("found: {:?}",&s);
                        env::set_var(_varpath, &s);
                        output.push_str(&format!("{}", &s));
                    }
                    else{
                        // println!("(var not found) ${:?}", _varpath);
                        output.push_str(&format!("{}", &varpath));
                    }
                }
                else{
                    if condition{
                        output.push_str(&format!("/{}", &varpath));
                    }
                    else{
                        output.push_str(&format!("{}", &varpath));
                    }
                    // println!("{}", &varpath);    
                }
            }
            output.push_str(":");
        }
        if output.ends_with(':') {
            output = (&output[..output.len() - 1]).to_string();
        }
        // match skip_first_char_if_slash(output.clone()){
        //     Ok(res) => {
        //         output = res;
        //     }
        //     Err(e) => {
        //         eprintln!("{}", e);
        //     }
        // }   
        
        // output = skip_first_char_if_slash(&output).unwrap_or_else(|e| {
        //     eprintln!("{}", e);
        //     output.clone()  // Return the original output in case of an error
        // });

        // if (&output[1..]).to_string().starts_with('/') {
        //     output = (&output[1..]).to_string();
        // }
        output = output.replace("//", "/");
        // println!("{:?}",output);
        let string_list: Vec<&str> = output.split(":").collect();
        // Create a HashSet to store unique values
        let unique_values: HashSet<_> = string_list.into_iter().collect();
        // Convert the HashSet back to a Vec if needed
        let unique_list: Vec<_> = unique_values.into_iter().collect();
        // println!("{:?}", unique_list);
        output = unique_list.join(":");   
    }
    else{
        output = String::from(s1);
    }

    output

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
        if let Some((key, _value)) = line.split_once('=') {
            if env::var(key).is_err() {
                let value = evaluate_expr(_value);
                // let value = _value;
                // Only print in debug mode
                print_debug(&format!("{}={}", key, value));
                env::set_var(key, value);
            }
        }
    }
}
