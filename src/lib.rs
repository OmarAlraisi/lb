use std::{
    io::{Read, Write},
    net::{IpAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};

pub struct LoadBalancer;

impl LoadBalancer {
    pub fn with(servers: Vec<String>, ports: Vec<u16>) {
        // make the servers an atomic RC to pass it into threads
        let servers = Arc::new(servers);
        // iterate over all the ports provided by the user and create a listener in a seperate
        // thread
        for port in ports {
            let listener = TcpListener::bind(&format!("0.0.0.0:{}", port))
                .expect(&format!("Failed to listen to port {}", port));
            let servers = servers.clone();
            thread::spawn(move || {
                // wait for a connection
                while let Ok((client_stream, sock)) = listener.accept() {
                    // choose a backend server to handle this request
                    let handler = format!(
                        "{}:{}",
                        servers[match sock.ip() {
                            IpAddr::V4(ip) => {
                                let octets = ip.octets();
                                u32::from_be_bytes(octets) as usize % servers.len()
                            }
                            IpAddr::V6(_) => unimplemented!(),
                        }],
                        port
                    );
                    thread::spawn(move || {
                        // connect to a backend server
                        if let Ok(server_stream) = TcpStream::connect(handler) {
                            // To write from client to server
                            let mut client_rx = client_stream
                                .try_clone()
                                .expect("Failed to clone a reader stream");
                            let mut server_tx = server_stream
                                .try_clone()
                                .expect("Failed to clone a transmitter stream");

                            thread::spawn(move || {
                                let mut buf = [0u8; 1024];
                                while let Ok(nread) = client_rx.read(&mut buf[..]) {
                                    if nread == 0 {
                                        break;
                                    }
                                    server_tx.write(&buf[..nread]).unwrap();
                                }
                            });

                            // To write from server to client
                            let mut client_tx = client_stream
                                .try_clone()
                                .expect("Failed to clone a transmitter stream");
                            let mut server_rx = server_stream
                                .try_clone()
                                .expect("Failed to clone a reader stream");

                            thread::spawn(move || {
                                let mut buf = [0u8; 1024];
                                while let Ok(nread) = server_rx.read(&mut buf[..]) {
                                    if nread == 0 {
                                        break;
                                    }
                                    client_tx.write(&buf[..nread]).unwrap();
                                }
                            });
                        }
                    });
                }
            });
        }
    }
}
