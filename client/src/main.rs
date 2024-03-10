use lib::Message;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native("login", options, Box::new(|cc| Box::new(MyApp::new(cc)))).unwrap();
}
struct MyApp {
    menu: Menu,
    server: Option<std::net::IpAddr>,
    ip_submission: String,
    runtime: tokio::runtime::Runtime
}
#[derive(Clone)]
enum Menu {
    Login {
        username: String,
        password: String,
        login_failure: Option<LoginResult>,
        login_now: bool,
    },
    Contacts,
    Chat {
        user_id: u32,
    },
}
#[derive(Clone, Copy, Debug)]
enum LoginResult {
    NoServer,
    ConnectionTimeout,
    NetworkError,
    SomethingWentWrong,
    BadUser,
    BadPass,
    Success,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_style(egui::style::Style {
            override_text_style: Some(egui::style::TextStyle::Monospace),
            ..egui::style::Style::default()
        });
        egui_extras::install_image_loaders(&cc.egui_ctx);

        MyApp {
            menu: Menu::Login {
                username: String::new(),
                password: String::new(),
                login_failure: None,
                login_now: false,
            },
            server: None,
            ip_submission: String::default(),
            runtime: tokio::runtime::Builder::new_multi_thread().build().unwrap()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.runtime.block_on(async{
        egui::TopBottomPanel::bottom("server panel").show(ctx, |ui| {
            ui.label(format!(
                "current server: {}",
                if let Some(v) = self.server {
                    v.to_string()
                } else {
                    "None".to_owned()
                }
            ));
            if ui
                .text_edit_singleline(&mut self.ip_submission)
                .lost_focus()
            {
                self.server = self.ip_submission.parse().ok();
                self.ip_submission.clear();
            };
        });
        match self.menu {
            Menu::Login { .. } => login_menu(&mut self.menu, &self.server, ctx, frame).await,
            Menu::Contacts => contacts_menu(&mut self.menu, ctx, frame),
            Menu::Chat { user_id } => chat_menu(&mut self.menu, ctx, frame, user_id),
        }
    });
    }
}
async fn login_menu(menu: &mut Menu, server: &Option<std::net::IpAddr>, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let mut login: bool = false;
    let (username, password, login_failure, login_now) = match menu {
        Menu::Login {
            ref mut username,
            ref mut password,
            ref mut login_failure,
            ref mut login_now,
        } => (username, password, login_failure, login_now),
        _ => panic!("this is unreachable"),
    };
    if *login_now {
        let result = handle_login(server, username.clone(), password.clone()).await;
        password.clear();
        username.clear();
        if let LoginResult::Success = result {
            login = true;
        } else {
            *login_failure = Some(result);
        }
    }
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            #[cfg(feature = "HELLA sus")]
            ui.collapsing("THERE IS NOTHING IN HERE", |ui|{
                ui.image("https://static1.e621.net/data/a5/b8/a5b888a8b5dfaf28825c9e6a1ae49ff8.png")
            });
            ui.label("Welcome to login");
            egui::TextEdit::singleline(username)
                .hint_text("username")
                .show(ui);
            egui::TextEdit::singleline(password)
                .password(true)
                .hint_text("password")
                .show(ui);
            *login_now = ui.button("log me in scotty").clicked();
            if let Some(e) = login_failure{
                ui.label(format!("login error: {:?}", e));
            }
            if *login_now {
                ui.label("⟳");
            }

        });
    });
    if login {
        *menu = Menu::Contacts
    }
}
async fn handle_login(
    ip: &Option<std::net::IpAddr>,
    username: String,
    mut password: String,
) -> LoginResult {
    let ip = match ip {
        Some(v) => v,
        None => return LoginResult::NoServer
    };
    let passhash = lib::get_hash(&password);
    password.clear();
    let mut stream = match std::net::TcpStream::connect_timeout(&std::net::SocketAddr::new(*ip, lib::PORT), std::time::Duration::from_secs(1)){
        Ok(v) => v,
        Err(_e) => return LoginResult::ConnectionTimeout
    };
    lib::send_message(
        &mut stream,
        &lib::Message::LoginRequest { username, password: passhash}
    ).unwrap();
    let response = match lib::get_message(&mut stream).await{
        Ok(m) => m,
        Err(_e) => return LoginResult::NetworkError,
    };
    match response{
        Message::LoginReply(status) => {
            match status{
                lib::LoginStatus::Accepted => LoginResult::Success,
                lib::LoginStatus::BadPass => LoginResult::BadPass,
                lib::LoginStatus::BadUser => LoginResult::BadUser,
            }
        },
        _ => LoginResult::SomethingWentWrong,

    }
}
fn contacts_menu(menu: &mut Menu, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("log out").clicked() {
                *menu = Menu::Login {
                    username: String::new(),
                    password: String::new(),
                    login_failure: None,
                    login_now: false,
                }
            }
        });
        for i in 0..10 {
            if ui.button(i.to_string()).clicked() {
                *menu = Menu::Chat { user_id: i }
            };
        }
    });
}
fn chat_menu(menu: &mut Menu, ctx: &egui::Context, _frame: &mut eframe::Frame, chat_id: u32) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if ui.button("back").clicked() {
            *menu = Menu::Contacts;
        }
        ui.vertical_centered(|ui| {
            ui.label(format!("this is chat #{}!", chat_id));
        })
    });
}
