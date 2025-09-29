//! Renderer module for 3D scene visualization
//!
//! Provides Bevy-based rendering system with scene graph integration.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::graph::SceneGraph;

/// Renderer plugin for Bevy
pub struct RendererPlugin;

/// Scene entity mapping (vertex ID -> Bevy entity)
#[derive(Resource)]
pub struct SceneEntityMap {
    /// Mapping from scene graph vertex IDs to Bevy entities
    pub entity_map: std::collections::HashMap<String, Entity>,
    /// Mapping from Bevy entities to vertex IDs
    pub reverse_map: std::collections::HashMap<Entity, String>,
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SceneEntityMap {
            entity_map: std::collections::HashMap::new(),
            reverse_map: std::collections::HashMap::new(),
        })
        .add_systems(Update, sync_scene_with_renderer);
    }
}

impl Default for RendererPlugin {
    fn default() -> Self {
        Self
    }
}

/// Sync scene graph with Bevy renderer
fn sync_scene_with_renderer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut entity_map: ResMut<SceneEntityMap>,
    scene_graph: Res<SceneGraph>,
) {
    // Sync vertices (scene objects)
    for node_index in scene_graph.graph.node_indices() {
        if let Some(vertex) = scene_graph.graph.node_weight(node_index) {
            let vertex_id = &vertex.id;

            if let Some(&entity) = entity_map.entity_map.get(vertex_id) {
                // Update existing entity
                update_entity_from_vertex(&mut commands, entity, vertex);
            } else {
                // Create new entity
                let entity = create_entity_from_vertex(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    vertex,
                );
                entity_map.entity_map.insert(vertex_id.clone(), entity);
                entity_map.reverse_map.insert(entity, vertex_id.clone());
            }
        }
    }

    // Remove entities for deleted vertices
    let existing_vertex_ids: std::collections::HashSet<_> = scene_graph.graph.node_weights()
        .map(|v| v.id.clone())
        .collect();

    let mut to_remove = Vec::new();
    for (vertex_id, &entity) in &entity_map.entity_map {
        if !existing_vertex_ids.contains(vertex_id) {
            commands.entity(entity).despawn_recursive();
            to_remove.push((vertex_id.clone(), entity));
        }
    }

    for (vertex_id, entity) in to_remove {
        entity_map.entity_map.remove(&vertex_id);
        entity_map.reverse_map.remove(&entity);
    }
}

