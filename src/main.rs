use std::{env, thread};
use std::io::{Error, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const PORT: u32 = 80;

mod dns_concurrent {
    use std::io::{Error, Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    use std::sync::mpsc;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;
    use std::thread::JoinHandle;
    use crate::PORT;

    struct IpChannels {
        ip_addr: String,
        _channel_receiver: Receiver<()>
    }

    pub struct DnsTcpStream {
        stream: TcpStream,
        ip: String,
        dns: String
    }

    pub struct DnsIdle;

    pub struct AvailableTcpConnections {
        ip_addresses: Vec<IpChannels>,
        dns_name: String,
        _tx: Sender<DnsTcpStream>,
        _rx: Receiver<DnsTcpStream>
    }

    pub struct WaitingConcurrentConnections {
        _receive_handler: JoinHandle<()>,
        _worker_handler: Vec<JoinHandle<()>>,
    }



    impl DnsIdle {

        pub fn init_state_machine() -> DnsIdle {
            DnsIdle
        }

        pub fn query_ips(self, dns: &str) -> Result<AvailableTcpConnections, Error>{
            let socket_addresses = format!("{}:{}", dns, PORT).to_socket_addrs()?;


            for socket_addr in socket_addresses{
                socket_addr.ip().to_string()
            }
            let ip_addresses = ip_addresses_res
                .map(|address| IpChannels{
                                            ip_addr: address.ip().to_string(),
                    _channel_receiver: channel().1}).collect();
            Ok(AvailableTcpConnections::new(ip_addresses, String::from(dns)))
        }
    }

    impl AvailableTcpConnections {

        pub fn new(ip_addresses: Vec<String>, dns_name: String) -> Self{
            let (_tx, _rx) = mpsc::channel();
            AvailableTcpConnections {
                ip_addresses,
                dns_name,
                _tx,
                _rx
            }
        }

        pub fn init_concurrent_connections(self) -> WaitingConcurrentConnections{
            let rec_thread = self.spawn_receiver_thread();
            let worker_threads = self.spawn_worker_threads();
            WaitingConcurrentConnections{
                _receive_handler: rec_thread,
                _worker_handler: worker_threads
            }
        }

        pub fn reset(self) -> DnsIdle {
            DnsIdle::init_state_machine()
        }

        fn spawn_receiver_thread(&self) -> JoinHandle<()>{
            let rec_func = thread::spawn(move || {
                while let Ok(dns_tcp_stream) = self._rx.recv() {
                    println!("Sent data!");
                    if let Err(error) = send_data(&dns_tcp_stream.stream, &self.dns_name){
                        eprintln!("Error ({}): {}", dns_tcp_stream.ip, error);
                    } else {
                        if let Ok(data) = rec_data(dns_tcp_stream.stream){
                            println!("Data:\n {}", data);
                        }else{
                            println!("Failed to read");
                            continue;
                        }
                        println!("Data sent successfully to {}!", dns_tcp_stream.ip);
                        drop(self._tx.clone());
                        break;
                    }
                    }
            });
            rec_func
        }

        fn spawn_worker_threads(&self) -> Vec<JoinHandle<()>> {
            let mut thread_handles = Vec::with_capacity(self.ip_addresses.len());
            for ip_address in self.ip_addresses {
                let tx_cpy = self._tx.clone();
                thread_handles.push(thread::spawn(move || {
                    if let Ok(dns_tcp_stream) = initialise_connection(&ip_address){
                        // dns_tcp_stream.dns = dns;
                        if let Err(e) = tx_cpy.send(dns_tcp_stream){
                            eprintln!("Error: {}", e);
                        }
                    }
                }));
            }
            thread_handles
        }
    }




    fn send_data(mut tcp_stream: &TcpStream, dns: &str) -> Result<(), Error> {
        let message = format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", dns);
        tcp_stream.write_all(message.as_bytes())
    }

    fn rec_data(mut tcp_stream: TcpStream) -> Result<String, Error> {
        let mut buffer = Vec::new();

        tcp_stream.read_to_end(&mut buffer)?;

        String::from_utf8(buffer).map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn initialise_connection(ip: &str) -> Result<DnsTcpStream, Error> {
        let stream = TcpStream::connect(format!("{}:{}", ip, PORT))?;
        Ok(DnsTcpStream{stream, ip: ip.to_string(), dns: String::new()})
    }

}


struct TcpDisconnected;


impl TcpDisconnected {

    fn initialise_connection(self, dns_data: &DnsReturned) -> Result<TcpConnected, Error> {
        let stream = TcpStream::connect(format!("{}:{}", dns_data., PORT))?;
        Ok(TcpConnected{stream, ip: ip.to_string(), dns: String::new()})
    }

}

struct TcpConnected{
    stream: TcpStream
}





fn reciever_func(rx: Receiver<TcpStream>) {

}

fn program_name(path : &String) -> String {
    Path::new(path.as_str()).file_name().unwrap().to_str().unwrap().to_string()
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
            if let Err(error) = send_data(&dns_tcp_stream.stream, &dns){
                eprintln!("Error ({}): {}", dns_tcp_stream.ip, error);
            } else{
                if let Ok(data) = rec_data(dns_tcp_stream.stream){
                    println!("Data:\n {}", data);
                }else{
                    println!("Failed to read");
                    continue;
                }
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
