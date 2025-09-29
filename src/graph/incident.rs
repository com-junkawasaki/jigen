//! Incident definitions for graph events and interactions
//!
//! Incidents represent events, interactions, and state changes in the scene graph.

use super::{vertex::Vertex, edge::Edge};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Incident representing an event or interaction in the graph
#[derive(Debug, Clone)]
pub struct Incident {
    /// Unique incident ID
    pub id: String,

    /// Incident type
    pub incident_type: IncidentType,

    /// Involved vertices
    pub vertices: Vec<String>,

    /// Involved edges
    pub edges: Vec<String>,

    /// Incident properties
    pub properties: IncidentProperties,

    /// Timestamp when incident occurred
    pub timestamp: Instant,

    /// Incident duration (for continuous events)
    pub duration: Option<Duration>,

    /// Incident priority/severity
    pub priority: IncidentPriority,

    /// Processing state
    pub state: IncidentState,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of incidents that can occur
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncidentType {
    /// Collision between objects
    Collision,
    /// Force interaction
    ForceInteraction,
    /// Constraint violation
    ConstraintViolation,
    /// State change (position, rotation, etc.)
    StateChange,
    /// User input event
    UserInput,
    /// Physics simulation step
    PhysicsStep,
    /// Rendering event
    RenderEvent,
    /// Custom incident type
    Custom(String),
}

/// Properties specific to different incident types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentProperties {
    /// Collision incident properties
    Collision {
        /// Collision point
        contact_point: [f32; 3],
        /// Collision normal
        normal: [f32; 3],
        /// Impulse magnitude
        impulse: f32,
        /// Relative velocity
        relative_velocity: [f32; 3],
    },

    /// Force interaction properties
    ForceInteraction {
        /// Force vector
        force: [f32; 3],
        /// Application point
        point: [f32; 3],
        /// Force type
        force_type: String,
    },

    /// Constraint violation properties
    ConstraintViolation {
        /// Constraint type
        constraint_type: String,
        /// Violation amount
        violation_amount: f32,
        /// Required correction
        correction: [f32; 3],
    },

    /// State change properties
    StateChange {
        /// What changed
        changed_property: String,
        /// Old value
        old_value: serde_json::Value,
        /// New value
        new_value: serde_json::Value,
    },

    /// User input properties
    UserInput {
        /// Input type (mouse, keyboard, touch, etc.)
        input_type: String,
        /// Input action
        action: String,
        /// Input value (coordinates, key code, etc.)
        value: serde_json::Value,
    },

    /// Physics step properties
    PhysicsStep {
        /// Simulation time step
        delta_time: f32,
        /// Number of iterations
        iterations: u32,
        /// Simulation stability metric
        stability: f32,
    },

    /// Render event properties
    RenderEvent {
        /// Render pass type
        pass_type: String,
        /// Objects rendered
        objects_rendered: u32,
        /// Render time
        render_time_ms: f32,
    },

    /// Custom properties
    Custom {
        /// Custom data
        data: serde_json::Value,
    },
}

/// Incident priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IncidentPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Processing state of an incident
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentState {
    /// Newly created, not yet processed
    Pending,
    /// Currently being processed
    Processing,
    /// Successfully processed
    Completed,
    /// Processing failed
    Failed,
    /// Incident was cancelled
    Cancelled,
}

/// Incident handler trait for processing incidents
pub trait IncidentHandler {
    /// Handle a collision incident
    fn handle_collision(&mut self, incident: &Incident, vertices: &[&Vertex], edges: &[&Edge]) -> IncidentResult;

    /// Handle a force interaction incident
    fn handle_force_interaction(&mut self, incident: &Incident, vertices: &[&Vertex], edges: &[&Edge]) -> IncidentResult;

    /// Handle a constraint violation
    fn handle_constraint_violation(&mut self, incident: &Incident, vertices: &[&Vertex], edges: &[&Edge]) -> IncidentResult;

    /// Handle a state change incident
    fn handle_state_change(&mut self, incident: &Incident, vertices: &[&Vertex], edges: &[&Edge]) -> IncidentResult;

    /// Handle a user input incident
    fn handle_user_input(&mut self, incident: &Incident, vertices: &[&Vertex], edges: &[&Edge]) -> IncidentResult;

    /// Handle any custom incident type
    fn handle_custom(&mut self, incident: &Incident, vertices: &[&Vertex], edges: &[&Edge]) -> IncidentResult;
}

/// Result of incident processing
#[derive(Debug, Clone)]
pub enum IncidentResult {
    /// Processing completed successfully
    Success,
    /// Processing failed with error message
    Failure(String),
    /// Incident should be deferred for later processing
    Defer,
    /// Incident should be cancelled
    Cancel,
    /// Incident generated new incidents
    Cascade(Vec<Incident>),
}

