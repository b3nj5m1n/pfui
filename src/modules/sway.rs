use std::{thread::sleep, time::Duration};

use serde::Serialize;
use sway::{Connection,EventType, NodeType};
use anyhow::Result;

use crate::Module;

#[derive(Debug, Serialize)]
struct Data {
    kbd_layout: Option<String>,
    window_title: Option<String>,
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

fn flatten_nodes(node: &mut sway::Node) -> Vec<sway::Node> {
    let mut result: Vec<sway::Node> = Vec::new();
    for child in &mut node.nodes {
        result.append(&mut flatten_nodes(child));
    }
    result.push(node.to_owned());
    result
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
        // TODO: Make this more robust
        // Currently, we go through all nodes that exist and find a focused on, this should work
        // reliably on a single monitor setup, but I believe it's going to fail when multiple
        // monitors are involved.
        // When we call get_tree(), the data is structured as follows:
        // We get the root node, this should be the only node at this level
        // The root node contains monitor nodes
        // The monitor nodes contain workspace nodes, corresponding to the workspaces on that
        // monitor
        // The monitor nodes contain Con nodes, which are what we're after.
        // To make this work for multi-monitor, we'd have to extract the focused node for each
        // monitor and then output the active window name for each monitor seperately.
        let mut root_node = conn.get_tree().unwrap();
        let window_title = if let Some(current_window) = flatten_nodes(&mut root_node)
            .into_iter()
            .filter(|node| node.node_type == NodeType::Con && node.focused)
            .collect::<Vec<sway::Node>>()
            .first()
        {
            current_window.name.clone()
        } else {
            None
        };

        Ok(Self {
            kbd_layout: layout,
            window_title,
            workspaces,
            binding_modes,
        })
    }
}

pub struct Sway {}

impl Module for Sway {
    type Connection = Connection;

    fn connect(&mut self, timeout: u64) -> Result<Self::Connection> {
        let mut conn = Connection::new();
        while let Err(..) = conn {
            conn = Connection::new();
            crate::print(&None::<Data>);
            sleep(Duration::new(timeout, 0));
        }
        Ok(conn?)
    }

    fn output(&self, conn: &mut Self::Connection) {
        let data = Data::get(conn);
        match data {
            Ok(data) => crate::print(&Some(&data)),
            Err(_) => crate::print::<Data>(&None),
        }
    }

    fn start(&mut self, timeout: u64) -> Result<()> {
        let mut conn = self.connect(timeout)?;
        self.output(&mut conn);
        for _ in Connection::new()?.subscribe([
            EventType::Input,
            EventType::Workspace,
            EventType::Window,
        ])? {
            self.output(&mut conn);
        }
        Ok(())
    }
}
