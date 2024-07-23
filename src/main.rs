#[allow(warnings)]
extern crate serde;
extern crate toml;

use std::env;
mod common;
mod exec;
mod envs;
mod manager;
mod network;


fn update() {
    let _ = exec::tty("curl https://get.zakuro.ai/zc | sh");
}


fn launch() {
    manager::pull();
    manager::kill();
    manager::restart();
}
fn setup() {
    common::download_auth();
    common::download_conf();
    launch();
}



fn help() {

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
      kill            Remove current running zakuro containers.
      restart         Restart the containers with updated images.
      wg0ip           Get the IP in the cluster.
      rmi             Remove zakuro images.
      workers         List all workers connect to the main clusters (10.13.13.2).
      context <path>  Set new zakuro context | untested.
    
\nTo get more help with docker, check out our guides at https://docs.zakuro.ai/");
}


fn main() {

    envs::update();
    if let (Ok(_zakuro_auth),) = (
        env::var("ZAKURO_AUTH"),
    ){
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
                    "ps" => manager::ps(),
                    "dist" => {
                        match common::dist(){
                            Ok(res) => {
                                println!("{}", res);
                            }
                            Err(why) => {
                                eprintln!("{}", why);
                            }
                        }
                    },
                    "images" => manager::images(),
                    // "nmap" => network::nmap(),
                    "launch" => launch(),
                    "download_conf" => common::download_conf(),
                    "download_auth" => common::download_auth(),
                    "setup" => setup(),
                    "pull" => manager::pull(),
                    "workers" => exec::execute_fn(manager::workers()),
                    "wg0ip" => exec::execute_fn(network::wg0ip(Some(false))),
                    "logs" => common::logs(true),
                    "nodes" => manager::nodes(),
                    "restart" => manager::restart(),
                    "servers" => manager::server_list(),
                    "add_worker" => manager::add_worker(),
                    "kill" => manager::kill(),
                    "connect" => manager::connect(),
                    "update" => update(),
                    "rmi" => manager::rmi(),
                    "--version" => common::version(),
                    "-v" => common::version(),
                    "vars" => {
                        common::context(None);
                    },
                    _ => {
                        help()
                    }
                }
            }
            3 => {
                let arg0 = &args[1];
                let arg1 = &args[2];
                match &arg0[..] {
                    "--docker" => match &arg1[..] {
                        "rm" => manager::remove_container(),
                        _ => {
                            let _ = exec::zk0(&arg1[..]);
                        }
                    },
                    "-d" => match &arg1[..] {
                        "rm" => manager::remove_container(),
                        _ => {
                            let _ = exec::zk0(&arg1[..]);
                        }
                    },
                    "push" => {
                        manager::push(Some(&arg1[..]));
                    },
                    "context" => {
                        common::context(Some(&arg1[..]));
                    },
                    "build" => {
                        common::build(Some(args));
                    }
                    _ => help(),
                }
            }
            _ => {
                let arg0 = &args[1];
                let _arg1 = &args[2];
                match &arg0[..] {
                    "build" => {
                        common::build(Some(args));
                    }
                    _ => {
                        help();
                    }
                }
            }
        }
    }
    else{
        eprintln!("Oops something bad happened!\n\nYou are missing ZAKURO_AUTH.\nPlease request access and try again.\n");
        return
    }
}

