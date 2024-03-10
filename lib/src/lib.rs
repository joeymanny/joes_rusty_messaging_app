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
    InternalError
}
#[derive(serde::Serialize, serde::Deserialize)]


pub enum LoginStatus {
    Accepted,
    BadUser,
    BadPass,
}
pub fn get_hash(input: &String) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha512::new();
    hasher.update(input);
    let hash = hasher.finalize();
    let mut buf = String::with_capacity(128);
    for v in hash.into_iter() {
        buf.push_str(&format!("{v:02X}"));
    }
    buf
}
pub async fn get_stream_string(stream: &mut TcpStream) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    stream.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;
    for byte in stream.bytes() {
        match byte {
            Ok(b) => match b{
                0 => break,
                v => buf.push(char::from_u32(v as u32).unwrap()), // todo: handle invalid chars
            },
            Err(e) => {
                return Err(e.into())
            }
        }
    }
    Ok(buf)
}

pub async fn get_message(stream: &mut TcpStream) -> Result<Message, Box<dyn std::error::Error>> {
    let string = get_stream_string(stream).await?;
    match serde_json::from_str(&string) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.into()),
    }
}
pub fn send_message(stream: &mut TcpStream, message: &Message) -> Result<(), Box<dyn std::error::Error>>{
    let mut serialized = serde_json::to_string(&message)?.as_bytes().to_vec();
    serialized.push(0);
    stream.write_all(&serialized)?;
    Ok(())
}
