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
    LoginReply(LoginResult),
    BadRequest,
}
#[derive(serde::Serialize, serde::Deserialize)]

pub enum LoginResult {
    Accepted,
    BadUser,
    BadPass,
}
pub fn get_hash(input: String) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher
        .finalize()
        .as_slice()
        .iter()
        .fold(String::new(), |s, b| format!("{s}{b:X?}"))
}

// pub fn send_malformed_request(socket: &mut TcpStream) {
//     let mut data = "oh no, malformed request!".as_bytes().to_vec();
//     data.push(0);
//     socket.write(&data).unwrap();
// }

pub fn get_stream_string(stream: &mut TcpStream) -> String {
    let mut buf = String::new();

    for byte in stream.bytes() {
        match byte.expect("byte couldn't be read") {
            0 => break,
            v => buf.push(char::from_u32(v as u32).expect("non-char byte send")),
        }
    }
    stream.flush().unwrap();
    // let string = buf
    //     .bytes()
    //     .map(|b| char::from_u32(b.unwrap() as u32).unwrap())
    //     .collect::<String>()
    // ;
    buf
}

pub fn get_message(stream: &mut TcpStream) -> Result<Message, String> {
    let string = get_stream_string(stream);
    match serde_json::from_str(&string) {
        Ok(v) => Ok(v),
        Err(_) => Err(string),
    }
}
pub fn send_message(stream: &mut TcpStream, message: &Message) {
    let mut serialized = serde_json::to_string(&message).unwrap().as_bytes().to_vec();
    serialized.push(0);
    stream.write_all(&serialized).unwrap();
    stream.flush().unwrap();
    // stream.shutdown(std::net::Shutdown::Write).unwrap();
}
