use std::error::Error;

use egui::Button;
use lib::Message;



fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native("login", options, Box::new(|cc| Box::new(MyApp::new(cc)))).unwrap();
}
struct MyApp {
    current_menu: Menu,
    server: Option<std::net::IpAddr>,
    ip_submission: String,
    runtime: tokio::runtime::Runtime,
    our_id: Option<lib::Uid>
    
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
            current_menu: Menu::Login {
                username: String::new(),
                password: String::new(),
                login_failure: None,
                login_now: false,
            },
            server: None,
            ip_submission: String::default(),
            runtime: tokio::runtime::Builder::new_multi_thread().enable_io().build().unwrap(),
            our_id: None
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
            let ip_input = ui.text_edit_singleline(&mut self.ip_submission);
            if (ip_input.lost_focus() || ip_input.clicked_elsewhere()) && !self.ip_submission.is_empty(){
                self.server = self.ip_submission.parse().ok();
                self.ip_submission.clear();
            };
        });
        match self.current_menu {
            Menu::Login { .. } => login_menu(&mut self.current_menu, &self.server, ctx, frame, &mut self.our_id).await,
            Menu::Contacts => contacts_menu(&mut self.current_menu, ctx, frame, &mut self.our_id),
            Menu::Chat { user_id } => chat_menu(&mut self.current_menu, ctx, frame, user_id),
        }
    });
    }
}
async fn login_menu(
    menu: &mut Menu,
    server: &Option<std::net::IpAddr>,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    our_id: &mut Option<lib::Uid>
) {
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
        let result = handle_login(server, username.clone(), password.clone(), our_id).await;
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
            #[cfg(feature = "HELLA_sus")]
            ui.collapsing("THERE IS NOTHING IN HERE", |ui|{
                ui.label("SIKE IT'S BOOBIES");
                ui.label("ok fine god have furry astolfo");

                ui.horizontal(|ui|{
                    ui.add(
                        egui::Image::new("https://static1.e621.net/data/a5/b8/a5b888a8b5dfaf28825c9e6a1ae49ff8.png")
                            .fit_to_exact_size(egui::Vec2 { x: f32::MAX, y: 400. })
                            .rounding(100.)
                    );
                    ui.add(
                        egui::Image::new("https://static1.e621.net/data/fd/1e/fd1ede23d0d4c3b617664a5a9e587c89.jpg")
                            .fit_to_exact_size(egui::Vec2 { x: f32::MAX, y: 400. })
                            .rounding(100.)
                    );
    
                })
            });
            ui.label("Welcome to login");
            egui::TextEdit::singleline(username)
                .hint_text("username")
                .show(ui);

            egui::TextEdit::singleline(password)
                .password(true)
                .hint_text("password")
                .show(ui);
            let button_response = ui.add_enabled(!(username.is_empty() || password.is_empty()), Button::new("log me in scotty"));
            *login_now = if button_response.enabled() {
                button_response.clicked()
            } else{ false };

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
    our_id: &mut Option<lib::Uid>
) -> LoginResult {
    let ip = match ip {
        Some(v) => v,
        None => return LoginResult::NoServer
    };
    let passhash = lib::get_hash(&password);
    password.clear();
    let mut stream = match tokio::net::TcpStream::connect(&std::net::SocketAddr::new(*ip, lib::PORT)).await{
        Ok(v) => v,
        Err(_e) => return LoginResult::ConnectionTimeout
    };
    lib::send_message(
        &mut stream,
        lib::Message::LoginRequest { username, password: passhash}
    ).await.unwrap();
    let response = match lib::get_message(&mut stream).await{
        Ok(m) => m,
        Err(_e) => return LoginResult::NetworkError,
    };
    match response{
        Message::LoginReply(status) => {
            match status{
                lib::LoginStatus::Accepted{ id } => {
                    eprintln!("new login, old id was {:?}; new id is {}", our_id.replace(id), id);
                    LoginResult::Success
                },
                lib::LoginStatus::BadPass => LoginResult::BadPass,
                lib::LoginStatus::BadUser => LoginResult::BadUser,
            }
        },
        _ => LoginResult::SomethingWentWrong,

    }
}
fn contacts_menu(menu: &mut Menu,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    our_id: &mut Option<lib::Uid>
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("log out").clicked() {
                handle_logout(our_id).unwrap();
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

fn handle_logout(our_id: &mut Option<lib::Uid>,) -> Result<(), Box<dyn Error>>{
    eprintln!("logout; id was {:?}", our_id.take());
    Ok(())
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
