// #![allow(unused, dead_code)]
// #![deny(unused_must_use)]

use std::{collections::HashMap, time::Duration};

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
        .add_systems(Update, ui)
        .add_systems(FixedUpdate, tick)
        .run();
}

fn setup(mut commands: Commands) {
    let graph = Graph::default();
    let simulation_tick = Simulation {
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        ticks: 0,
    };
    commands.insert_resource(simulation_tick);
    commands.insert_resource(graph);
}

fn ui(mut contexts: EguiContexts, mut graph: ResMut<Graph>, mut simulation: ResMut<Simulation>) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            graph
                .state
                .show(&mut GraphViewer, &SnarlStyle::default(), "snarl", ui);
        });
        egui::Window::new("Controls").show(ctx, |ui| {
            // slider for the simulation speed
            ui.horizontal(|ui| {
                let mut paused = simulation.timer.paused();
                if ui.checkbox(&mut paused, "Paused").changed() {
                    if paused {
                        simulation.timer.pause();
                    } else {
                        simulation.timer.unpause();
                    }
                }

                if paused {
                    if ui.button("Step").clicked() {
                        graph.tick(&mut simulation, Duration::ZERO);
                    }
                } else {
                    let duration = simulation.timer.duration().as_secs_f32();
                    let mut hz = if duration == 0.0 { 0.0 } else { 1.0 / duration };
                    ui.add(egui::Slider::new(&mut hz, 0.0..=100.0).integer().text("Hz"));
                    simulation
                        .timer
                        .set_duration(Duration::from_secs_f32(if hz == 0.0 {
                            0.0
                        } else {
                            1.0 / hz
                        }));
                }
            });
        });
    }
}

fn tick(mut graph: ResMut<Graph>, time: Res<Time>, mut simulation: ResMut<Simulation>) {
    graph.tick(&mut simulation, time.delta());
}

#[derive(Default, Resource)]
struct Graph {
    state: Snarl<Node>,
}

struct GraphViewer;

#[derive(Resource)]
struct Simulation {
    timer: Timer,
    ticks: u64,
}

#[derive(strum::Display, strum::EnumIter)]
enum Node {
    Input(bool),
    Output(bool),
    Nand(bool),
    Clock(bool),
    Node(bool),
    Not(bool),
    And(bool),
    Or(bool),
    Xor(bool),
    Nor(bool),
    Xnor(bool),
}

impl Node {
    fn input_count(&self) -> usize {
        match self {
            Node::Input(_) => 0,
            Node::Output(_) => 1,
            Node::Nand(_) => 2,
            Node::Clock(_) => 0,
            Node::Node(_) => 1,
            Node::Not(_) => 1,
            Node::And(_) => 2,
            Node::Or(_) => 2,
            Node::Xor(_) => 2,
            Node::Nor(_) => 2,
            Node::Xnor(_) => 2,
        }
    }

    fn output_count(&self) -> usize {
        match self {
            Node::Input(_) => 1,
            Node::Output(_) => 0,
            Node::Nand(_) => 1,
            Node::Clock(_) => 1,
            Node::Node(_) => 1,
            Node::Not(_) => 1,
            Node::And(_) => 1,
            Node::Or(_) => 1,
            Node::Xor(_) => 1,
            Node::Nor(_) => 1,
            Node::Xnor(_) => 1,
        }
    }

    fn has_body(&self) -> bool {
        match self {
            Node::Input(_) => true,
            Node::Output(_) => true,
            Node::Nand(_) => false,
            Node::Clock(_) => false,
            Node::Node(_) => false,
            Node::Not(_) => false,
            Node::And(_) => false,
            Node::Or(_) => false,
            Node::Xor(_) => false,
            Node::Nor(_) => false,
            Node::Xnor(_) => false,
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
            Node::Nand(_) => unreachable!(),
            Node::Clock(_) => unreachable!(),
            Node::Node(_) => unreachable!(),
            Node::Not(_) => unreachable!(),
            Node::And(_) => unreachable!(),
            Node::Or(_) => unreachable!(),
            Node::Xor(_) => unreachable!(),
            Node::Nor(_) => unreachable!(),
            Node::Xnor(_) => unreachable!(),
        }
    }

    fn graph_menu_item(self, ui: &mut egui::Ui, snarl: &mut Snarl<Node>, pos: Pos2) {
        if ui.button(format!("Add {}", self)).clicked() {
            ui.close_menu();
            if !self.has_body() && self.input_count() <= 1 && self.output_count() <= 1 {
                snarl.insert_node_collapsed(pos, self);
            } else {
                snarl.insert_node(pos, self);
            }
        }
    }
}

impl Graph {
    #[recursive]
    fn eval(
        &mut self,
        in_pin: InPinId,
        ticks: u64,
        cache: &mut HashMap<(InPinId, u64), bool>,
    ) -> bool {
        if let Some(value) = cache.get(&(in_pin, ticks)) {
            return *value;
        }

        let result = self.state.in_pin(in_pin).remotes.iter().any(|remote| {
            let node = remote.node;
            match self.state.get_node(remote.node).unwrap() {
                Node::Input(value) => *value,
                Node::Nand(_) => {
                    let a = self.eval(InPinId { node, input: 0 }, ticks, cache);
                    let b = self.eval(InPinId { node, input: 1 }, ticks, cache);
                    !(a & b)
                }
                Node::Output(_) => unreachable!("Outputs should only be connected to inputs"),
                Node::Clock(_) => ticks % 2 == 0,
                Node::Node(_) => self.eval(InPinId { node, input: 0 }, ticks, cache),
                Node::Not(_) => !self.eval(InPinId { node, input: 0 }, ticks, cache),
                Node::And(_) => {
                    let a = self.eval(InPinId { node, input: 0 }, ticks, cache);
                    let b = self.eval(InPinId { node, input: 1 }, ticks, cache);
                    a & b
                }
                Node::Or(_) => {
                    let a = self.eval(InPinId { node, input: 0 }, ticks, cache);
                    let b = self.eval(InPinId { node, input: 1 }, ticks, cache);
                    a | b
                }
                Node::Xor(_) => {
                    let a = self.eval(InPinId { node, input: 0 }, ticks, cache);
                    let b = self.eval(InPinId { node, input: 1 }, ticks, cache);
                    a ^ b
                }
                Node::Nor(_) => {
                    let a = self.eval(InPinId { node, input: 0 }, ticks, cache);
                    let b = self.eval(InPinId { node, input: 1 }, ticks, cache);
                    !(a | b)
                }
                Node::Xnor(_) => {
                    let a = self.eval(InPinId { node, input: 0 }, ticks, cache);
                    let b = self.eval(InPinId { node, input: 1 }, ticks, cache);
                    !(a ^ b)
                }
            }
        });

        cache.insert((in_pin, ticks), result);
        result
    }

    fn tick(&mut self, simulation: &mut Simulation, dt: Duration) {
        simulation.timer.tick(dt);

        if dt == Duration::ZERO || simulation.timer.finished() {
            simulation.ticks += 1;

            let outputs = self
                .state
                .node_ids()
                .filter_map(|(id, node)| match node {
                    Node::Output(_) => Some(id),
                    _ => None,
                })
                .collect::<Vec<_>>();

            let mut cache = HashMap::new();

            for node in outputs {
                let result = self.eval(InPinId { node, input: 0 }, simulation.ticks, &mut cache);
                if let Node::Output(value) = self.state.get_node_mut(node).unwrap() {
                    *value = result;
                }
            }
        }
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
