// #![allow(unused, dead_code)]
// #![deny(unused_must_use)]

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Pos2},
    EguiContexts, EguiPlugin,
};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    InPinId, Snarl,
};
use recursive::recursive;
use strum::IntoEnumIterator;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (ui, tick))
        .run();
}

fn setup(mut commands: Commands) {
    let graph = Graph::default();
    let simulation_tick = SimulationTick {
        timer: Timer::from_seconds(0.0, TimerMode::Repeating),
    };
    commands.insert_resource(simulation_tick);
    commands.insert_resource(graph);
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

fn tick(mut graph: ResMut<Graph>, time: Res<Time>, mut simulation_tick: ResMut<SimulationTick>) {
    simulation_tick.timer.tick(time.delta());
    if simulation_tick.timer.finished() {
        let outputs = graph
            .state
            .node_ids()
            .filter_map(|(id, node)| match node {
                Node::Output(_) => Some(id),
                _ => None,
            })
            .collect::<Vec<_>>();

        for node in outputs {
            let result = graph.eval(InPinId { node, input: 0 });
            if let Node::Output(value) = graph.state.get_node_mut(node).unwrap() {
                *value = result;
            }
        }
    }
}

#[derive(Default, Resource)]
struct Graph {
    state: Snarl<Node>,
}

struct GraphViewer;

#[derive(Resource)]
struct SimulationTick {
    timer: Timer,
}

#[derive(strum::Display, strum::EnumIter)]
enum Node {
    Input(bool),
    Output(bool),
    Nand(bool),
}

impl Node {
    fn input_count(&self) -> usize {
        match self {
            Node::Input(_) => 0,
            Node::Output(_) => 1,
            Node::Nand(_) => 2,
        }
    }

    fn output_count(&self) -> usize {
        match self {
            Node::Input(_) => 1,
            Node::Output(_) => 0,
            Node::Nand(_) => 1,
        }
    }

    fn has_body(&self) -> bool {
        match self {
            Node::Input(_) => true,
            Node::Output(_) => true,
            Node::Nand(_) => false,
        }
    }

    fn show_body(&mut self, ui: &mut egui::Ui) {
        match self {
            Node::Input(value) => {
                ui.checkbox(value, "");
            }
            Node::Output(value) => {
                ui.checkbox(value, "");
            }
            Node::Nand(_) => unreachable!("Nand nodes should not have bodies"),
        }
    }

    fn graph_menu_item(self, ui: &mut egui::Ui, snarl: &mut Snarl<Node>, pos: Pos2) {
        if ui.button(format!("Add {}", self)).clicked() {
            snarl.insert_node(pos, self);
            ui.close_menu();
        }
    }
}

impl Graph {
    #[recursive]
    fn eval(&mut self, in_pin: InPinId) -> bool {
        self.state.in_pin(in_pin).remotes.iter().any(|remote| {
            let node = remote.node;
            match self.state.get_node(remote.node).unwrap() {
                Node::Input(value) => *value,
                Node::Nand(_) => {
                    let a = self.eval(InPinId { node, input: 0 });
                    let b = self.eval(InPinId { node, input: 1 });
                    !(a & b)
                }
                Node::Output(_) => unreachable!("Outputs should only be connected to inputs"),
            }
        })
    }
}

impl SnarlViewer<Node> for GraphViewer {
    fn title(&mut self, node: &Node) -> String {
        node.to_string()
    }

    fn outputs(&mut self, node: &Node) -> usize {
        node.output_count()
    }

    fn inputs(&mut self, node: &Node) -> usize {
        node.input_count()
    }

    fn show_input(
        &mut self,
        _pin: &egui_snarl::InPin,
        _ui: &mut egui::Ui,
        _scale: f32,
        _snarl: &mut Snarl<Node>,
    ) -> egui_snarl::ui::PinInfo {
        PinInfo::default()
    }

    fn show_output(
        &mut self,
        _pin: &egui_snarl::OutPin,
        _ui: &mut egui::Ui,
        _scale: f32,
        _snarl: &mut Snarl<Node>,
    ) -> egui_snarl::ui::PinInfo {
        PinInfo::default()
    }

    fn has_graph_menu(&mut self, _pos: Pos2, _snarl: &mut Snarl<Node>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: Pos2,
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        Node::iter().for_each(|value| value.graph_menu_item(ui, snarl, pos));
    }

    fn has_body(&mut self, node: &Node) -> bool {
        node.has_body()
    }

    fn show_body(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[egui_snarl::InPin],
        _outputs: &[egui_snarl::OutPin],
        ui: &mut egui::Ui,
        _scale: f32,
        snarl: &mut Snarl<Node>,
    ) {
        if let Some(node) = snarl.get_node_mut(node) {
            node.show_body(ui);
        }
    }
}
