
pub(crate) mod tcp_connection_sm {
    use std::io::{Error, Read, Write};
    use std::net::{Shutdown, TcpStream};

    const PORT: u32 = 80;

    pub(crate) struct TcpDisconnected;


    pub(crate) struct TcpConnected{
        pub(crate) connection_stream: TcpStream,
        dns: String,
    }


    pub(crate) fn init_tcp_connection_sm() -> TcpDisconnected {
        TcpDisconnected{}
    }

    impl TcpDisconnected {

        pub fn connect(self, ip_addr: &str, dns: &str) -> Result<TcpConnected, Error>{
            let stream = TcpStream::connect(format!("{}:{}", ip_addr, PORT))?;
            Ok(TcpConnected{connection_stream: stream, dns: String::from(dns)})
        }
    }

    impl TcpConnected {


        pub fn send_get_request(mut self) -> Result<TcpConnected, Error> {
            let message = format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", self.dns);
            self.connection_stream.write_all(message.as_bytes())?;
            Ok(self)
        }

        pub fn read_response(mut self) -> Result<(TcpConnected, String), Error> {
            let mut buffer = Vec::new();


            self.connection_stream.read_to_end(&mut buffer)?;

            let response = String::from_utf8(buffer)
                    .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

            Ok((self, response))
        }

        pub fn disconnect(self) -> TcpDisconnected {
            // If shutdown fails most likely already shut so just move to disconnect anyways
            self.connection_stream.shutdown(Shutdown::Both).unwrap_or(());
            TcpDisconnected
        }

    }


}