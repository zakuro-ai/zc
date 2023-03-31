use colored::Colorize;
use html5ever::rcdom::*;
use soup::prelude::*;
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;

fn exec(command: &str, print_command: Option<bool>) -> String {
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

fn zk0(command: &str) -> String {
    let docker;
    if Path::new("/usr/bin/docker").exists() {
        docker = "/usr/bin/docker";
    } else {
        docker = "/usr/local/bin/docker";
    }
    let command = format!("{} exec -i zk0 bash -c '{}' ", docker, command);
    println!("{}", &command);
    return exec(&command, Some(true));
}

fn wg0ip() {
    for iface in ifaces::Interface::get_all().unwrap().into_iter() {
        if iface.name == "wg0" {
            let addr = String::from(format!("{}", iface.addr.unwrap()));
            let v: Vec<&str> = addr.split(":").collect();
            println!("{}", v[0]);
        }
    }
}

fn nmap() {
    exec(
        "nmap -sP 10.13.13.0/24 -oG - | awk '/Up$/{print $2}'",
        Some(true),
    );
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
        return exec(&format!("zc {} ", command), Some(true));
    }
}

fn help() {
    let s = "Usage:  zc [OPTIONS] COMMAND \n
    A self-sufficient runtime for zakuro \n
    Commands: \n
        restart         Restart the zakuro service \n
        add_worker      Add a worker to the network \n
        down            Stop the container \n
        up              Start the container \n
        logs            Fetch the logs of master node \n
        pull            Pull the latest zakuro version \n
        test_network    Test the zakuro network \n
    To get more help with docker, check out our guides at https://docs.zakuro.ai/go/guides/";
    // io::stdout().write_all(s.as_bytes()).unwrap();
    println!("{}", s)
}

fn logs() {
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
        let wid = get_tag_text(worker.clone(), "a");
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
    println!(
        "{}\t\t\t{}",
        "[Nodes]".bold().blue(),
        exec(
            "nmap -sP 10.13.13.0/24 -oG - | awk '/Up$/{print $2}'",
            Some(false),
        )
        .replace("\n", "|")
    );

    println!("{}", "====WORKERS====".bold().yellow());
    for (k, v) in &dworkers {
        println!("{}", k.purple());
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
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let arg0 = &args[1];
            match &arg0[..] {
                "help" => help(),
                "nmap" => nmap(),
                "nmap_inf" => nmap_inf(),
                "wg0ip" => wg0ip(),
                "logs" => logs(),
                _ => {
                    zakuro_cli(&arg0[..], Path::new("/var/run/docker.sock").exists());
                }
            }
        }
        3 => {
            let arg0 = &args[1];
            let arg1 = &args[2];
            match &arg0[..] {
                "--docker" => {
                    zakuro_cli(&arg1[..], true);
                }
                _ => help(),
            }
        }
        _ => help(),
    }
}
