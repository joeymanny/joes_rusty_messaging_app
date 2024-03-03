use std::net::{Ipv4Addr, TcpStream};

use clap::Parser;
use lib::Message;
use owo_colors::OwoColorize;

#[derive(clap::Parser, Debug)]
struct Arguments {
    #[arg(short = 'b', long = "bind")]
    ip: Option<std::net::IpAddr>,
}

fn main() {
    let Arguments { ip } = Arguments::parse();
    let address: std::net::IpAddr = match ip {
        Some(ip) => ip,
        None => {
            eprintln!(
                "{} no ip provided, defaulting to 127.0.0.1",
                "notice".yellow()
            );
            std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        }
    };
    let listener = match std::net::TcpListener::bind(std::net::SocketAddr::new(address, lib::PORT))
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{} couldn't bind to {}: {}",
                "critical".bold().red().on_black(),
                address,
                e
            );
            panic!("can't serve without a valid binding")
        }
    };
    eprintln!("serving on {}", address.blue());
    loop {
        // accept an incoming message
        let (mut stream, client_sock) = match listener.accept() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{} problem accepting tcp session: {}", "error".red(), e);
                continue;
            }
        };
        println!("incoming connection from {}", client_sock.ip());
        // interpret as a string
        let data = lib::get_stream_string(&mut stream);
        let message = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => {
                lib::send_message(&mut stream, &lib::Message::BadRequest);
                println!("bad request from {}; terminating session", client_sock.ip());
                continue;
            }
        };
        #[allow(clippy::single_match)]
        match message {
            lib::Message::LoginRequest { username, password } => {
                println!("login request from {}", client_sock.ip());
                handle_login(username, password, &mut stream);
            }
            _ => (),
        }
    }
}
fn handle_login(username: String, password: String, stream: &mut TcpStream) {
    dbg!(username);
    dbg!(password);
    lib::send_message(stream, &Message::LoginReply(lib::LoginResult::Accepted));
}