impl Incident {
    /// Create a new collision incident
    pub fn collision(
        id: String,
        vertex_ids: Vec<String>,
        contact_point: [f32; 3],
        normal: [f32; 3],
        impulse: f32,
        relative_velocity: [f32; 3],
    ) -> Self {
        Self {
            id,
            incident_type: IncidentType::Collision,
            vertices: vertex_ids,
            edges: Vec::new(),
            properties: IncidentProperties::Collision {
                contact_point,
                normal,
                impulse,
                relative_velocity,
            },
            timestamp: Instant::now(),
            duration: None,
            priority: IncidentPriority::High,
            state: IncidentState::Pending,
            metadata: HashMap::new(),
        }
    }

    /// Create a force interaction incident
    pub fn force_interaction(
        id: String,
        vertex_ids: Vec<String>,
        edge_ids: Vec<String>,
        force: [f32; 3],
        point: [f32; 3],
        force_type: String,
    ) -> Self {
        Self {
            id,
            incident_type: IncidentType::ForceInteraction,
            vertices: vertex_ids,
            edges: edge_ids,
            properties: IncidentProperties::ForceInteraction {
                force,
                point,
                force_type,
            },
            timestamp: Instant::now(),
            duration: None,
            priority: IncidentPriority::Medium,
            state: IncidentState::Pending,
            metadata: HashMap::new(),
        }
    }

    /// Create a state change incident
    pub fn state_change(
        id: String,
        vertex_ids: Vec<String>,
        changed_property: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    ) -> Self {
        Self {
            id,
            incident_type: IncidentType::StateChange,
            vertices: vertex_ids,
            edges: Vec::new(),
            properties: IncidentProperties::StateChange {
                changed_property,
                old_value,
                new_value,
            },
            timestamp: Instant::now(),
            duration: None,
            priority: IncidentPriority::Low,
            state: IncidentState::Pending,
            metadata: HashMap::new(),
        }
    }

    /// Create a user input incident
    pub fn user_input(
        id: String,
        input_type: String,
        action: String,
        value: serde_json::Value,
    ) -> Self {
        Self {
            id,
            incident_type: IncidentType::UserInput,
            vertices: Vec::new(),
            edges: Vec::new(),
            properties: IncidentProperties::UserInput {
                input_type,
                action,
                value,
            },
            timestamp: Instant::now(),
            duration: None,
            priority: IncidentPriority::High,
            state: IncidentState::Pending,
            metadata: HashMap::new(),
        }
    }

    /// Create a physics step incident
    pub fn physics_step(
        id: String,
        delta_time: f32,
        iterations: u32,
        stability: f32,
    ) -> Self {
        Self {
            id,
            incident_type: IncidentType::PhysicsStep,
            vertices: Vec::new(),
            edges: Vec::new(),
            properties: IncidentProperties::PhysicsStep {
                delta_time,
                iterations,
                stability,
            },
            timestamp: Instant::now(),
            duration: Some(Duration::from_secs_f32(delta_time)),
            priority: IncidentPriority::Medium,
            state: IncidentState::Pending,
            metadata: HashMap::new(),
        }
    }

    /// Mark incident as processing
    pub fn start_processing(&mut self) {
        self.state = IncidentState::Processing;
    }

    /// Mark incident as completed
    pub fn complete(&mut self) {
        self.state = IncidentState::Completed;
    }

    /// Mark incident as failed
    pub fn fail(&mut self, error: String) {
        self.metadata.insert("error".to_string(), serde_json::Value::String(error));
        self.state = IncidentState::Failed;
    }

    /// Mark incident as cancelled
    pub fn cancel(&mut self) {
        self.state = IncidentState::Cancelled;
    }

    /// Check if incident is active (pending or processing)
    pub fn is_active(&self) -> bool {
        matches!(self.state, IncidentState::Pending | IncidentState::Processing)
    }

    /// Check if incident is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.state, IncidentState::Completed)
    }

    /// Get incident age
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Add metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Default incident handler implementation
pub struct DefaultIncidentHandler;

impl IncidentHandler for DefaultIncidentHandler {
    fn handle_collision(&mut self, incident: &Incident, _vertices: &[&Vertex], _edges: &[&Edge]) -> IncidentResult {
        if let IncidentProperties::Collision { impulse, .. } = &incident.properties {
            // Basic collision response - just log for now
            tracing::info!("Collision incident processed with impulse: {}", impulse);
            IncidentResult::Success
        } else {
            IncidentResult::Failure("Invalid collision properties".to_string())
        }
    }

    fn handle_force_interaction(&mut self, incident: &Incident, _vertices: &[&Vertex], _edges: &[&Edge]) -> IncidentResult {
        if let IncidentProperties::ForceInteraction { force, .. } = &incident.properties {
            // Basic force interaction response
            tracing::debug!("Force interaction processed: {:?}", force);
            IncidentResult::Success
        } else {
            IncidentResult::Failure("Invalid force properties".to_string())
        }
    }

