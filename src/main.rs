use std::{env, thread};
use std::io::{Error, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

const PORT: u32 = 80;


struct DnsTcpStream {
    stream: TcpStream,
    ip: String,
    dns: String
}

fn program_name(path : &String) -> String {
    Path::new(path.as_str()).file_name().unwrap().to_str().unwrap().to_string()
}

fn get_ip_addresses(dns: &str) -> Result<Vec<String>, Error> {
    let ip_addresses = format!("{}:{}", dns, PORT).to_socket_addrs()?;

    Ok(ip_addresses.map(|address| address.ip().to_string())
        .collect()
    )
}

fn initialise_connection(ip: &str) -> Result<DnsTcpStream, Error> {
    let stream = TcpStream::connect(format!("{}:{}", ip, PORT))?;
    Ok(DnsTcpStream{stream, ip: ip.to_string(), dns: String::new()})
}

fn send_data(mut tcp_stream: TcpStream, dns: &str) -> Result<(), Error> {
    let message = format!("{} {}", "GET / HTTP/1.1\r\nHost: {}\r\n\r\n", dns);
    tcp_stream.write_all(message.as_bytes())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ./{} <dns server>", program_name(&args[0]));
        return;
    }
    let dns = args[1].clone();
    println!("Searching ips for: {}...", dns);
    let ip_addresses_res = get_ip_addresses(&dns);
    if let Err(e) = ip_addresses_res {
        eprintln!("Error: {}", e);
        return;
    }
    let ip_addresses = ip_addresses_res.unwrap();

    let mut thread_handles = Vec::with_capacity(ip_addresses.len());
    let (tx, rx) = mpsc::channel();
    let tx_clone: Sender<DnsTcpStream> = tx.clone();
    let reciever = thread::spawn(move || {
        while let Ok(dns_tcp_stream) = rx.recv() {
            println!("Sent data!");
            if let Err(error) = send_data(dns_tcp_stream.stream, &dns){
                eprintln!("Error ({}): {}", dns_tcp_stream.ip, error);
            } else{
                println!("Data sent successfully to {}!", dns_tcp_stream.ip);
                drop(tx);
                break;
            }
        }
    });
    println!("Ip addresses: {:?}", ip_addresses);
    for ip_address in ip_addresses {
        let tx_cpy = tx_clone.clone();
        thread_handles.push(thread::spawn(move || {
            if let Ok(dns_tcp_stream) = initialise_connection(&ip_address){
                // dns_tcp_stream.dns = dns;
                if let Err(e) = tx_cpy.send(dns_tcp_stream){
                    eprintln!("Error: {}", e);
                }
            }
        }));
    }


    for handle in thread_handles {
        handle.join().unwrap();
    }
    reciever.join().unwrap();
}
