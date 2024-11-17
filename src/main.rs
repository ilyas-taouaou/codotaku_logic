#![allow(unused, dead_code)]
#![deny(unused_must_use)]

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Pos2},
    EguiContexts, EguiPlugin,
};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    InPinId, NodeId, Snarl,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (ui, tick))
        .run();
}

enum Node {
    Input(bool),
    Output(bool),
    Nand(bool),
}

#[derive(Default, Resource)]
struct Graph {
    state: Snarl<Node>,
}

fn setup(mut commands: Commands) {
    let mut graph = Graph::default();
    let simulation_tick = SimulationTick {
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    };
    commands.insert_resource(simulation_tick);
    commands.insert_resource(graph);
}

struct GraphViewer;

impl SnarlViewer<Node> for GraphViewer {
    fn title(&mut self, node: &Node) -> String {
        match node {
            Node::Input(_) => "Input".to_string(),
            Node::Output(_) => "Output".to_string(),
            Node::Nand(_) => "Nand".to_string(),
        }
    }

    fn outputs(&mut self, node: &Node) -> usize {
        match node {
            Node::Input(_) => 1,
            Node::Output(_) => 0,
            Node::Nand(_) => 1,
        }
    }

    fn inputs(&mut self, node: &Node) -> usize {
        match node {
            Node::Input(_) => 0,
            Node::Output(_) => 1,
            Node::Nand(_) => 2,
        }
    }

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<Node>,
    ) -> egui_snarl::ui::PinInfo {
        PinInfo::default()
    }

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<Node>,
    ) -> egui_snarl::ui::PinInfo {
        PinInfo::default()
    }

    fn has_graph_menu(&mut self, pos: Pos2, snarl: &mut Snarl<Node>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: Pos2,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        if ui.button("Add Input").clicked() {
            snarl.insert_node(pos, Node::Input(false));
            ui.close_menu();
        }
        if ui.button("Add Output").clicked() {
            snarl.insert_node(pos, Node::Output(false));
            ui.close_menu();
        }
        if ui.button("Add Nand").clicked() {
            snarl.insert_node(pos, Node::Nand(false));
            ui.close_menu();
        }
    }

    fn has_body(&mut self, node: &Node) -> bool {
        match node {
            Node::Input(_) => true,
            Node::Output(_) => true,
            Node::Nand(_) => false,
        }
    }

    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        inputs: &[egui_snarl::InPin],
        outputs: &[egui_snarl::OutPin],
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        if let Some(node) = snarl.get_node_mut(node) {
            match node {
                Node::Input(value) => {
                    ui.checkbox(value, "");
                }
                Node::Output(value) => {
                    ui.checkbox(value, "");
                }
                Node::Nand(_) => unreachable!("Nand nodes should not have bodies"),
            }
        }
    }
}

fn ui(mut contexts: EguiContexts, mut graph: ResMut<Graph>) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            graph
                .state
                .show(&mut GraphViewer, &SnarlStyle::default(), "snarl", ui);
        });
    }
}

#[derive(Resource)]
struct SimulationTick {
    timer: Timer,
}

fn tick(mut graph: ResMut<Graph>, time: Res<Time>, mut tick: ResMut<SimulationTick>) {
    tick.timer.tick(time.delta());
    if tick.timer.finished() {
        let mut outputs = vec![];
        for (id, node) in graph.state.node_ids() {
            match node {
                Node::Output(_) => outputs.push(id),
                _ => {}
            }
        }
        for node in outputs {
            let result = f(&mut graph, InPinId { node, input: 0 });
            let mut node = graph.state.get_node_mut(node).unwrap();
            match node {
                Node::Output(value) => {
                    *value = result;
                }
                _ => unreachable!("Outputs should only be connected to inputs"),
            }
        }
    }
}

fn f(graph: &mut Graph, in_pin: InPinId) -> bool {
    let pin = graph.state.in_pin(in_pin);
    let mut result = false;
    for remote in pin.remotes {
        let remote_node = graph.state.get_node(remote.node).unwrap();
        match remote_node {
            Node::Input(value) => {
                result |= *value;
            }
            Node::Nand(output) => {
                let a = f(
                    graph,
                    InPinId {
                        node: remote.node,
                        input: 0,
                    },
                );
                let b = f(
                    graph,
                    InPinId {
                        node: remote.node,
                        input: 1,
                    },
                );
                result |= !(a & b);
            }
            _ => unreachable!("Outputs should only be connected to inputs"),
        }
    }
    result
}
