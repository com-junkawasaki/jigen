//! # Jigen - Declarative 3D Scene DSL with Physics
//!
//! A Bevy-based framework for declarative 3D scene description using JSON DSL
//! with integrated physics simulation. Supports v(vertex), e(edge), i(incident) graph model.
//!
//! ## Features
//!
//! - **Declarative DSL**: JSON-based scene description similar to A-Frame/Three.js React
//! - **Physics Simulation**: Integrated force dynamics and collision detection
//! - **Graph Model**: v-e-i (vertex-edge-incident) graph structure for scene topology
//! - **WASM Support**: Run in browsers with WebAssembly
//! - **Bevy Integration**: Leverages Bevy's ECS and rendering pipeline
//!
//! ## Architecture
//!
//! The system follows a process network graph model where:
//! - **v**: Vertices represent scene objects/nodes
//! - **e**: Edges represent relationships/connections
//! - **i**: Incidents represent interactions/physics forces

pub mod dsl;
pub mod graph;
// pub mod physics; // Temporarily disabled
pub mod renderer;
pub mod wasm;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Main application state for Jigen
#[derive(Resource, Debug)]
pub struct JigenApp {
    pub scene_graph: graph::SceneGraph,
}

/// Configuration for scene initialization
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct SceneConfig {
    pub title: String,
    pub dimensions: (u32, u32),
    pub physics_enabled: bool,
    pub gravity: [f32; 3],
    pub background_color: [f32; 4],
}

/// Initialize the Jigen application with Bevy
pub fn init_jigen(config: SceneConfig) -> App {
    let mut app = App::new();

    // Add Bevy plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: config.title,
            resolution: (config.dimensions.0 as f32, config.dimensions.1 as f32).into(),
            ..default()
        }),
        ..default()
    }));

    // Add physics plugin if enabled (temporarily disabled)
    // if config.physics_enabled {
    //     app.add_plugins(physics::PhysicsPlugin);
    // }

    // Add renderer
    app.add_plugins(renderer::RendererPlugin);

    // Initialize resources
    app.insert_resource(JigenApp {
        scene_graph: graph::SceneGraph::new(),
    });

    // Add systems
    app.add_systems(Startup, setup_scene);
    app.add_systems(Update, update_scene);

    app
}

/// Setup initial scene
fn setup_scene(
    mut commands: Commands,
    config: Res<SceneConfig>,
) {
    // Set background color
    commands.insert_resource(ClearColor(Color::srgba(
        config.background_color[0],
        config.background_color[1],
        config.background_color[2],
        config.background_color[3],
    )));

    // Initialize camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Setup lighting
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

/// Update scene graph and physics
fn update_scene(
    mut jigen_app: ResMut<JigenApp>,
    time: Res<Time>,
) {
    // Update scene graph topology
    jigen_app.scene_graph.update_topology();
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WASM entry point
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let config = SceneConfig {
        title: "Jigen - Declarative 3D Scene".to_string(),
        dimensions: (800, 600),
        physics_enabled: true,
        gravity: [0.0, -9.81, 0.0],
        background_color: [0.1, 0.1, 0.1, 1.0],
    };

    init_jigen(config).run();
}

/// Native entry point
#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let config = SceneConfig {
        title: "Jigen - Declarative 3D Scene".to_string(),
        dimensions: (1280, 720),
        physics_enabled: true,
        gravity: [0.0, -9.81, 0.0],
        background_color: [0.1, 0.1, 0.1, 1.0],
    };

    init_jigen(config).run();
}
