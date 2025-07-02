use cli_clipboard;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::tailscale::Tailscale;
mod tailscale;

slint::slint! {
    import { ScrollView } from "std-widgets.slint";

    export component ToggleSwitch inherits Rectangle {
        callback toggled;
        in-out property <string> text;
        in-out property <bool> checked;
        in-out property<bool> enabled <=> touch-area.enabled;
        height: 20px;
        horizontal-stretch: 0;
        vertical-stretch: 0;

        HorizontalLayout {
            spacing: 8px;
            indicator := Rectangle {
                width: 40px;
                border-width: 1px;
                border-radius: root.height / 2;
                border-color: self.background.darker(25%);
                background: root.enabled ? (root.checked ? #12aa20: white)  : white;
                animate background { duration: 100ms; }

                bubble := Rectangle {
                    width: root.height - 8px;
                    height: bubble.width;
                    border-radius: bubble.height / 2;
                    y: 4px;
                    x: 4px + self.a * (indicator.width - bubble.width - 8px);
                    property <float> a: root.checked ? 1 : 0;
                    background: root.checked ? white : (root.enabled ? #999999 : gray);
                    animate a, background { duration: 200ms; easing: ease;}
                }
            }

            Text {
                min-width: max(100px, self.preferred-width);
                text: root.text;
                vertical-alignment: center;
                color: root.enabled ? black : gray;
            }
        }

        touch-area := TouchArea {
            width: root.width;
            height: root.height;
            clicked => {
                if (root.enabled) {
                    root.checked = !root.checked;
                    root.toggled();
                }
            }
        }
    }

    export struct MachineData  {
        ip: string,
        hostname: string,
        user: string,
        os: string,
        online: bool,
        details: string,
    }

    component Machine inherits Rectangle {
        callback clicked;

        in property <string> ip;
        in property <string> name;
        in property <bool> is_online;

        HorizontalLayout {
            spacing: 5px;
            alignment: space-between;
            padding-left: 6px;
            padding-right: 12px;
            padding-top: 2px;

            HorizontalLayout{
                spacing: 5px;

                Rectangle {
                    background: is_online ?#125619 : #888888;
                    width: 12px;
                    height: 12px;
                    border-radius: 6px;
                }

                Text {
                    text: ip;
                    color: #bbbbbb;
                    width: 100px;
                    horizontal-alignment: right;
                }
            }

            Text {
                text: name;
                color: #ffffff;
            }
        }

        TouchArea {
            clicked => {
                root.clicked();
            }
        }
    }

    export component MainWindow inherits Window {
        width: 326px;
        height: 326px;
        always-on-top: true;
        title: "Tailscale";
        icon: @image-url("imgs/tailscale-dark.svg");

        callback toggle();
        callback copy_machine_ip(string);

        in property <bool> is_on;
        in property <[MachineData]> machines: [];
        in property <bool> copy_success: false;

        VerticalLayout{
            spacing: 5px;

            Rectangle {
                background: #aaaaaa;
                height: 35px;

                ToggleSwitch {
                    x: 12px;
                    checked: is_on;
                    text: is_on ? "tailscale running" : "tailscale stopped";
                    toggled => {
                        toggle();
                    }
                }
            }

            ScrollView {
                VerticalLayout {
                    spacing: 5px;

                    for tile[i] in machines : Machine {
                        ip: tile.ip;
                        name: tile.hostname;
                        is_online: tile.online;
                        clicked => {
                            root.copy_machine_ip(tile.ip);
                        }
                    }
                }
            }
        }

        if copy_success : Rectangle {
            background: black;
            height: 35px;
            y: root.height - self.height;

            Text {
                text: "successfully copied";
                color: #ffffff;
            }
        }
    }
}

pub struct TailscaleState {
    pub enabled: bool,
    pub machines: Vec<MachineData>,
}

fn main() {
    use slint::Model;

    let main_window = MainWindow::new().unwrap();
    update_tailscale_state(&main_window);

    let main_window_weak = main_window.as_weak();

    // toggle tailscale
    let main_window_weak_for_toggle = main_window_weak.clone();
    main_window.on_toggle(move || {
        let _ = Tailscale::toggle();
        let main_window = main_window_weak_for_toggle.unwrap();
        update_tailscale_state(&main_window);
    });

    // copy machine ip
    let main_window_weak_for_copy = main_window_weak.clone();
    main_window.on_copy_machine_ip(move |ip| {
        let res = cli_clipboard::set_contents(ip.into());

        if res.is_err() {
            return;
        }

        let main_window = main_window_weak_for_copy.unwrap();
        main_window.set_copy_success(true);
        slint::Timer::single_shot(std::time::Duration::from_secs(1), move || {
            main_window.set_copy_success(false);
        });
    });

    main_window.run().unwrap();
}

fn update_tailscale_state(main_window: &MainWindow) {
    let enabled = Tailscale::is_enabled().unwrap_or(false);
    main_window.set_is_on(enabled);

    let machines = match enabled {
        false => vec![],
        true => Tailscale::status().unwrap_or(vec![]),
    };

    let machine_model = Rc::new(slint::VecModel::from(machines));
    main_window.set_machines(machine_model.clone().into());
}
