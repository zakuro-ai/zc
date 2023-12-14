use crate::exec;
use crate::common;
use crate::envs;
use std::{env, fs};


use colored::Colorize;

pub fn rmi(){
    kill();
    let _ = exec::cmd("docker rmi -f $(docker images --filter \"label=maintainer=dev@zakuro.ai\" -a -q)", None);
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
    let _ = exec::cmd(
        &format!(
            "docker images --filter \"label=maintainer=dev@zakuro.ai\" -a",
        ),
        Some(true),
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
        ),
        None,
    ){
        if ids.len() > 1 {
            for id in ids.split(" ") {
                let _ = exec::cmd(&format!("docker stop {}", id,), Some(true));
                let _ = exec::cmd(&format!("docker rm {}", id), Some(true));
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
    let _ = exec::cmd(
        &format!("docker ps --filter --filter \"label=maintainer=dev@zakuro.ai\" -a"),
        Some(true),
    );
}
