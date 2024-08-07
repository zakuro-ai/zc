#[allow(warnings)]
use std::str;
use crate::exec;
use std::{env, fs};
use crate::envs;
use std::path::Path;
use std::{collections::HashMap, fs::File};
use html5ever::rcdom::*;
use soup::prelude::*;
use colored::Colorize;
use std::io::Write;
use std::rc::Rc;

pub fn print_debug(s: &str) {
    if cfg!(debug_assertions) {
        println!("DEBUG: {}", s);
    }
}


// The file `built.rs` was placed there by cargo and `build.rs`
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}


pub fn dist() -> Result<String, String>  {
    if let Ok(dist_value) = exec::cmd("echo $(uname -m)", None){
        if dist_value=="aarch64"{
            Ok(String::from("aarch64"))
        }
        else if dist_value=="x86_64"{
            Ok(String::from("amd64"))
        }
        else{
            Ok(String::from("arm64"))
        }        
    }
    else{
        Err(format!("Couldn't extract dist"))
    }
}

pub fn build(image: Option<Vec<String>>){
    if let (Some(image_value),Ok(dist_str)) = (
        image,
        dist()
    ) {
        let truncated_vector: Vec<String> = image_value.into_iter().skip(2).collect();
        let d;
        if &dist_str=="arm64"{
            d = "aarch64"
        }
        else if &dist_str=="amd64"{
            d ="amd64"
        }
        else{
            d=&dist_str;
        }        
        let s: String = truncated_vector.join(" ");
        
        let _ = exec::cmd(
            &format!("BUILDARCH={} BUILDARCHI={} docker compose build {}", 
            d,
            &dist_str,
            s),
         None);
    }
}


fn create_directory(path: &str) -> std::io::Result<()> {
    let path = Path::new(path);
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn create_directories() {
    envs::update();
    if let Ok(zakuro_home) = env::var("ZAKURO_HOME") {
        //Create dirs
        for image in vec![
            "config", "network", "storage", "compute", "node", "hub", "lib", "logs", "bin",
        ] {
            if let Err(e) =  create_directory(&format!("{}/{}", zakuro_home, image)) {
                eprintln!("Error creating directory: {}", e);
            }
        }
    } 
}

pub fn download_conf() {
    envs::update();
    if let Ok(zakuro_home) = env::var("ZAKURO_HOME") {
        create_directories();

        // Download the default config
        let _ = exec::tty(
            &format!(
                "wget -q 'http://get.zakuro.ai/zk0?config=default' -O {}/default-zakuro.yaml",
                zakuro_home
            ),
        );

        //Download confs
        for image in vec!["network", "storage", "compute", "node", "hub"] {
            let _ = exec::tty(
                &format!(
                    "wget -q 'http://get.zakuro.ai/zk0?config={}' -O {}/{}/{}-zakuro.yaml",
                    image, zakuro_home, image, image
                ),
            );
            let _ = exec::tty(
                &format!(
                    "wget -q 'http://get.zakuro.ai/zk0?config={}_env' -O {}/{}/.env",
                    image, zakuro_home, image
                ),
            );
        }
    }
}


pub fn download_auth() {
    envs::update();
    if let (Ok(zakuro_home), Ok(zakuro_auth)) = (
        env::var("ZAKURO_HOME"),
        env::var("ZAKURO_AUTH"),
    ) {
        if !zakuro_home.is_empty() && !zakuro_auth.is_empty() {
            print_debug(&format!("{}, {}", zakuro_home, zakuro_auth));
            create_directories();
            let command = &format!(
                "curl -s --location 'https://get.zakuro.ai/profile' \
                --header 'Content-Type: application/json' \
                --data '{{\"pkey\": \"{}\"}}' > {}/config/wg0.conf",
                zakuro_auth, zakuro_home
            );

            match exec::cmd(command, Some(false)) {
                Ok(_) => { 
                    print_debug(&format!("{}", &command));
                }
                Err(e) => {
                    eprintln!("Error :{}", e);
                }
            }
        }
    } else {
        eprintln!("Missing ZAKURO_CONTEXT or ZAKURO_AUTH");
    }
}

pub fn context(path: Option<&str>) {
    // envs::update();
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
        // println!("{:?}", vars);
    }
}

