use std::{
    net::Ipv4Addr, sync::Arc
};
use tokio::net::{
    TcpListener, TcpStream
};

use clap::Parser;
use lib::Message;
use owo_colors::OwoColorize;
use sqlx::{Pool, Postgres};

const FALLBACK_ADDRESS: std::net::IpAddr = std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[allow(unused)]
struct KeepAlive {
    time: std::time::SystemTime,
    stream: std::net::TcpStream,
    ongoing_message: Vec<u8>,
}


#[derive(clap::Parser, Debug)]
#[command(about = "server for joe's messaging app")]
struct Arguments {
    #[arg(short = 'b', long = "bind", help = "which ip address to listen for connections from")]
    ip: Option<std::net::IpAddr>,

    #[arg(short = 'c', long = "cpus", help = "number of cpus to use to handle connections")]
    cpus: Option<usize>,

    #[arg(long = "no-color", help = "whether to use colorful output")]
    no_color: bool,

    #[arg(short = 'u', long = "user", help = "postgres user to connect to the database with; defaults to `messaging_app_user`")]
    postgres_user: Option<String>,

    #[arg(short = 'd', long = "database", help = "name of database to connect to as `user`; defaults to `messaging_app`")]
    postgres_db: Option<String>,

    #[arg(short = 's', long = "socket", help = "unix socket to find postgres server; defaults to `/var/run/postgresql`")]
    postgres_socket: Option<String>,

    #[arg(short = 'p', long = "port", help = "which port to connect to the postgres server with; defaults to `5432`")]
    postgres_port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let Arguments { ip, cpus, no_color, postgres_user, postgres_db, postgres_socket, postgres_port } = Arguments::parse();
    let (notice, critical, error) = lib::which_colors(no_color);
    let num_cpu = cpus.unwrap_or_else(|| {
        let n = num_cpus::get();
        eprintln!(
            "{notice} no thread count supplied, defaulting to number available ({n})"
        );
        n
    });
    let address: std::net::IpAddr = match ip {
        Some(ip) => ip,
        None => {
            eprintln!("{notice} no ip provided, defaulting to 127.0.0.1");
            FALLBACK_ADDRESS
        }
    };
    let postgres_port = match postgres_port {
        Some(v) => v,
        None => {
            eprintln!("{notice}: no port set, defaulting to {}", lib::PORT);
            5432
        }
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(num_cpu as u32)
        .connect_with(
            sqlx::postgres::PgConnectOptions::new()
                .socket(postgres_socket.unwrap_or("/var/run/postgresql".into()))
                .port(postgres_port)
                .username(&postgres_user.unwrap_or("messaging_app_user".into()))
                .database(&postgres_db.unwrap_or("messaging_app".into())),
        ).await.unwrap();
    let pool = std::sync::Arc::new(pool);
    let listener: tokio::net::TcpListener = 
            match TcpListener::bind(
                std::net::SocketAddr::new(
                    address,
                    lib::PORT
                )
            ).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{critical} couldn't bind to {} : {}", address, e);
                    panic!("can't serve without a valid binding")
                }
            };

    let mut workers: Vec<(Arc<tokio::sync::Mutex<bool>>, tokio::sync::mpsc::Sender<tokio::net::TcpStream>, tokio::sync::mpsc::Receiver<()>)> = vec![];
    
    for i in 0..num_cpu {
        let is_ready = Arc::new(tokio::sync::Mutex::new(false));

        let (ready_sender, mut ready_receiver) = tokio::sync::mpsc::channel::<tokio::net::TcpStream>(1);

        let (ack_sender, ack_receiver) = tokio::sync::mpsc::channel::<()>(1);

        workers.push((Arc::clone(&is_ready), ready_sender, ack_receiver));
        // 49152-65535
        let id = i;
        let error = error.clone();
        let notice = notice.clone();
        let _critical = critical.clone();
        let pool = Arc::clone(&pool);
        // rt.block_on(future);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_time().enable_io().build().unwrap();
            rt.block_on(async {
            loop {
                {
                *is_ready.lock().await = true;      // we are ready
                }
                // block waiting for incoming task
                let mut stream = match tokio::task::block_in_place(|| ready_receiver.blocking_recv()){
                    Some(v) => {
                        v
                    },
                    None => break // break loop, ending work
                };
                {
                    *is_ready.lock().await = false; // we no longer are ready
                }
                // acknowledge that lock is ready- prevents more work being schedule while awaiting lock
                ack_sender.send(()).await.expect("couldn't send ack");
                eprintln!("sent ack");
                let data = match lib::get_stream_string(&mut stream).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("{error}: couldn't read data from {}: {e:?}", stream.peer_addr().unwrap_or(lib::ERR_SOCKET));
                        continue;
                    }
                };
                let message = match serde_json::from_str(&data) {
                    Ok(v) => v,
                    Err(_) => {
                        match lib::send_message(&mut stream, lib::Message::BadRequest).await {
                            Err(e) => {
                                eprintln!("{error}: coundn't send BadRequest reply to {}: {e:?}", stream.peer_addr().unwrap_or(lib::ERR_SOCKET));
                                continue;
                            }
                            _ => { /* no issue sending error */ }
                        };
                        println!("{notice}: bad request from {:?}: {data}", stream.peer_addr().unwrap_or(lib::ERR_SOCKET));
                        continue;
                    }
                };
                match message {
                    lib::Message::LoginRequest { username, password } => {
                        println!("login request from {}", stream.peer_addr().unwrap_or(lib::ERR_SOCKET));
                        handle_login(username, password, &pool, &mut stream).await;
                        eprintln!("handled by thread #{id}");
                    },
                    _ => { eprintln!("{notice}: unhandles messages from {}: {message:?}", stream.peer_addr().unwrap_or(lib::ERR_SOCKET)); }
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
        // continually accept streams
        let (stream, _) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => { eprintln!("error accepting connection: {e}"); break }
        };
        'find_worker: loop {
            // look at all workers
            for (is_ready, channel, ack_receiver) in workers.iter_mut(){
                match is_ready.try_lock(){
                    // worker is available
                    Ok(is_ready) if *is_ready => {
                        std::mem::drop(is_ready);
                        channel.send(stream).await.expect("channel should be open");
                        // wait for lock to be updated
                        eprintln!("awaiting ack");
                        tokio::task::block_in_place(||ack_receiver.blocking_recv()).expect("couldn't receiver ack");
                        break 'find_worker
                    },
                    _ => (),
                }
                // worker wasn't found: next worker
            }
            // no worker was found, loop
        }
    }
}

// #[derive(sqlx::FromRow)]
struct UserTableRow {
    id: lib::Uid,
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
    let rows: Vec<UserTableRow> = sqlx::query_as!( UserTableRow,
        "SELECT * FROM users WHERE $1 = users.username",
        username
        )
        // .bind(username)
        .fetch_all(pool)
        .await
        .unwrap();
    match rows.len(){
        0 => lib::send_message(stream, Message::LoginReply(lib::LoginStatus::BadUser)),
        1 => {
            if password == rows[0].password {
                lib::send_message(stream, Message::LoginReply(lib::LoginStatus::Accepted{ id: rows[0].id }))
            } else {
                lib::send_message(stream, Message::LoginReply(lib::LoginStatus::BadPass))
            }
        },
        _ => {
            // there are duplicate users, very bad
            lib::send_message(stream, Message::InternalError)
        }
    }.await.expect("couldn't send reply");
    // if let Err(e) = lib::send_message(stream, &Message::LoginReply(lib::LoginStatus::Accepted)) {
    //     eprintln!("coundn't send data: {e:?}");
    // };
}

