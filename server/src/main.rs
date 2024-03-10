use std::{
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::Arc,
};

use clap::Parser;
use lib::Message;
use owo_colors::OwoColorize;
use sqlx::{Pool, Postgres};

const FALLBACK_ADDRESS: std::net::IpAddr = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(clap::Parser, Debug)]
#[command(about = "server for joe's messaging app")]
struct Arguments {
    #[arg(short = 'b', long = "bind", help = "ip address to bind to")]
    ip: Option<std::net::IpAddr>,
    #[arg(short = 'c', long = "cpus", help = "number of cpus to use")]
    cpus: Option<usize>,
    #[arg(long = "no-color", help = "whether to use colorful output")]
    no_color: bool,
}

fn main() {
    let Arguments { ip, cpus, no_color } = Arguments::parse();
    let (notice, critical, error) = which_colors(no_color);
    let num_cpu = cpus.unwrap_or_else(|| {
        let n = num_cpus::get();
        eprintln!(
            "{} no thread count supplied, defaulting to number available ({n})",
            notice
        );
        n
    });
    let rt = tokio::runtime::Runtime::new()
    .expect("problem getting tokio runtime");
    let pool = rt
        .block_on(
            sqlx::postgres::PgPoolOptions::new()
                .min_connections(num_cpu as u32)
                .connect_with(
                    sqlx::postgres::PgConnectOptions::new()
                        .socket("/var/run/postgresql")
                        .port(5432)
                        .username("messaging_app_user")
                        .database("messaging_app"),
                ),
        )
        .unwrap();
    let pool = std::sync::Arc::new(pool);
    let address: std::net::IpAddr = match ip {
        Some(ip) => ip,
        None => {
            eprintln!("{} no ip provided, defaulting to 127.0.0.1", notice);
            FALLBACK_ADDRESS
        }
    };

    let listener: std::net::TcpListener =
        match TcpListener::bind(std::net::SocketAddr::new(address, lib::PORT)) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{} couldn't bind to {} : {}", critical, address, e);
                panic!("can't serve without a valid binding")
            }
        };
    for i in 0..num_cpu {
        let listener = listener
            .try_clone()
            .unwrap_or_else(|e| panic!("thread #{i} counldn't clone tcp listener: {e}"));
        let id = i;
        let error = error.clone();
        let pool = Arc::clone(&pool);
        // rt.block_on(future);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_time().build().unwrap();
            rt.block_on(async {
            loop {
                // block waiting for incoming message
                let (mut stream, client_sock) = match listener.accept() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("{} problem accepting tcp session: {}", error, e);
                        continue;
                    }
                };
                println!("incoming connection from {}", client_sock.ip());

                let data = match lib::get_stream_string(&mut stream).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("error getting data: {e:?}");
                        continue;
                    }
                };
                let message = match serde_json::from_str(&data) {
                    Ok(v) => v,
                    Err(_) => {
                        match lib::send_message(&mut stream, &lib::Message::BadRequest) {
                            Err(e) => {
                                eprintln!("coundn't send BadRequest reply: {e:?}");
                                continue;
                            }
                            _ => { /* no issue sending error */ }
                        };
                        println!("bad request from {}", client_sock.ip());
                        continue;
                    }
                };
                match message {
                    lib::Message::LoginRequest { username, password } => {
                        println!("login request from {}", client_sock.ip());
                        handle_login(username, password, &pool, &mut stream).await;
                        eprintln!("handled by thread #{id}");
                    }
                    _ => { /* handle other messages here */ }
                }
            }
            })
        });
    }
    eprintln!(
        "serving on {} with {num_cpu} threads",
        if no_color {
            address.to_string()
        } else {
            address.blue().to_string()
        }
    );
    loop {
        std::thread::sleep(std::time::Duration::MAX);
    }
}

#[derive(sqlx::FromRow)]
struct UserTableRow{
    id: i32,
    username: String,
    password: String,
    email: Option<String>
}

async fn handle_login(
    username: String,
    password: String,
    pool: &Pool<Postgres>,
    stream: &mut TcpStream,
) {
    eprintln!("{username}");
    eprintln!("{password}");
    let rows: Vec<UserTableRow> = sqlx::query_as("SELECT * FROM users WHERE $1 = users.username")
        .bind(username)
        .fetch_all(pool)
        .await
        .unwrap();
    match rows.len(){
        0 => lib::send_message(stream, &Message::LoginReply(lib::LoginStatus::BadUser)),
        1 => {
            if password == rows[0].password {
                lib::send_message(stream, &Message::LoginReply(lib::LoginStatus::Accepted))
            } else {
                lib::send_message(stream, &Message::LoginReply(lib::LoginStatus::BadPass))
            }
        },
        _ => {
            lib::send_message(stream, &Message::InternalError)
        }
    }.expect("couldn't send reply");
    if let Err(e) = lib::send_message(stream, &Message::LoginReply(lib::LoginStatus::Accepted)) {
        eprintln!("coundn't send data: {e:?}");
    }
}
fn which_colors(is: bool) -> (String, String, String) {
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
