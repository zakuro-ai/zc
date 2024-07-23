#[allow(warnings)]
use crate::exec;
use crate::common;
use crate::envs;
use crate::network;
use std::{env};

use std::error::Error as StdError;
use serde_json::Value;

use colored::Colorize;

pub fn rmi(){
    kill();
    let _ = exec::tty("docker rmi -f $(docker images --filter \"label=maintainer=dev@zakuro.ai\" -a -q)");
}

pub fn push(image: Option<&str>){
    if let (Some(image_value),Ok(dist_str)) = (
        image,
        common::dist()
    ) {

    let _ = exec::cmd(&format!(
        "docker tag zakuroai/{}:latest zakuroai/{}:{} && docker push zakuroai/{}:{} && docker rmi zakuroai/{}:{}", 
        image_value, 
        image_value, 
        dist_str,
        image_value, 
        dist_str,
        image_value, 
        dist_str
    ), 
    None);
    }
}

pub fn images() {
    let _ = exec::tty(
        &format!(
            "docker images --filter \"label=maintainer=dev@zakuro.ai\" -a",
        )
    );
}

pub fn pull() {
    if let Ok(dist_str) = common::dist(){
        for image in vec!["network", "storage", "compute"] {
            exec::tty(&format!("docker pull zakuroai/{}:{}", image, &dist_str));
            exec::tty(&format!("docker tag zakuroai/{}:{} zakuroai/{}:latest",
                                image, &dist_str, image));
            exec::tty(&format!("docker rmi zakuroai/{}:{}", image, &dist_str));
        }
    }
}

pub fn kill() {
    if let Ok(ids) = exec::cmd(
        &format!(
            "ids=$(docker ps --filter 'name=zk0*' -a -q);echo $ids",
        ), None
    ){
        if ids.len() > 1 {
            for id in ids.split(" ") {
                let _ = exec::tty(&format!("docker stop {}", id,));
                let _ = exec::tty(&format!("docker rm {}", id));
            }
        }
    }
}

pub fn remove_container() {
    let _ = exec::cmd(&format!("docker stop zk0 && docker rm zk0"), None);
}

pub fn restart() {
    envs::update();
    if let (Ok(zakuro_context),) = (
        env::var("ZAKURO_CONTEXT"),
    ) {
        let _ = exec::tty(
            &format!(
                "cd {} && docker compose down && docker compose up -d",
                zakuro_context
            ),
        );
    }
        
}

pub fn workers() -> Result<Option<String>, Box<dyn StdError>> {
    match network::wg0ip(Some(true)) {
        Ok(Some(_addr_str)) => {
            let master_url = "http://10.13.13.2:8080/json";
            let response: Result<ureq::Response, ureq::Error> = ureq::get(master_url).call();
        
            match response {
                Ok(response) => {
                    let cluster_info: Value = serde_json::from_reader(response.into_reader())?;
                    if let Some(workers) = cluster_info.get("workers") {
                        for worker in workers.as_array().unwrap() {
                            // println!("Worker ID: {}", worker["id"]);
                            println!("Host: {}", worker["host"]);
                            // println!("Port: {}", worker["port"]);
                            println!("State: {}", worker["state"]);
                            println!("Memory: {}", worker["memory"]);
                            println!("Core: {}", worker["cores"]);
                            println!("--------------------------");
                        }
                        return Ok(None);
                    } else {
                        println!("No workers found");
                    }
                }
                Err(err) => {
                    eprintln!("Failed to fetch data: {}", err);
                }
            }
        }
        Ok(None) => {
            match  exec::zk0("zc workers"){
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(err) => {
            // Handle the error case
            eprintln!("Failed to fetch data: {}", err);
        }
    }
    Ok(None)
}

pub fn add_worker() {
    let _ = exec::cmd(&format!("docker exec -d zk0 bash -c '/spark'"), Some(true));
}

pub fn connect() {
    let _ = exec::tty("docker exec -it -e TERM=xterm-256color zk0 /bin/bash");
}

pub fn nodes() {
    if let Ok(s) = exec::cmd(&format!("{} | {}",
    &format!("nmap -sP 10.13.13.0/24 -oG -"), "awk '/Up$/{print $2}'"), None){
        println!(
            "{}\t\t\t{}",
            "[Nodes]".bold().blue(),
            s.replace("\n", "|")
        );
    }
}

pub fn server_list() {
    let _ = exec::zk0("jupyter-server list");
}


pub fn ps() {
    let _ = exec::tty(
        &format!("docker ps --filter \"label=maintainer=dev@zakuro.ai\" -a")
    );
}
