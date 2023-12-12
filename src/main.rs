extern crate serde;
extern crate toml;

use colored::Colorize;
use html5ever::rcdom::*;
use soup::prelude::*;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::str;
use std::{collections::HashMap, fs::File};
use std::{env, fs};
mod common;
mod envs;
use common::{dist, exec};
use envs::docker;
use std::process::{Command, Stdio};

// The file `built.rs` was placed there by cargo and `build.rs`
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn connect() {
    common::command("docker exec -it -e TERM=xterm-256color zk0 /bin/bash")
}

pub fn logs(alive: bool) {
    fn clean(c: String) -> String {
        let splits: Vec<&str> = c.split_whitespace().collect();
        return splits.join(" ");
    }
    fn get_tag_soup(soup: Soup, c: &str) -> Rc<Node> {
        return soup
            .tag(c)
            .find()
            .expect(&format!("Couldn't find tag {}", c));
    }
    fn get_tag(node: Rc<Node>, c: &str) -> Rc<Node> {
        return node
            .tag(c)
            .find()
            .expect(&format!("Couldn't find tag {}", c));
    }
    fn get_tag_text(node: Rc<Node>, c: &str) -> String {
        return get_tag(node, c).text();
    }
    fn get_tag_rec(node: Rc<Node>, cs: &mut Vec<&str>) -> Rc<Node> {
        let c = cs.remove(0);
        if cs.len() == 0 {
            return get_tag(node, c);
        } else {
            return get_tag_rec(get_tag(node, c), cs);
        }
    }
    fn get_all(node: Rc<Node>, c: &str) -> Vec<Rc<Node>> {
        return node.tag(c).find_all().collect::<Vec<_>>();
    }
    let html = exec("curl -s http://spark-master.zakuro.ai:8080", Some(false));
    let soup = Soup::new(&html);
    let body = &get_tag_soup(soup, "body");
    // let h3 = &get_tag_rec(body.clone(), &mut vec!["div", "div", "div", "h3"]);
    let h3 = &get_tag_rec(body.clone(), &mut vec!["div", "div", "div", "h3"]);
    let version = clean(get_tag_rec(h3.clone(), &mut vec!["a", "span"]).text());
    let h3_text = clean(h3.text());
    let splits: Vec<&str> = h3_text.split("at ").collect();
    let url = splits[1];
    let tables = get_all(body.clone(), "tbody");

    let mut dworkers = HashMap::new();
    let trs = get_all(tables[0].clone(), "tr");
    let mut alive_workers = 0;
    let mut cores_used = 0;
    let mut cores_total = 0;
    let mut mem_used = 0.0;
    let mut mem_total = 0.0;
    let mut app_running = 0;
    let mut app_completed = 0;

    for worker in trs {
        // let wid = get_tag_text(worker.clone(), "a");
        let wid = worker.text();
        let worker_info = get_all(worker.clone(), "td");
        dworkers.insert(
            clean(wid),
            HashMap::from([
                ("status", clean(worker_info[2].text())),
                ("cores", clean(worker_info[3].text())),
                ("mem", clean(worker_info[4].text())),
                ("resources", clean(worker_info[5].text())),
            ]),
        );
        if clean(worker_info[2].text()) == "ALIVE" {
            alive_workers += 1;
            // Cores
            let cores = clean(worker_info[3].text());
            let mut splits: Vec<&str> = cores.split(" ").collect();
            let cores_total_worker: i32 = splits[0].parse().unwrap();
            cores_total += cores_total_worker;
            splits = splits[1].split("(").collect();
            splits = splits[1].split(" ").collect();
            let cores_used_worker: i32 = splits[0].parse().unwrap();
            cores_used += cores_used_worker;
            // Mem
            let mut mem = clean(worker_info[4].text());
            mem = mem.replace("(", "").replace(")", "");
            splits = mem.split(" ").collect();

            let mut mem_total_worker: f64 = splits[0].parse().unwrap();
            let mem_total_worker_dim = splits[1];
            let mut mem_used_worker: f64 = splits[2].parse().unwrap();
            let mem_used_worker_dim = splits[3];

            if mem_total_worker_dim == "GiB" {
                mem_total_worker *= 1000.0;
            }
            if mem_used_worker_dim == "GiB" {
                mem_used_worker *= 1000.0;
            }

            mem_total += mem_total_worker;
            mem_used += mem_used_worker;
        }
    }
    mem_total /= 1000.0;
    mem_used /= 1000.0;

    let mut dapp = HashMap::new();
    let trs = get_all(tables[1].clone(), "tr");
    for app in trs {
        let aid = get_tag_text(app.clone(), "a");
        let app_info = get_all(app.clone(), "td");
        dapp.insert(
            clean(aid),
            HashMap::from([
                ("cores", clean(app_info[2].text())),
                ("mem", clean(app_info[3].text())),
                ("resources", clean(app_info[4].text())),
                ("submitted", clean(app_info[5].text())),
                ("user", clean(app_info[6].text())),
                ("status", clean(app_info[7].text())),
                ("duration", clean(app_info[8].text())),
            ]),
        );
        if clean(app_info[7].text()) == "RUNNING" {
            app_running += 1;
        }
    }

    let trs = get_all(tables[2].clone(), "tr");
    for app in trs {
        let aid = get_tag_text(app.clone(), "a");
        let app_info = get_all(app.clone(), "td");
        if clean(app_info[7].text()) == "FINISHED" {
            app_completed += 1;
        }
    }
    // println!("{:?}", dworkers);
    // println!("{:?}", dapp);
    println!("{}\t\t{}", "[VERSION]".blue().bold(), version);
    println!("{}\t\t\t{}", "[URL]".blue().bold(), url);
    println!("{}\t\t{}", "[Alive Workers]".blue().bold(), alive_workers);
    println!(
        "{}\t\t{}/{} used",
        "[Cores in use]".blue().bold(),
        cores_used,
        cores_total
    );
    println!(
        "{}\t\t{}/{} GiB used",
        "[Memory in use]".blue().bold(),
        mem_used,
        mem_total
    );
    println!(
        "{}\t\t{} Running, {} Completed",
        "[Applications]".bold().blue(),
        app_running,
        app_completed
    );

    println!("{}", "====WORKERS====".bold().yellow());
    for (k, v) in &dworkers {
        let mut condition = true;
        if alive {
            if v["status"] != "ALIVE" {
                condition = false;
            }
        }

        if condition {
            println!("{}", k.purple());
        }
    }
    println!("{}", "====APPS====".bold().yellow());
    for (k, v) in &dapp {
        let s = format!(
            "{}@{} allocated {} cores, {} memory",
            v["user"], k, v["cores"], v["mem"]
        );
        println!("{}", s.purple());
    }
}

