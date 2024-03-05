use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub const MAX_USERNAME_LEN: usize = 20;

pub const PORT: u16 = 62100;

pub const ERR_MSG_STDIN: &str = "problem with stdin";

pub const ERR_MSG_STDOUT: &str = "problem with stout";

#[derive(serde::Serialize, serde::Deserialize)]

pub enum Message {
    LoginRequest { username: String, password: String },
    LoginReply(LoginStatus),
    BadRequest,
}
#[derive(serde::Serialize, serde::Deserialize)]

pub enum LoginStatus {
    Accepted,
    BadUser,
    BadPass,
}
pub fn get_hash(input: &String) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    let hash = hasher.finalize();
    let mut buf = String::with_capacity(hash.as_slice().len() * 2 + 1);
    for v in hash {
        buf.push_str(&format!("{v:X?}"));
    }
    buf
    // slice
    //     .iter()
    //     .for_each(|v| string.push(char::from_u32(*v as u32).unwrap()));
    // string
}

// pub fn send_malformed_request(socket: &mut TcpStream) {
//     let mut data = "oh no, malformed request!".as_bytes().to_vec();
//     data.push(0);
//     socket.write(&data).unwrap();
// }

pub fn get_stream_string(stream: &mut TcpStream) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();

    for byte in stream.bytes() {
        match byte {
            Ok(b) => match b{
                0 => break,
                v => buf.push(char::from_u32(v as u32).expect("non-char byte sent")),
            },
            Err(e) => {
                return Err(e.into())
            }
        }
    }
    stream.flush().unwrap();
    // let string = buf
    //     .bytes()
    //     .map(|b| char::from_u32(b.unwrap() as u32).unwrap())
    //     .collect::<String>()
    // ;
    Ok(buf)
}

pub fn get_message(stream: &mut TcpStream) -> Result<Message, Box<dyn std::error::Error>> {
    let string = get_stream_string(stream)?;
    match serde_json::from_str(&string) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.into()),
    }
}
pub fn send_message(stream: &mut TcpStream, message: &Message) -> Result<(), Box<dyn std::error::Error>>{
    let mut serialized = serde_json::to_string(&message).unwrap().as_bytes().to_vec();
    serialized.push(0);
    stream.write_all(&serialized)?;
    stream.flush().unwrap();
    Ok(())
    // stream.shutdown(std::net::Shutdown::Write).unwrap();
}
