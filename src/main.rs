use clap::Parser;
use num_cpus;
use std::io::Write;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use std::{io, thread};
use std::{net::IpAddr, net::TcpStream, sync::mpsc::channel, sync::mpsc::Sender};

const MAX: u32 = 65536;
/// Simpe port sniffer
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// ip addrress either IPv4 or Ipv6
    #[clap(short, long, value_parser)]
    ip: IpAddr,

    //number of threads to use, default value is num of logical threads
    #[clap(short = 'j', long, value_parser, default_value_t = num_cpus::get() as u32)]
    threads: u32,

    #[clap(short, long, value_parser, default_value = "")]
    flags: String,
}

fn scan(tx: Sender<u32>, start_port: u32, addr: IpAddr, num_threads: u32) {
    let mut port = start_port + 1;
    loop {
        let mut connection = addr.to_string().to_owned();
        connection.push(':');
        connection.push_str(port.to_string().as_str());

        match TcpStream::connect(connection) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => (),
        }
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args = Args::parse();
    println!("ip {}", args.ip);
    println!("threads {}", args.threads);
    let num_threads = args.threads;
    let (sender, receiver) = channel();

    let mut handlers: Vec<JoinHandle<()>> = Vec::new();

    for i in 0..num_threads {
        let sender = sender.clone();
        let handle = thread::spawn(move || scan(sender, i, args.ip, num_threads));
        handlers.push(handle);
    }
    let mut open_ports = Vec::new();

    drop(sender);

    for p in receiver {
        open_ports.push(p);
    }
    println!("\n");

    open_ports.sort();

    for v in open_ports {
        println!("open port: {}", v);
    }

    for handle in handlers {
        handle.join();
    }
}