fn zk0(command: &str) -> String {
    return exec(
        &format!("{} exec -it zk0 bash -c '{}'", docker(), command),
        Some(true),
    );
}

fn wg0ip() {
    for iface in ifaces::Interface::get_all().unwrap().into_iter() {
        if String::from(format!("{:?}", iface.kind)) == "Ipv4" {
            let addr = String::from(format!("{}", iface.addr.unwrap()));
            let v: Vec<&str> = addr.split(":").collect();
            if v[0].starts_with("10.13.13") {
                println!("{}", v[0]);
            }
        }
    }
}

fn add_worker() {
    let command = format!("{} exec -d zk0 bash -c '/spark'", docker());
    common::exec(&command, Some(true));
}

fn nmap() {
    let c0 = &format!("{} -sP 10.13.13.0/24 -oG -", envs::nmap());
    let c1 = "awk '/Up$/{print $2}'";
    common::exec(&format!("{} | {}", c0, c1), Some(true));
}

fn nmap_inf() {
    while true {
        nmap()
    }
}
// fn zakuro_cli(command: &str, from_docker: bool) -> String {
//     if from_docker {
//         return zk0(&format!("zc {} ", command));
//     } else {
//         return common::exec(&format!("zc {} ", command), Some(true));
//     }
// }

fn nodes() {
    let c0 = &format!("{} -sP 10.13.13.0/24 -oG -", envs::nmap());
    let c1 = "awk '/Up$/{print $2}'";
    println!(
        "{}\t\t\t{}",
        "[Nodes]".bold().blue(),
        common::exec(&format!("{} | {}", c0, c1), Some(false)).replace("\n", "|")
    );
}

fn remove_container() {
    let command = format!("{} stop zk0 && {} rm zk0", docker(), docker());
    common::exec(&command, Some(false));
}