pub fn version()
{
    if let (Some(short_hash),) = (
        built_info::GIT_COMMIT_HASH_SHORT,
    ) {

        let built_time = built::util::strptime(built_info::BUILT_TIME_UTC);
        println!("zc version {}, build {} on {}", built_info::PKG_VERSION, short_hash, built_time.with_timezone(&built::chrono::offset::Local),
);
}
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
    // let html = exec("curl -s http://spark-master.zakuro.ai:8080", Some(false));
    let html = match exec::stdout("curl -s http://spark-master.zakuro.ai:8080") {
        Ok(html)=>{
            html
        }
        Err(why) => {
            eprintln!("Failed to execute the command: {:?}", why);
            return; // Exit the function on error
        }

    };
    println!("{}", html);
    // let soup = Soup::new(&html);
    // let body = &get_tag_soup(soup, "body");
    // // let h3 = &get_tag_rec(body.clone(), &mut vec!["div", "div", "div", "h3"]);
    // let h3 = &get_tag_rec(body.clone(), &mut vec!["div", "div", "div", "h3"]);
    // let version = clean(get_tag_rec(h3.clone(), &mut vec!["a", "span"]).text());
    // let h3_text = clean(h3.text());
    // let splits: Vec<&str> = h3_text.split("at ").collect();
    // let url = splits[1];
    // let tables = get_all(body.clone(), "tbody");

    // let mut dworkers = HashMap::new();
    // let trs = get_all(tables[0].clone(), "tr");
    // let mut alive_workers = 0;
    // let mut cores_used = 0;
    // let mut cores_total = 0;
    // let mut mem_used = 0.0;
    // let mut mem_total = 0.0;
    // let mut app_running = 0;
    // let mut app_completed = 0;

    // for worker in trs {
    //     // let wid = get_tag_text(worker.clone(), "a");
    //     let wid = worker.text();
    //     let worker_info = get_all(worker.clone(), "td");
    //     dworkers.insert(
    //         clean(wid),
    //         HashMap::from([
    //             ("status", clean(worker_info[2].text())),
    //             ("cores", clean(worker_info[3].text())),
    //             ("mem", clean(worker_info[4].text())),
    //             ("resources", clean(worker_info[5].text())),
    //         ]),
    //     );
    //     if clean(worker_info[2].text()) == "ALIVE" {
    //         alive_workers += 1;
    //         // Cores
    //         let cores = clean(worker_info[3].text());
    //         let mut splits: Vec<&str> = cores.split(" ").collect();
    //         let cores_total_worker: i32 = splits[0].parse().unwrap();
    //         cores_total += cores_total_worker;
    //         splits = splits[1].split("(").collect();
    //         splits = splits[1].split(" ").collect();
    //         let cores_used_worker: i32 = splits[0].parse().unwrap();
    //         cores_used += cores_used_worker;
    //         // Mem
    //         let mut mem = clean(worker_info[4].text());
    //         mem = mem.replace("(", "").replace(")", "");
    //         splits = mem.split(" ").collect();

    //         let mut mem_total_worker: f64 = splits[0].parse().unwrap();
    //         let mem_total_worker_dim = splits[1];
    //         let mut mem_used_worker: f64 = splits[2].parse().unwrap();
    //         let mem_used_worker_dim = splits[3];

    //         if mem_total_worker_dim == "GiB" {
    //             mem_total_worker *= 1000.0;
    //         }
    //         if mem_used_worker_dim == "GiB" {
    //             mem_used_worker *= 1000.0;
    //         }

    //         mem_total += mem_total_worker;
    //         mem_used += mem_used_worker;
    //     }
    // }
    // mem_total /= 1000.0;
    // mem_used /= 1000.0;

    // let mut dapp = HashMap::new();
    // let trs = get_all(tables[1].clone(), "tr");
    // for app in trs {
    //     let aid = get_tag_text(app.clone(), "a");
    //     let app_info = get_all(app.clone(), "td");
    //     dapp.insert(
    //         clean(aid),
    //         HashMap::from([
    //             ("cores", clean(app_info[2].text())),
    //             ("mem", clean(app_info[3].text())),
    //             ("resources", clean(app_info[4].text())),
    //             ("submitted", clean(app_info[5].text())),
    //             ("user", clean(app_info[6].text())),
    //             ("status", clean(app_info[7].text())),
    //             ("duration", clean(app_info[8].text())),
    //         ]),
    //     );
    //     if clean(app_info[7].text()) == "RUNNING" {
    //         app_running += 1;
    //     }
    // }

    // let trs = get_all(tables[2].clone(), "tr");
    // for app in trs {
    //     let _aid = get_tag_text(app.clone(), "a");
    //     let app_info = get_all(app.clone(), "td");
    //     if clean(app_info[7].text()) == "FINISHED" {
    //         app_completed += 1;
    //     }
    // }
    // // println!("{:?}", dworkers);
    // // println!("{:?}", dapp);
    // println!("{}\t\t{}", "[VERSION]".blue().bold(), version);
    // println!("{}\t\t\t{}", "[URL]".blue().bold(), url);
    // println!("{}\t\t{}", "[Alive Workers]".blue().bold(), alive_workers);
    // println!(
    //     "{}\t\t{}/{} used",
    //     "[Cores in use]".blue().bold(),
    //     cores_used,
    //     cores_total
    // );
    // println!(
    //     "{}\t\t{}/{} GiB used",
    //     "[Memory in use]".blue().bold(),
    //     mem_used,
    //     mem_total
    // );
    // println!(
    //     "{}\t\t{} Running, {} Completed",
    //     "[Applications]".bold().blue(),
    //     app_running,
    //     app_completed
    // );

    // println!("{}", "====WORKERS====".bold().yellow());
    // for (k, v) in &dworkers {
    //     let mut condition = true;
    //     if alive {
    //         if v["status"] != "ALIVE" {
    //             condition = false;
    //         }
    //     }

    //     if condition {
    //         println!("{}", k.purple());
    //     }
    // }
    // println!("{}", "====APPS====".bold().yellow());
    // for (k, v) in &dapp {
    //     let s = format!(
    //         "{}@{} allocated {} cores, {} memory",
    //         v["user"], k, v["cores"], v["mem"]
    //     );
    //     println!("{}", s.purple());
    // }
}
