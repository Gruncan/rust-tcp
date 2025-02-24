use std::env;
use std::net::ToSocketAddrs;
use std::path::Path;

const PORT: u32 = 80;


fn program_name(path : &String) -> String {
    Path::new(path.as_str()).file_name().unwrap().to_str().unwrap().to_string()
}




fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ./{} <dns server>", program_name(&args[0]));
        return;
    }
    let dns = &args[1];
    println!("Searching ips for: {}...", dns);
    let ip_addresses = format!("{}:{}", dns, PORT).to_socket_addrs();
    match ip_addresses {
        Ok(addresses) => {
            for addr in addresses {
                println!("{}", addr.to_string());
            }
        }
        Err(e) => {
            eprintln!("Failed to find ip addresses! {}", e);
        }
    }
}