fn server_list() {
    zk0("jupyter-server list ");
}
fn restart() {
    match envs::vars() {
        Ok(vars) => {
            common::exec(
                &format!(
                    ". {}; cd {} && {} compose down && {} compose up -d",
                    envs::CONFIG_FILE,
                    vars.get("ZAKURO_CONTEXT").unwrap(),
                    docker(),
                    docker(),
                ),
                Some(false),
            );
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

fn context(path: Option<&str>) {
    match envs::vars() {
        Ok(vars) => {
            // Specify the file path
            let zakuro_env: String = fs::read_to_string(envs::CONFIG_FILE).unwrap();
            if let Some(path_str) = path {
                let output_line = format!("export ZAKURO_CONTEXT={}", path_str);
                let path = Path::new(path_str);
                if path.exists() {
                    // let path_env = &format!("{}/.zakuro/env", env::var("HOME").unwrap());
                    let mut lines = Vec::new();
                    for line in zakuro_env.split("\n") {
                        if !line.contains("export ZAKURO_CONTEXT") {
                            lines.push(line);
                        }
                    }
                    lines.push(&output_line);
                    // Concatenate the strings into a single string
                    let concatenated = lines.join("\n");

                    let mut file = File::create(envs::CONFIG_FILE).unwrap();

                    // Write the concatenated string to the file
                    file.write_all(concatenated.as_bytes()).unwrap();
                }
            } else {
                println!("{:?}", vars);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
fn pull() {
    let dist_str = dist();
    for image in vec!["network", "storage", "compute"] {
        common::command(
            &format!("docker pull zakuroai/{}:{} && \
            docker tag zakuroai/{}:{} zakuroai/{}:latest && \
            docker rmi zakuroai/{}:{}", 
            image,
            dist_str,
            image,
            dist_str,
            image,
            image,
            dist_str
        ),
        );
    }
}

fn download_conf() {
    match envs::vars() {
        Ok(vars) => {
            let zakuro_root = vars.get("ZAKURO_HOME").unwrap();
            // println!("{}", zakuro_root);
            //Create dirs
            for image in vec![
                "config", "network", "storage", "compute", "node", "hub", "lib", "logs", "bin",
            ] {
                let command = &format!("mkdir -p {}/{}", zakuro_root, image);
                // println!("{}", command);
                common::exec(command, Some(true));
            }
            //Download confs
            for image in vec!["network", "storage", "compute", "node", "hub"] {
                common::exec(
                    &format!(
                        "wget -q 'http://get.zakuro.ai/zk0?config={}' -O {}/{}/{}-zakuro.yml",
                        image, zakuro_root, image, image
                    ),
                    Some(true),
                );
                common::exec(
                    &format!(
                        "wget -q 'http://get.zakuro.ai/zk0?config={}_env' -O {}/{}/.env",
                        image, zakuro_root, image
                    ),
                    Some(true),
                );
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

fn download_auth() {
    match envs::vars() {
        Ok(vars) => {
            let zakuro_root = vars.get("ZAKURO_HOME").unwrap();
            let zakuro_auth = vars.get("ZAKURO_AUTH").unwrap();
            let command = &format!(
                "curl -s --location --request GET 'https://get.zakuro.ai/profile' \
            --header 'Content-Type: application/json' \
            --data '{{\"pkey\": \"{}\"}}' > {}/config/wg0.conf",
                zakuro_auth, zakuro_root
            );

            let result = common::exec(command, Some(false));
            println!("{}", result);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
fn kill() {
    let ids = common::exec(
        &format!(
            "ids=$({} ps --filter 'name=zk0*' -a -q);echo $ids",
            docker()
        ),
        Some(false),
    );
    if ids.len() > 1 {
        for id in ids.split(" ") {
            common::exec(&format!("{} stop {}", docker(), id,), Some(true));
            common::exec(&format!("{} rm {}", docker(), id), Some(true));
        }
    }
}

fn ps() {
    common::exec(
        &format!("{} ps --filter --filter \"label=maintainer=dev@zakuro.ai\" -a", docker()),
        Some(true),
    );
}

fn update() {
    common::command("curl https://get.zakuro.ai/zc | sh");
}


fn images() {
    common::exec(
        &format!(
            "{} images --filter \"label=maintainer=dev@zakuro.ai\" -a",
            docker()
        ),
        Some(true),
    );
}

fn launch() {
    pull();
    kill();
    restart();
}
fn setup() {
    download_auth();
    download_conf();
    launch();
}

fn version()
{
    if let (Some(short_hash),) = (
        built_info::GIT_COMMIT_HASH_SHORT,
    ) {

        let built_time = built::util::strptime(built_info::BUILT_TIME_UTC);
        println!("zc version {}, build {} on {}", built_info::PKG_VERSION, short_hash, built_time.with_timezone(&built::chrono::offset::Local),
);
}
}
fn rmi(){
    kill();
    common::command("docker rmi -f $(docker images --filter \"label=maintainer=dev@zakuro.ai\" -a -q)");
}

fn push(image: Option<&str>){
    if let (Some(image_value),) = (
        image,
    ) {

    common::command(&format!(
        "docker tag zakuroai/{}:latest zakuroai/{}:{} && docker push zakuroai/{}:{} && docker rmi zakuroai/{}:{}", 
        image_value, 
        image_value, 
        dist(),
        image_value, 
        dist(),
        image_value, 
        dist()
    ));
    }
}

fn build(image: Option<Vec<String>>){
    if let (Some(image_value),) = (
        image,
    ) {
        let truncated_vector: Vec<String> = image_value.into_iter().skip(2).collect();
        let _d = dist();
        let d;
        if _d=="arm64"{
            d = "aarch64"
        }
        else if _d=="x86_64"{
            d ="amd64"
        }
        else{
            d=&_d;
        }        
        let s: String = truncated_vector.join(" ");
        common::command(&format!("BUILDARCH={} BUILDARCHI={} docker compose build {}", d, dist(), s));
    }

}
fn help() {

    // io::stdout().write_all(s.as_bytes()).unwrap();
    println!("\nUsage: zc [OPTIONS] COMMAND
    \nA self-sufficient runtime for zakuro
Options:
      -d, --docker    Execute the commands from zk0.
      -v, --version   Get the version of the current command line.
      -h, --help      Print this help.
    \nCommands:
      connect         Enter zk0 in interactive mode.
      update          Update the command line.
      pull            Pull updated images.
      images          List zakuro images built on the machine.
      ps              List current running zakuro containers.
      context <path>  Set new zakuro context.
      kill            Remove current running zakuro containers.
      restart         Restart the containers with updated images.
      wg0ip           Get the IP in the cluster.
      rmi             Remove zakuro images.
    
\nTo get more help with docker, check out our guides at https://docs.zakuro.ai/go/guides/");
}


fn main() {

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            help();
        }
        2 => {
            let arg0 = &args[1];
            match &arg0[..] {
                "-h" => help(),
                "--help" => help(),
                "ps" => ps(),
                "images" => images(),
                "nmap" => nmap(),
                "launch" => launch(),
                "download_conf" => download_conf(),
                "download_auth" => download_auth(),
                "setup" => setup(),
                "pull" => pull(),
                "nmap_inf" => nmap_inf(),
                "wg0ip" => wg0ip(),
                "logs" => logs(true),
                "nodes" => nodes(),
                "restart" => restart(),
                "servers" => server_list(),
                "add_worker" => add_worker(),
                "kill" => kill(),
                "connect" => connect(),
                "update" => update(),
                "rmi" => rmi(),
                "--version" => version(),
                "-v" => version(),
                "vars" => {
                    context(None);
                }
                _ => {
                    help();
                }
            }
        }
        3 => {
            let arg0 = &args[1];
            let arg1 = &args[2];
            match &arg0[..] {
                "--docker" => match &arg1[..] {
                    "rm" => remove_container(),
                    _ => {
                        zk0(&arg1[..]);
                    }
                },
                "-d" => match &arg1[..] {
                    "rm" => remove_container(),
                    _ => {
                        zk0(&arg1[..]);
                    }
                },
                "push" => {
                    push(Some(&arg1[..]));
                },
                "context" => {
                    context(Some(&arg1[..]));
                },
                "build" => {
                    build(Some(args));
                }
                _ => help(),
            }
        }
        _ => {
            let arg0 = &args[1];
            let arg1 = &args[2];
            match &arg0[..] {
            "build" => {
                build(Some(args));
            }
            _ => {
                help();
            }
        }
    }
}
}
