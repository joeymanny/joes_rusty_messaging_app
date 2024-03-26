use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

// this type must be copy
pub type Uid = i32;

pub const MAX_USERNAME_LEN: usize = 20;

pub const PORT: u16 = 62100;

pub const ERR_MSG_STDIN: &str = "problem with stdin";

pub const ERR_MSG_STDOUT: &str = "problem with stout";

pub const ERR_SOCKET: std::net::SocketAddr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), 0));

#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub enum Message {
    LoginRequest { username: String, password: String },
    LoginReply(LoginStatus),
    BadRequest,
    InternalError,
    KeepAliveBegin(Uid)
}

pub fn which_colors(is: bool) -> (String, String, String) {
    use owo_colors::OwoColorize;
    if is {
        // dull
        (
            "notice:".to_owned(),
            "critical:".to_owned(),
            "error:".to_owned(),
        )
    } else {
        // colorful
        (
            "notice".yellow().to_string(),
            "critical".bold().red().on_black().to_string(),
            "error".red().to_string(),
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub enum LoginStatus {
    Accepted{
        id: Uid,
    },
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
    loop {
        stream.readable().await?;
        let byte = stream.read_u8().await?;
        if byte == 0b00000000 {
            break;
        } else{
            buf.push(char::try_from(byte)?)
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
pub async fn send_message(stream: &mut TcpStream, message: Message) -> Result<(), Box<dyn std::error::Error>>{
    // dbg!(serde_json::to_vec(&message))?;
    // panic!();
    let mut serialized = serde_json::to_string(&message)?.as_bytes().to_vec();
    serialized.push(0);
    stream.write(&serialized).await.unwrap();
    Ok(())
}
