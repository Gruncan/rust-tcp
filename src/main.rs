use std::env;
use std::io::{Error, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;

const PORT: u32 = 80;


fn program_name(path : &String) -> String {
    Path::new(path.as_str()).file_name().unwrap().to_str().unwrap().to_string()
}

fn get_ip_addresses(dns: &str) -> Result<Vec<String>, Error> {
    let ip_addresses = format!("{}:{}", dns, PORT).to_socket_addrs()?;

    Ok(ip_addresses.map(|address| address.ip().to_string())
        .collect()
    )
}

fn initialise_connection(ip: &str) -> Result<TcpStream, Error> {
    TcpStream::connect(format!("{}:{}", ip, PORT))
}

fn send_data(mut tcp_stream: TcpStream, dns: &str) -> Result<(), Error> {
    tcp_stream.write_all(format!("{} {}", b"GET / HTTP/1.1\r\nHost: {}\r\n\r\n", dns).as_bytes())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ./{} <dns server>", program_name(&args[0]));
        return;
    }
    let dns = &args[1];
    println!("Searching ips for: {}...", dns);
    let ip_addresses = get_ip_addresses(&dns);
    match ip_addresses {
        Ok(addresses) => {
            for address in addresses {
                let connection = initialise_connection(&address);
                if let Ok(_) = connection {
                    println!("Connection successful on IP: {}", address);
                    break;
                }else{
                    println!("Failed to connect to IP: {}", address);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
