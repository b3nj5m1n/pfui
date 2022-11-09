use serde::Serialize;
use sway::{Connection, Event, EventType, Fallible};

use crate::Module;

#[derive(Debug, Serialize)]
struct Data {
    kbd_layout: Option<String>,
    workspaces: Vec<Workspace>,
    binding_modes: Vec<BindingMode>,
}

#[derive(Debug, Serialize)]
struct Workspace {
    id: i64,
    name: String,
    visible: bool,
    focused: bool,
    urgent: bool,
    output: String,
}

#[derive(Debug, Serialize)]
struct BindingMode {
    name: String,
    active: bool,
}

impl Data {
    fn get(conn: &mut Connection) -> Result<Self, Box<dyn std::error::Error>> {
        let workspaces: Vec<Workspace> = conn
            .get_workspaces()?
            .into_iter()
            .map(|workspace| Workspace {
                id: workspace.id,
                name: workspace.name,
                visible: workspace.visible,
                focused: workspace.focused,
                urgent: workspace.urgent,
                output: workspace.output,
            })
            .collect();
        let inputs = conn.get_inputs()?;
        let mut layout = None;
        for l in inputs {
            if let Some(layout_name) = l.xkb_active_layout_name {
                layout = Some(layout_name);
            }
        }
        let current_binding_mode = conn.get_binding_state()?;
        let binding_modes = conn
            .get_binding_modes()?
            .into_iter()
            .map(|mode| BindingMode {
                name: mode.clone(),
                active: mode == current_binding_mode,
            })
            .collect();
        Ok(Self {
            kbd_layout: layout,
            workspaces,
            binding_modes,
        })
    }
}

pub struct Sway {}

impl Module for Sway {
    type Connection = Connection;

    fn connect(&mut self, timeout: u64) -> Result<Self::Connection, Box<dyn std::error::Error>> {
        return Ok(Connection::new()?);
    }

    fn output(&self, conn: &mut Self::Connection) {
        let data = Data::get(conn);
        match data {
            Ok(data) => crate::print(&Some(&data)),
            Err(_) => crate::print::<Data>(&None)
        }
    }

    fn start(&mut self, timeout: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.connect(timeout)?;
        for event in Connection::new()?.subscribe([EventType::Input, EventType::Workspace])? {
            self.output(&mut conn);
        }
        Ok(())
    }
}
