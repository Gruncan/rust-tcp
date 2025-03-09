use std::{env, io, thread};
use std::io::{Error, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

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
    let stream = TcpStream::connect(format!("{}:{}", ip, PORT))?;
    Ok(stream)
}

fn send_data(mut tcp_stream: &TcpStream, dns: &str) -> Result<(), Error> {
    let message = format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", dns);
    tcp_stream.write_all(message.as_bytes())
}

fn read_data(mut tcp_stream: &TcpStream) -> Result<String, Error> {
    let mut buffer = Vec::new();
    tcp_stream.read_to_end(&mut buffer)?;

    let response = String::from_utf8(buffer)
        .map_err(|e| Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(response)
}

fn receiver_executor_wrapper(receiver_channel: Receiver<TcpStream>, transmitter_channel: Sender<TcpStream>, dns: &str) -> Result<(), Error> {
    while let Ok(dns_tcp_stream) = receiver_channel.recv() {
        if send_data(&dns_tcp_stream, &dns).is_ok(){
            drop(transmitter_channel);
            let data = read_data(&dns_tcp_stream)?;
            println!("{}", data);
            break;
        } else{
            eprintln!("Tcp stream failed to send data!");
        }
    }
    Ok(())
}

fn worker_executor_wrapper(ip_address: &str, transmitter_channel: Sender<TcpStream>, start_channel: Receiver<()>) -> Result<(), Error> {
    if start_channel.recv().is_err() {
        return Err(Error::new(io::ErrorKind::Other, "Failed to receive start!"));
    }

    let tcp_stream = initialise_connection(&ip_address)?;

    if transmitter_channel.send(tcp_stream).is_err() {
        return Err(Error::new(io::ErrorKind::Other, "Failed to send tcp_stream!"));
    }


    Ok(())
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
    let mut start_channels = Vec::with_capacity(ip_addresses.len());

    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    let receiver = thread::spawn(move || {
        match receiver_executor_wrapper(rx, tx_clone, &dns) {
            Ok(_) => {},
            Err(e) => {eprintln!("Error: {}", e); return;}
        }
    });


    for ip_address in ip_addresses {
        let (stx, srx) = mpsc::channel();
        start_channels.push(stx);
        let tx_cpy = tx.clone();
        thread_handles.push(thread::spawn(move || {
            worker_executor_wrapper(&*ip_address, tx_cpy, srx)
        }));
    }

    for start_channel in start_channels {
        start_channel.send(()).unwrap();
    }

    receiver.join().unwrap();

}
