use clap::Parser;
use owo_colors::OwoColorize;
use std::{io::Write, net::SocketAddr};
use lib::{Message, ReplyType};

#[derive(clap::Parser, Debug)]
struct Arguments{
    #[arg(short='s', long="server")]
    server: Option<std::net::IpAddr>
}

fn main() {
    let mut main_menu_choice = String::new();
    let mut server_addr = if let Arguments{ server: Some(ip) } = Arguments::parse() {
        Some(std::net::SocketAddr::new(ip, lib::PORT))
    }else{ None };
    if let None = server_addr{
        eprintln!("{} no server address configured, use the `server` menu to set one", "notice".yellow());
    }
    'main: loop {
        println!("welcome. menu options are:\n    login\tlog in to messaging app\n    server\tdefine a server (menu)\n    exit  \texit the program");
        std::io::stdin().read_line(&mut main_menu_choice).expect(lib::ERR_MSG_STDIN);
        match main_menu_choice.trim() {
            "login" => handle_login(server_addr),
            "exit" => break 'main,
            "server" => server_address_menu(&mut server_addr),
            _ => println!("{} unrecognized menu option", "error".red()),
        }
        main_menu_choice.clear();
    }
}

fn server_address_menu(server: &mut Option<SocketAddr>) {
    match server{
        Some(v) => println!("server: {}", v.ip()),
        None => println!("server: unset"),
    }
    let mut buf = String::new();
    loop {
        print!("set server ip address: ");
        std::io::stdout().flush().expect(lib::ERR_MSG_STDOUT);
        std::io::stdin().read_line(&mut buf).expect(lib::ERR_MSG_STDIN);
        if buf == "back\n".to_owned() {
            return;
        }
        match buf.trim().parse::<std::net::IpAddr>() {
            Ok(v) => {
                println!("{} server ip set to {v}", "success!".green());
                *server =  Some(SocketAddr::new(v, lib::PORT));
                return;
            }
            Err(_) => {
                println!("invalid ip. try again or go back with `back`");
            }
        }
        buf.clear();
    }
}

fn handle_login(socket: Option<SocketAddr>) {
    let socket = match socket {
            None => {println!(
                "{} no server defined; returning to previous menu",
                "notice".yellow()
            );
            return;
        },
        Some(v) => v
    };
    let username = get_username();
    let password = get_password();
    let request = lib::Message::LoginRequest {
        username,
        password: lib::get_hash(password),
    };
    match std::net::TcpStream::connect(socket) {
        Ok(mut stream) => {
            lib::send_message(&mut stream, &request);
            let message = match lib::get_message(&mut stream){
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{} malformed request from server: {}", "error".red(), e);
                    return
                }
            };

            match message {
                Message::LoginReply(reply) => {
                    match reply {
                        ReplyType::Accepted => { println!("{}", "login accepted!".green()) },
                        ReplyType::BadPass => { println!("{} bad password", "warning".red()) },
                        ReplyType::BadUser => { println!("{} bad user", "warning".red()) }
                    }
                },
                Message::BadRequest => {
                    println!("{}", "we sent a request the server didn't understand!".red())
                },
                Message::LoginRequest { username: _, password: _ } => {
                    println!("uh the server is asking us to login wtf")
                }

            }
            lib::send_message(&mut stream, &request);
        
        },
        _ => ()
    }
}

fn get_password() -> String {
    let mut buf = String::new();
    print!("password: ");
    std::io::stdout().flush().expect(lib::ERR_MSG_STDOUT);
    std::io::stdin().read_line(&mut buf).expect(lib::ERR_MSG_STDIN);
    lib::get_hash(buf.trim().to_owned())
}
fn get_username() -> String {
    loop {
        let mut buf = String::new();
        print!("username: ");
        std::io::stdout().flush().expect(lib::ERR_MSG_STDOUT);
        std::io::stdin().read_line(&mut buf).expect(lib::ERR_MSG_STDIN);
        if buf.len() < lib::MAX_USERNAME_LEN {
            return lib::get_hash(buf.trim().to_owned());
        }
        println!("20 character limit");
        buf.clear();
    }
}
