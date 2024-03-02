
pub const MAX_USERNAME_LEN: usize = 20;

pub const SOCKET: u16 = 62100;

#[derive(serde::Serialize, serde::Deserialize)]

pub enum Message{
    LoginRequest{
        username: String,
        password: String,
    },
    LoginReply(ReplyType)
}
#[derive(serde::Serialize, serde::Deserialize)]

pub enum ReplyType{
    Accepted,
    BadUser,
    BadPass,
}
pub fn get_hash(input: String) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().as_slice().into_iter().map(|v| format!("{v:X?}")).collect::<String>()
}