    fn handle_constraint_violation(&mut self, incident: &Incident, _vertices: &[&Vertex], _edges: &[&Edge]) -> IncidentResult {
        if let IncidentProperties::ConstraintViolation { correction, .. } = &incident.properties {
            // Constraint violation response
            tracing::warn!("Constraint violation detected, correction: {:?}", correction);
            IncidentResult::Success
        } else {
            IncidentResult::Failure("Invalid constraint properties".to_string())
        }
    }

    fn handle_state_change(&mut self, incident: &Incident, _vertices: &[&Vertex], _edges: &[&Edge]) -> IncidentResult {
        if let IncidentProperties::StateChange { changed_property, .. } = &incident.properties {
            // State change logging
            tracing::debug!("State change: {}", changed_property);
            IncidentResult::Success
        } else {
            IncidentResult::Failure("Invalid state change properties".to_string())
        }
    }

    fn handle_user_input(&mut self, incident: &Incident, _vertices: &[&Vertex], _edges: &[&Edge]) -> IncidentResult {
        if let IncidentProperties::UserInput { action, .. } = &incident.properties {
            // User input processing
            tracing::info!("User input processed: {}", action);
            IncidentResult::Success
        } else {
            IncidentResult::Failure("Invalid user input properties".to_string())
        }
    }

    fn handle_custom(&mut self, incident: &Incident, _vertices: &[&Vertex], _edges: &[&Edge]) -> IncidentResult {
        // Generic custom incident handler
        tracing::debug!("Custom incident processed: {}", incident.id);
        IncidentResult::Success
    }
}

/// Incident manager for coordinating incident processing
#[derive(Debug)]
pub struct IncidentManager {
    /// Active incidents
    incidents: Vec<Incident>,
    /// Incident handler
    handler: Box<dyn IncidentHandler>,
    /// Maximum number of active incidents
    max_incidents: usize,
}

impl IncidentManager {
    /// Create a new incident manager
    pub fn new(max_incidents: usize) -> Self {
        Self {
            incidents: Vec::new(),
            handler: Box::new(DefaultIncidentHandler),
            max_incidents,
        }
    }

    /// Add an incident to be processed
    pub fn add_incident(&mut self, incident: Incident) {
        if self.incidents.len() < self.max_incidents {
            self.incidents.push(incident);
        } else {
            tracing::warn!("Incident queue full, dropping incident: {}", incident.id);
        }
    }

    /// Process pending incidents
    pub fn process_incidents(&mut self, vertices: &[&Vertex], edges: &[&Edge]) {
        let mut completed_indices = Vec::new();

        for (index, incident) in self.incidents.iter_mut().enumerate() {
            if !incident.is_active() {
                continue;
            }

            incident.start_processing();

            let result = match incident.incident_type {
                IncidentType::Collision => self.handler.handle_collision(incident, vertices, edges),
                IncidentType::ForceInteraction => self.handler.handle_force_interaction(incident, vertices, edges),
                IncidentType::ConstraintViolation => self.handler.handle_constraint_violation(incident, vertices, edges),
                IncidentType::StateChange => self.handler.handle_state_change(incident, vertices, edges),
                IncidentType::UserInput => self.handler.handle_user_input(incident, vertices, edges),
                IncidentType::PhysicsStep | IncidentType::RenderEvent => {
                    // These are handled by specific systems
                    IncidentResult::Success
                }
                IncidentType::Custom(_) => self.handler.handle_custom(incident, vertices, edges),
            };

            match result {
                IncidentResult::Success => {
                    incident.complete();
                    completed_indices.push(index);
                }
                IncidentResult::Failure(error) => {
                    incident.fail(error);
                    completed_indices.push(index);
                }
                IncidentResult::Defer => {
                    // Leave as processing for next frame
                    incident.state = IncidentState::Pending;
                }
                IncidentResult::Cancel => {
                    incident.cancel();
                    completed_indices.push(index);
                }
                IncidentResult::Cascade(new_incidents) => {
                    incident.complete();
                    completed_indices.push(index);
                    // Add new incidents
                    for new_incident in new_incidents {
                        self.add_incident(new_incident);
                    }
                }
            }
        }

        // Remove completed incidents (in reverse order to maintain indices)
        for index in completed_indices.into_iter().rev() {
            self.incidents.swap_remove(index);
        }
    }

    /// Get active incidents count
    pub fn active_count(&self) -> usize {
        self.incidents.iter().filter(|i| i.is_active()).count()
    }

    /// Get completed incidents count
    pub fn completed_count(&self) -> usize {
        self.incidents.iter().filter(|i| i.is_completed()).count()
    }

    /// Clear all incidents
    pub fn clear(&mut self) {
        self.incidents.clear();
    }

    /// Set custom incident handler
    pub fn set_handler(&mut self, handler: Box<dyn IncidentHandler>) {
        self.handler = handler;
    }
}

impl Default for IncidentManager {
    fn default() -> Self {
        Self::new(1000)
    }
}
