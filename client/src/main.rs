use owo_colors::OwoColorize;
use std::{io::Write, net::SocketAddr};

fn main() {
    let mut layer1_choice = String::new();
    let mut server_addr = None;
    'main: loop {
        println!("welcome. menu options are:\n    login\tlog in to messaging app\n    server\tdefine a server (menu)\n    exit  \texit the program");
        std::io::stdin().read_line(&mut layer1_choice).unwrap();
        match layer1_choice.trim() {
            "login" => handle_login(server_addr),
            "exit" => break 'main,
            "server" => server_addr = server_address_menu(),
            _ => (),
        }
        layer1_choice.clear();
    }
    // let mut writer = TcpStream::connect("192.168.40.12:8080").unwrap();
    // let mut username = String::new();
    // println!("e");
    // std::io::stdin().read_line(&mut username).unwrap();
    // let mut hasher = sha2::Sha256::new();
    // hasher.update(username.as_bytes());
    // let user_hash = hasher.finalize();
    // writer.write(format!("{}", user_hash.as_slice().into_iter().map(|v| format!("{v:X?}")).collect::<String>() ).as_bytes()).unwrap();
}

fn server_address_menu() -> Option<std::net::SocketAddr> {
    let mut buf = String::new();
    loop {
        print!("set server ip address: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut buf).unwrap();
        if buf == "back\n".to_owned() {
            return None;
        }
        match buf.trim().parse::<std::net::IpAddr>() {
            Ok(v) => {
                println!("success!\nserver ip set to {v}");
                return Some(SocketAddr::new(v, lib::SOCKET));
            }
            Err(_) => {
                println!("invalid ip. try again or go back with `back`");
            }
        }
        buf.clear();
    }
}

fn handle_login(socket: Option<SocketAddr>) {
    if let None = socket {
        println!(
            "{} no server defined; returning to previous menu",
            "warning".red()
        );
        return;
    }
    let username = get_username();
    let password = get_password();
    let request = lib::Message::LoginRequest {
        username,
        password: lib::get_hash(password),
    };
    match std::net::TcpStream::connect(socket.unwrap()) {
        Ok(mut stream) => {
            stream
                .write(serde_json::to_string(&request).unwrap().as_bytes())
                .unwrap();
            stream.flush().unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
        }
        Err(_) => (),
    }
}

fn get_password() -> String {
    let mut buf = String::new();
    print!("password: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut buf).unwrap();
    lib::get_hash(buf.trim().to_owned())
}
fn get_username() -> String {
    loop {
        let mut buf = String::new();
        print!("username: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut buf).unwrap();
        if buf.len() < lib::MAX_USERNAME_LEN {
            return lib::get_hash(buf.trim().to_owned());
        }
        println!("20 character limit");
        buf.clear();
    }
}
