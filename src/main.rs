use core::panic;
use lb::LoadBalancer;
use std::{env, net::Ipv4Addr};

fn main() {
    // read command line arguments
    let mut args = env::args();

    // ignore the first arg since it's only this binary's name
    args.next();

    let mut ports: Vec<u16> = Vec::new();
    let mut servers: Vec<String> = Vec::new();
    // go through the arguments and check each argument if it's a port number of an IPv4 address
    // otherwise just ignore it
    while let Some(arg) = args.next() {
        if let Ok(port) = arg.parse::<u16>() {
            ports.push(port);
        } else if let Ok(_) = arg.parse::<Ipv4Addr>() {
            servers.push(arg);
        }
    }

    if servers.is_empty() {
        panic!("No servers to balance on!")
    }
    if ports.is_empty() {
        panic!("No ports to balance!")
    }

    // start balancing
    LoadBalancer::with(servers, ports);

    // to ensure the application never quits
    loop {}
}
