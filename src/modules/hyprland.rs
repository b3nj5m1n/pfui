use anyhow::Result;
use clap::Subcommand;
use hyprland::{
    data::{Client, Devices, Workspace, Workspaces},
    event_listener::EventListener,
    prelude::*,
};
use serde::Serialize;

#[derive(Subcommand)]
pub enum HyprlandOpts {
    Workspace,
    Window,
    Keyboard,
}

#[derive(Serialize)]
struct WorkspaceData {
    is_active: bool,
    data: Workspace,
}

pub struct HyprlandListener {
    listener: EventListener,
}

#[derive(Debug, Serialize)]
struct KeyboardLayout(hyprland::data::Keyboard);
impl HyprlandListener {
    pub fn new(opts: &HyprlandOpts) -> Self {
        let mut listener = EventListener::new();
        match opts {
            HyprlandOpts::Workspace => {
                let print_workspace = || {
                    if let Ok(wspaces) = Workspaces::get() {
                        let active_workspace = Workspace::get_active();
                        let mut wspaces: Vec<_> = wspaces
                            .into_iter()
                            .map(|w| {
                                let is_active =
                                    matches!(&active_workspace, Ok(space) if space.id == w.id);
                                WorkspaceData { is_active, data: w }
                            })
                            .collect();
                        wspaces.sort_by_key(|wspace| match wspace.data.id {
                            hyprland::shared::WorkspaceType::Unnamed(id) => id,
                            hyprland::shared::WorkspaceType::Named(_) => i32::MAX,
                            hyprland::shared::WorkspaceType::Special(_) => i32::MAX,
                        });
                        crate::print(&Some(wspaces));
                    } else {
                        crate::print::<()>(&None);
                    }
                };
                // for initial;
                print_workspace();
                listener.add_workspace_added_handler(move |wtype| {
                    eprintln!("Workspace {wtype:?} added");
                    print_workspace()
                });
                listener.add_workspace_moved_handler(move |mon_event| {
                    eprintln!("Moniter changed: {mon_event:?}");
                    print_workspace()
                });
                listener.add_workspace_change_handler(move |wtype| {
                    eprintln!("Workspace {wtype:?} changed");
                    print_workspace()
                });
                listener.add_workspace_destroy_handler(move |wtype| {
                    eprintln!("Workspace {wtype:?} removed");
                    print_workspace()
                });
                listener.add_fullscreen_state_change_handler(move |_state| {
                    print_workspace();
                });
            }
            HyprlandOpts::Window => {
                let print_window = || {
                    if let Ok(Some(client)) = Client::get_active() {
                        crate::print(&Some(client.class));
                    } else {
                        crate::print::<()>(&None);
                    }
                };
                listener.add_window_open_handler(move |win_event| {
                    eprintln!("Active window opened: {win_event:?}");
                    print_window();
                });
                listener.add_window_close_handler(move |win_event| {
                    eprintln!("Window closed {win_event:?}");
                    print_window();
                });
                listener.add_window_moved_handler(move |win_event| {
                    eprintln!("Window moved: {win_event:?}");
                    print_window();
                });
                listener.add_active_window_change_handler(move |win_event| {
                    eprintln!("Window changed: {win_event:?}");
                    print_window();
                });
            }
            HyprlandOpts::Keyboard => {
                let print_keyboard =
                    || {
                        if let Ok(devs) = Devices::get() {
                            crate::print(&Some(devs.keyboards.get(0).expect(
                                "It is assumed that Atleast one keyboard should be present",
                            )));
                        } else {
                            crate::print::<()>(&None);
                        };
                    };
                // while in the beginning prinnt the active keyboard
                print_keyboard();
                listener.add_keyboard_layout_change_handler(move |layout| {
                    eprintln!("KeyboardLayout changed: {layout:?}");
                    print_keyboard();
                })
            }
        };
        Self { listener }
    }
    pub fn listen(self) -> Result<()> {
        self.listener.start_listener()?;
        Ok(())
    }
}
