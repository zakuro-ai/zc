use std::error::Error as StdError;
use crate::exec;

// #[allow(warnings)]
// pub fn nmap() {
//     let c0 = &format!("nmap -sP 10.13.13.0/24 -oG -");
//     let c1 = "awk '/Up$/{print $2}'";
//     let _ = exec::cmd(&format!("{} | {}", c0, c1), Some(true));
// }


pub fn wg0ip(force_local:Option<bool>)  -> Result<Option<String>,  Box<(dyn StdError + 'static)>>{
    for iface in ifaces::Interface::get_all().unwrap().into_iter() {
        if iface.name == "wg0" {
            if let Some(addr) = iface.addr {
                let addr_str = addr.to_string();
                if addr_str.starts_with("10.13.13") {
                    return Ok(Some(addr_str.trim_end_matches(":0").to_string()));
                }
            }
        }
    }
    match  force_local {
        Some(true) => {
            return Ok(None)
        }
        _ => {
            match exec::zk0("zc wg0ip") {
                Ok(addr_str) => {
                    return Ok(addr_str);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

    }
}