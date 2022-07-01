use clap::Parser;
use num_cpus;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::{io, thread};
use std::{net::IpAddr, net::TcpStream, sync::mpsc::channel, sync::mpsc::Sender};

//numer of ports available specified by TCP
const MAX: u32 = 65536;
/// Simpe port sniffer, give an ip and it will show the open ports
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// ip addrress either IPv4 or Ipv6
    #[clap(short, long, value_parser)]
    ip: IpAddr,

    //number of threads to use, default value is num of logical threads
    #[clap(short = 'j', long, value_parser, default_value_t = num_cpus::get() as u32)]
    threads: u32,
    //path to file to save to
    #[clap(short = 'o', long, value_parser)]
    path: Option<String>,
}

fn scan(tx: Sender<u32>, start_port: u32, addr: IpAddr, num_threads: u32) {
    let mut port = start_port + 1;
    loop {
        //build ip and port string
        let mut connection = addr.to_string().to_owned();
        connection.push(':');
        connection.push_str(port.to_string().as_str());

        match TcpStream::connect(connection) {
            //if connection is established send port number to receiver
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }

            Err(_) => (),
        }
        //stop looping if port number for thread is maxed out
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn save_port_list(open_ports: Vec<u32>, path: String) {
    let mut file = File::create(path).expect("Cannot open file");
    for p in open_ports {
        let mut p = p.to_string();
        p.push_str("\n");
        file.write(p.as_bytes()).expect("cannot write to file");
    }
}

fn main() {
    let args = Args::parse();

    print!("scanning {} ", args.ip);

    let num_threads = args.threads;
    let ip = args.ip;
    let (sender, receiver) = channel();

    for i in 0..num_threads {
        let sender = sender.clone();
        thread::spawn(move || scan(sender, i, ip, num_threads));
    }

    let mut open_ports = Vec::new();

    drop(sender);

    for p in receiver {
        open_ports.push(p);
    }

    println!("\n");

    open_ports.sort();

    for v in &open_ports {
        println!("open port: {}", v);
    }
    //if o flag is passed as argument then save to the specified path
    if let Some(path) = args.path {
        save_port_list(open_ports, path);
    }
}
