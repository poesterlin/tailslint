use cli_clipboard;
use gtk::gdk::keys::constants::Break;
use std::thread;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
};

// Assuming your tailscale module is still present
use crate::tailscale::Tailscale;
mod tailscale;

#[derive(Clone, Debug)]
pub struct MachineData {
    pub ip: String,
    pub hostname: String,
    pub online: bool,
}

const TOGGLE_ID: &str = "toggle";
const QUIT_ID: &str = "quit";
const REFRESH_ID: &str = "refresh";

fn main() {
    let handle = thread::spawn(run_tray_app);
    handle.join().unwrap();
}
/// Runs the entire tray application logic within the GTK event loop.
fn run_tray_app() {
    // --- 1. Initialization ---
    gtk::init().unwrap();
    let icon = load_icon("imgs/tailscale-32x32.png");

    // A simple message enum to communicate from the event handler to the main thread.
    // This MUST be `Send`.
    enum AppMessage {
        Toggle,
        Refresh,
        Quit,
        CopyIp(String),
    }

    // --- 2. Create the channel for cross-thread communication ---
    let (tx, rx) = std::sync::mpsc::channel::<AppMessage>();

    // --- 3. Create the initial menu and tray icon on the main thread ---
    // The `tray_icon` variable will be moved into the idle handler later.
    // It NEVER leaves the main GTK thread.
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(rebuild_menu()))
        .with_tooltip("Tailscale Control")
        .with_icon(icon)
        .build()
        .unwrap();

    // --- 4. Set the Global Event Handler ---
    // The handler's closure only captures the `tx` sender, which IS thread-safe.
    tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        let (toggle_id, refresh_id, quit_id) = get_menu_item_ids();
        let event_id = event.id();

        // The handler's only job is to send a message.
        let msg = if event_id == &toggle_id {
            AppMessage::Toggle
        } else if event_id == &refresh_id {
            AppMessage::Refresh
        } else if event_id == &quit_id {
            AppMessage::Quit
        } else {
            AppMessage::CopyIp(event_id.0.clone())
        };

        tx.send(msg).unwrap();
    }));

    glib::source::idle_add_local(move || {
        if let Ok(message) = rx.try_recv() {
            match message {
                AppMessage::Refresh => {
                    // We are on the main thread, so we can safely call `set_menu`.
                    tray_icon.set_menu(Some(Box::new(rebuild_menu())));
                }
                AppMessage::Toggle => {
                    let _ = Tailscale::toggle();
                    // We are on the main thread, so we can safely call `set_menu`.
                    tray_icon.set_menu(Some(Box::new(rebuild_menu())));
                }
                AppMessage::CopyIp(ip) => {
                    if cli_clipboard::set_contents(ip.clone()).is_ok() {
                        println!("Copied IP {} to clipboard!", ip);
                    } else {
                        eprintln!("Failed to copy IP {} to clipboard.", ip);
                    }
                }
                AppMessage::Quit => {
                    println!("Quitting...");
                    gtk::main_quit();
                    return glib::ControlFlow::Break;
                }
            }
        }

       glib::ControlFlow::Continue
    });

    // --- 6. Run the GTK Event Loop ---
    gtk::main();
}

/// Rebuilds the menu based on current state.
fn rebuild_menu() -> Menu {
    let menu = Menu::new();
    let is_enabled = Tailscale::is_enabled().unwrap_or(false);
    let (toggle_item, refresh_item, quit_item) = build_control_items(is_enabled);

    if is_enabled {
        menu.append_items(&[
            &toggle_item,
            &refresh_item,
            &PredefinedMenuItem::separator(),
        ])
        .unwrap();

        let machines = Tailscale::status().unwrap_or_else(|_| vec![]);
        for machine in machines {
            let icon = if machine.online { "🟢" } else { "⚫" };
            let text = format!("{} {} ({})", icon, machine.hostname, machine.ip);
            let id = MenuId::new(machine.ip.clone());
            let machine_item = MenuItem::with_id(id, text, true, None);

            menu.append(&machine_item).unwrap();
        }
    } else {
        menu.append(&toggle_item).unwrap();
    }

    menu.append_items(&[&PredefinedMenuItem::separator(), &quit_item])
        .unwrap();

    menu
}

/// Helper to create control menu items.
fn build_control_items(is_enabled: bool) -> (MenuItem, MenuItem, MenuItem) {
    let toggle_text = if is_enabled {
        "Turn Tailscale Off"
    } else {
        "Turn Tailscale On"
    };
    let toggle_item = MenuItem::with_id(MenuId::new(TOGGLE_ID), toggle_text, true, None);
    let refresh_item = MenuItem::with_id(MenuId::new(REFRESH_ID), "Refresh", true, None);
    let quit_item = MenuItem::with_id(MenuId::new(QUIT_ID), "Quit", true, None);
    (toggle_item, refresh_item, quit_item)
}

/// Helper to get the IDs of control items without modifying state.
fn get_menu_item_ids() -> (MenuId, MenuId, MenuId) {
    let (toggle, refresh, quit) = build_control_items(true);
    (toggle.id().clone(), refresh.id().clone(), quit.id().clone())
}

/// Helper function to load a PNG icon for the tray.
fn load_icon(path: &str) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
