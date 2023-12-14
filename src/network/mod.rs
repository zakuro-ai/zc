use crate::exec;
pub fn nmap() {
    let c0 = &format!("nmap -sP 10.13.13.0/24 -oG -");
    let c1 = "awk '/Up$/{print $2}'";
    let _ = exec::cmd(&format!("{} | {}", c0, c1), Some(true));
}

pub fn nmap_inf() {
    loop {
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
pub fn wg0ip() {
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
