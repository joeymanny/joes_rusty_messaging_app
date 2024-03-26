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
}
enum Menu {
    Login {
        username: String,
        password: String,
        login_failure: Option<LoginResult>,
        login_now: bool,
        network_worker: Option<std::thread::JoinHandle<LoginResult>>,
    },
    LoggedIn {
        our_id: lib::Uid,
        submenu: LoggedInMenu,
    },
}
enum LoggedInMenu{
    Contacts,
    Chat(lib::Uid)
}
#[derive(Clone, Copy, Debug)]
enum LoginResult {
    NoServer,
    ConnectionTimeout,
    NetworkError,
    SomethingWentWrong,
    BadUser,
    BadPass,
    Success{ id: lib::Uid },
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
                network_worker: None,
            },
            server: None,
            ip_submission: String::default(),
            runtime: tokio::runtime::Builder::new_multi_thread().enable_io().build().unwrap(),
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
        let next_menu = match self.current_menu {
            Menu::Login { .. } => login_menu(self.current_menu, &self.server, ctx).await,
            Menu::LoggedIn{ our_id, submenu } => logged_in_menu(submenu, ctx, frame, our_id),
            // Menu::Chat { user_id } => chat_menu(&mut self.current_menu, ctx, frame, user_id),
        };
        
    });
    }
}
async fn login_menu(
    menu: Menu,
    server_ip: &Option<std::net::IpAddr>,
    ctx: &egui::Context,
) -> Option<Menu> {
    let mut login_success: Option<lib::Uid> = None;
    let (username, password, which_login_error, try_login, network_worker) = match menu {
        Menu::Login {
            ref mut username,
            ref mut password,
            ref mut login_failure,
            ref mut login_now,
            ref mut network_worker,
        } => (username, password, login_failure, login_now, network_worker),
        _ => unreachable!(),
    };
    if *try_login { // login button was clicked last frame
        *network_worker = Some(handle_login(server_ip.clone(), username.clone(), password.clone()));
        password.clear();
        username.clear();
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
            let button_response = ui.add_enabled(
                // grayed out if fields are empty
                !(username.is_empty() || password.is_empty()),
                Button::new("log me in scotty")
            );
            *try_login = if button_response.enabled() {
                button_response.clicked()
            } else{ false };

            if let Some(e) = which_login_error{
                ui.label(format!("login error: {:?}", e));
            }
            if let Some(_) = network_worker {
                ui.label("⟳");
            }

        });
    });
    'handle_network_worker: {
        if let Some(handle) = network_worker.take(){
            let result = if !handle.is_finished(){
                // worker not done; put it back
                *network_worker = Some(handle);
                break 'handle_network_worker
            } else{
                // worker done; handle result
                match handle.join() {
                    Ok(v) => v,
                    Err(e) => panic!("fatal: couldn't join worker: {e:?}")
                }
            };
            if let LoginResult::Success{ id } = result {
                login_success = Some(id);
            } else {
                *which_login_error = Some(result);
            }
        }
    }

    if let Some(our_id) = login_success {
        Some(Menu::LoggedIn{ our_id, submenu: LoggedInMenu::Contacts })
    } else{
        None
    }
}
fn handle_login(
    ip: Option<std::net::IpAddr>,
    username: String,
    mut password: String,
) -> std::thread::JoinHandle<LoginResult> {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_io().build().unwrap();
    rt.block_on(async {
    let ip = match ip {
        Some(v) => v,
        None => return LoginResult::NoServer
    };
    let passhash = lib::get_hash(&password);
    password.clear();
    let mut stream = match tokio::net::TcpStream::connect(&std::net::SocketAddr::new(ip, lib::PORT)).await{
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
                    eprintln!("new login, new id is {}", id);
                    LoginResult::Success{ id }
                },
                lib::LoginStatus::BadPass => LoginResult::BadPass,
                lib::LoginStatus::BadUser => LoginResult::BadUser,
            }
        },
        _ => LoginResult::SomethingWentWrong,

    }
    })})
}

fn logged_in_menu(
    menu: Menu,
    submenu: LoggedInMenu,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    our_id: lib::Uid,
) -> Option<Menu> {
    match submenu {
        LoggedInMenu::Contacts => contacts_menu(submenu, ctx, frame, our_id),
        LoggedInMenu::Chat( id ) => chat_menu(submenu, ctx, our_id, id)
    }
}

fn contacts_menu(
    submenu: LoggedInMenu,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    our_id: lib::Uid,
) -> Option<Menu> {
    let mut result = None;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("log out").clicked() {
                handle_logout(our_id).unwrap();
                result = Some(Menu::Login {
                    username: String::new(),
                    password: String::new(),
                    login_failure: None,
                    login_now: false,
                    network_worker: None,
                });
            }
        });
        for i in 0..10 {
            if ui.button(i.to_string()).clicked() {
                result = Some(Menu::LoggedIn { our_id, submenu: LoggedInMenu::Chat( i ) })
            };
        }
    });
    result
}

fn handle_logout(our_id: lib::Uid,) -> Result<(), Box<dyn Error>>{
    eprintln!("logout; id was {:?}", our_id);
    Ok(())
}

fn chat_menu(
    submenu: LoggedInMenu,
    ctx: &egui::Context,
    our_id: lib::Uid,
    chat_id: lib::Uid

) -> Option<Menu> {
    let mut result = None;
    egui::CentralPanel::default().show(ctx, |ui| {
        if ui.button("back").clicked() {
            result = Some(Menu::LoggedIn { our_id, submenu: LoggedInMenu::Contacts})
        }
        ui.vertical_centered(|ui| {
            ui.label(format!("this is chat #{}!", chat_id));
        })

    });
    result
}
