use std::{net::{UdpSocket, SocketAddr}, collections::HashMap};
use protohackers_tcp_helper::{
    cli_helper::Args
};
use clap::Parser;

struct UnusualDatabase {
    key_store: HashMap<String, String>
}

impl UnusualDatabase {
    fn new() -> UnusualDatabase {
        UnusualDatabase {
            key_store: HashMap::new()
        }
    }
    fn insert_key(&mut self, key: String, value: String) {
        self.key_store.insert(key, value);
    }

    fn retrieve_key(&self, key: &str) -> Option<&String> {
        self.key_store.get(key)
    }
}

fn server_init(port: u16) -> Result<UdpSocket, std::io::Error> {
    Ok(UdpSocket::bind(format!("0.0.0.0:{port}"))?)
}

fn find_equals_index(buf: &[u8]) -> Option<usize> {
    buf.iter().position(|&c| c == b'=')
}

fn receive_datagram(socket: &UdpSocket, buf: &mut [u8]) -> Result<(usize, String), std::io::Error> {
    let (num_bytes, src_addr) = socket.recv_from(buf)?;
    println!("Source:{};Num Bytes:{}", src_addr, num_bytes);
    Ok((num_bytes, src_addr.to_string()))
}

fn send_datagram(socket: &UdpSocket, data: &[u8], addr: String) -> Result<usize, std::io::Error> {
    println!("Sending data: {:?}", data);
    socket.send_to(data, addr)
}
 
fn main() {
    let args = Args::parse();
    let udp_socket = server_init(args.port).expect("Failed to bind to port");
    let mut unusual_database = UnusualDatabase::new();
    loop {
        let mut buf = vec![];
        let (bytes_read, src_addr) = match receive_datagram(&udp_socket, &mut buf) {
            Ok(data) => data,
            Err(_) => (0, String::from("")),
        };
        if bytes_read == 0 {
            continue;
        }
        println!("Data: {:?}", buf);
        let buf_length = buf.len();
        let equals_index = find_equals_index(&buf).unwrap_or_else(|| buf_length);
        if equals_index == buf_length {
            let mut key = String::from_utf8(buf).unwrap();
            let response = match unusual_database.retrieve_key(&key) {
                Some(value) => {
                    key.push_str(value);
                    key
                }
                None => {
                    key.push('=');
                    key
                }
            };
            let _ = send_datagram(&udp_socket, response.as_bytes(), src_addr);
            continue;
        }
        let key;
        let value;

        if equals_index == buf_length - 1 {
            key = String::from("");
            value = String::from("");
        } else {
            key = String::from_utf8(buf[..equals_index].to_vec()).unwrap_or_else(|_| String::from(""));
            value = String::from_utf8(buf[equals_index + 1..].to_vec()).unwrap_or_else(|_| String::from(""));
        }
        unusual_database.insert_key(key, value);
    }
}