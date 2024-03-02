use std::{io::Read, net::Ipv4Addr};

fn main() {
    let listener = std::net::TcpListener::bind(std::net::SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), lib::SOCKET)).expect("problem binding 127.0.0.1");
    println!("serving on 127.0.0.1");
    loop{
        let (stream, _) = listener.accept().expect("problem accepting tcp stream");
        let data = stream.bytes().map(|v| char::from_u32(v.unwrap() as u32).unwrap()).collect::<String>();
        let message: lib::Message = if let Ok(v) = serde_json::from_str(&data){ v } else { continue };
        match message{
            lib::Message::LoginRequest { username, password } => {
                handle_login(username, password);
            },
            _ => ()
        }
    }
}

fn handle_login(username: String, password: String){
    dbg!(username);
    dbg!(password);
}