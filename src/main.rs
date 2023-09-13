extern crate serde;
extern crate toml;

use colored::Colorize;
use html5ever::rcdom::*;
use soup::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::str;
mod common;
mod envs;
use common::{dist, exec};
use envs::docker;

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
        &format!("{} exec -i zk0 bash -c '{}'", docker(), command),
        Some(true),
    );
}

fn wg0ip() {
    for iface in ifaces::Interface::get_all().unwrap().into_iter() {
        if (String::from(format!("{:?}", iface.kind)) == "Ipv4") {
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

fn zakuro_cli(command: &str, from_docker: bool) -> String {
    if from_docker {
        return zk0(&format!("zc {} ", command));
    } else {
        return common::exec(&format!("zc {} ", command), Some(true));
    }
}

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

fn restart() {
    common::exec(&format!("{} restart zk0", docker()), Some(false));
}

fn reboot() {
    common::exec(
        &format!(
            "cd /home/jcadic/.zakuro/node; {} compose down;{} compose up -d;",
            docker(),
            docker()
        ),
        Some(true),
    );
}

fn server_list() {
    zk0("jupyter-server list ");
}
fn up() {
    match envs::vars() {
        Ok(vars) => {
            common::exec(
                &format!(
                    "cd {} && {} compose down; {} compose up -d",
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

    // let cntx = context(None);
    // common::exec(
    //     &format!(
    //         "{} stop zk0 && {} rm zk0 && {} compose -f {} up zk0 -d",
    //         docker(),
    //         docker(),
    //         docker(),
    //         cntx.fs.context
    //     ),
    //     Some(false),
    // );
    // server_list();
}

fn help() {
    let s = "\nUsage:  zc [OPTIONS] COMMAND
    \nA self-sufficient runtime for zakuro
Options:
      --docker        Execute the commands from zk0
    \nCommands:
      wg0ip           Get the IP in the cluster.
      nmap            Retrieve the list of nodes connected.
      logs            Fetch the logs of master node
      restart         Restart the zakuro service
      servers         Return the list of jupyter server runnning 
      add_worker      Add a worker to the network
      rm              Remove zk0
      context         Change the path to the context
\nTo get more help with docker, check out our guides at https://docs.zakuro.ai/go/guides/";
    // io::stdout().write_all(s.as_bytes()).unwrap();
    println!("{}", s)
}

fn context(path: Option<&str>) {
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
        println!("{}", zakuro_env);
    }
}
fn pull() {
    let dist_str = dist();
    for image in vec!["network", "storage", "compute"] {
        println!("Updating zakuroai/{} ...", image);
        common::exec(
            &format!("{} pull zakuroai/{}:{}", docker(), image, dist_str),
            Some(false),
        );
        common::exec(
            &format!(
                "{} tag zakuroai/{}:{} zakuroai/{}:latest",
                docker(),
                image,
                dist_str,
                image
            ),
            Some(false),
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
                "network", "storage", "compute", "node", "hub", "lib", "logs", "bin",
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
fn launch() {
    pull();
    kill();
    up();
}
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let arg0 = &args[1];
            match &arg0[..] {
                "help" => help(),
                "nmap" => nmap(),
                "up" => up(),
                "launch" => launch(),
                "download_conf" => download_conf(),
                "pull" => pull(),
                "nmap_inf" => nmap_inf(),
                "wg0ip" => wg0ip(),
                "logs" => logs(true),
                "nodes" => nodes(),
                "restart" => restart(),
                "reboot" => reboot(),
                "servers" => server_list(),
                "add_worker" => add_worker(),
                "kill" => kill(),
                "vars" => {
                    context(None);
                }
                _ => {
                    zakuro_cli(&arg0[..], Path::new("/var/run/docker.sock").exists());
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
                        zakuro_cli(&arg1[..], true);
                    }
                },
                "context" => {
                    context(Some(&arg1[..]));
                }
                _ => help(),
            }
        }
        _ => help(),
    }
}