/// Create Bevy entity from scene vertex
fn create_entity_from_vertex(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    vertex: &crate::graph::vertex::Vertex,
) -> Entity {
    let mut entity_commands = commands.spawn_empty();

    // Set transform
    entity_commands.insert(Transform {
        translation: Vec3::new(
            vertex.transform.position.x,
            vertex.transform.position.y,
            vertex.transform.position.z,
        ),
        rotation: Quat::from_xyzw(
            vertex.transform.rotation.i,
            vertex.transform.rotation.j,
            vertex.transform.rotation.k,
            vertex.transform.rotation.w,
        ),
        scale: Vec3::new(
            vertex.transform.scale.x,
            vertex.transform.scale.y,
            vertex.transform.scale.z,
        ),
    });

    // Add geometry based on vertex properties
    let mesh = create_mesh_from_vertex(vertex);
    let material = create_material_from_vertex(vertex);

    entity_commands
        .insert(meshes.add(mesh))
        .insert(materials.add(material));

    // Add physics body if physics state exists
    if let Some(physics_state) = &vertex.physics_state {
        match physics_state.is_kinematic {
            false => {
                entity_commands.insert(RigidBody::Dynamic);
            }
            true => {
                entity_commands.insert(RigidBody::KinematicPositionBased);
            }
        }

        // Add collider based on vertex geometry
        let collider = create_collider_from_vertex(vertex);
        entity_commands.insert(collider);

        // Set mass if specified
        if physics_state.mass > 0.0 {
            entity_commands.insert(AdditionalMassProperties::Mass(physics_state.mass));
        }

        // Set initial velocity
        entity_commands.insert(Velocity {
            linvel: Vec3::new(physics_state.velocity.x, physics_state.velocity.y, physics_state.velocity.z),
            angvel: Vec3::new(physics_state.angular_velocity.x, physics_state.angular_velocity.y, physics_state.angular_velocity.z),
        });
    }

    // Add lighting for light vertices
    match vertex.node_type {
        crate::dsl::scene::NodeType::Light => {
            // Add point light (simplified - in practice you'd parse light properties)
            entity_commands.insert(PointLightBundle {
                point_light: PointLight {
                    intensity: 1500.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::from_translation(vertex.transform.position.into()),
                ..default()
            });
        }
        _ => {}
    }

    entity_commands.id()
}

/// Update existing Bevy entity from vertex
fn update_entity_from_vertex(
    commands: &mut Commands,
    entity: Entity,
    vertex: &crate::graph::vertex::Vertex,
) {
    // Update transform
    let transform = Transform {
        translation: Vec3::new(
            vertex.transform.position.x,
            vertex.transform.position.y,
            vertex.transform.position.z,
        ),
        rotation: Quat::from_xyzw(
            vertex.transform.rotation.i,
            vertex.transform.rotation.j,
            vertex.transform.rotation.k,
            vertex.transform.rotation.w,
        ),
        scale: Vec3::new(
            vertex.transform.scale.x,
            vertex.transform.scale.y,
            vertex.transform.scale.z,
        ),
    };

    commands.entity(entity).insert(transform);
}

/// Create mesh from vertex properties
fn create_mesh_from_vertex(vertex: &crate::graph::vertex::Vertex) -> Mesh {
    // Parse geometry from vertex properties (simplified)
    // In practice, you'd deserialize the JSON properties

    // Default to cube
    Cuboid::new(1.0, 1.0, 1.0).into()
}

/// Create collider from vertex properties
fn create_collider_from_vertex(vertex: &crate::graph::vertex::Vertex) -> Collider {
    // Parse collision shape from vertex properties (simplified)
    // In practice, you'd deserialize the JSON properties

    // Default to cuboid collider matching the mesh
    Collider::cuboid(0.5, 0.5, 0.5)
}

/// Create material from vertex properties
fn create_material_from_vertex(vertex: &crate::graph::vertex::Vertex) -> StandardMaterial {
    // Parse material from vertex properties (simplified)
    // In practice, you'd deserialize the JSON properties

    // Default material
    StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        metallic: 0.0,
        perceptual_roughness: 0.5,
        ..default()
    }
}

/// Camera controller for scene navigation
#[derive(Component)]
pub struct SceneCamera {
    /// Movement speed
    pub speed: f32,
    /// Rotation sensitivity
    pub sensitivity: f32,
}

impl Default for SceneCamera {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.1,
        }
    }
}

/// Camera controller system
pub fn camera_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &SceneCamera)>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    for (mut transform, camera) in query.iter_mut() {
        // Keyboard movement
        let mut velocity = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            velocity += *transform.forward();
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            velocity += *transform.back();
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            velocity += *transform.left();
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            velocity += *transform.right();
        }
        if keys.pressed(KeyCode::KeyQ) {
            velocity += Vec3::Y;
        }
        if keys.pressed(KeyCode::KeyE) {
            velocity += Vec3::NEG_Y;
        }

        if velocity != Vec3::ZERO {
            velocity = velocity.normalize() * camera.speed * time.delta_seconds();
            transform.translation += velocity;
        }

        // Mouse look (only when right mouse button is pressed)
        if mouse_buttons.pressed(MouseButton::Right) {
            for motion in mouse_motion.read() {
                let delta = motion.delta * camera.sensitivity * time.delta_seconds();

                // Rotate around Y axis (yaw)
                transform.rotate_y(-delta.x);

                // Rotate around local X axis (pitch), with limits
                let pitch_delta = -delta.y;
                let current_pitch = transform.rotation.to_euler(EulerRot::YXZ).0;
                let new_pitch = (current_pitch + pitch_delta).clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);
                let pitch_correction = new_pitch - current_pitch;

                transform.rotate_local_x(pitch_correction);
            }
        }
    }
}

/// Add camera controller to Bevy app
pub fn add_camera_controller(app: &mut App) {
    app.add_systems(Update, camera_controller);
}
