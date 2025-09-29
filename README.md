# Jigen - Declarative 3D Scene DSL with Physics

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.14-black)](https://bevyengine.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-supported-blue)](https://webassembly.org/)

Jigen is a declarative 3D scene description framework built on [Bevy](https://bevyengine.org/) that enables A-Frame/Three.js React-like syntax with integrated physics simulation. It uses a v(vertex), e(edge), i(incident) graph model for scene topology management.

## Features

- **🎨 Declarative DSL**: JSON-based scene description similar to A-Frame and Three.js React
- **⚡ Physics Simulation**: Integrated force dynamics with Rapier physics engine
- **🔗 Graph Model**: v-e-i (vertex-edge-incident) graph structure for scene relationships
- **🌐 WebAssembly**: Run in browsers with full WebAssembly support
- **🎯 Bevy Integration**: Leverages Bevy's ECS and rendering pipeline
- **🔧 Extensible**: Plugin architecture for custom forces, constraints, and behaviors

## Architecture

Jigen follows a process network graph model where:

- **v (Vertices)**: Scene nodes/objects with properties and state
- **e (Edges)**: Relationships between vertices (parent-child, spatial, force interactions)
- **i (Incidents)**: Events and interactions (collisions, state changes, user input)

## Quick Start

### Native Build

```bash
# Clone the repository
git clone https://github.com/yourusername/jigen.git
cd jigen

# Build and run
cargo run --release
```

### WebAssembly Build

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for web
wasm-pack build --target web --out-dir pkg

# Serve locally
cd pkg && python3 -m http.server 8000
```

### JavaScript Usage

```javascript
import init, { JigenWasmApp, create_minimal_scene } from './pkg/jigen.js';

async function run() {
    // Initialize WASM module
    await init();

    // Create app
    const app = new JigenWasmApp();

    // Load minimal scene
    const sceneJson = create_minimal_scene();
    app.load_scene(sceneJson);

    // Animation loop
    function animate() {
        app.update(1/60); // 60 FPS
        requestAnimationFrame(animate);
    }
    animate();
}

run();
```

## Scene DSL Example

```json
{
  "metadata": {
    "version": "1.0",
    "title": "Physics Demo",
    "author": "Jigen User"
  },
  "globals": {
    "background": {"r": 0.1, "g": 0.1, "b": 0.1, "a": 1.0},
    "physics": {
      "enabled": true,
      "gravity": [0.0, -9.81, 0.0]
    }
  },
  "nodes": [
    {
      "id": "ground",
      "type": "geometry",
      "name": "Ground Plane",
      "transform": {
        "position": [0.0, -5.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [10.0, 1.0, 10.0]
      },
      "properties": {
        "primitive": "box",
        "material": {
          "color": {"r": 0.5, "g": 0.5, "b": 0.5, "a": 1.0}
        },
        "physics": {
          "type": "static",
          "friction": 0.8
        }
      }
    },
    {
      "id": "cube",
      "type": "geometry",
      "name": "Falling Cube",
      "transform": {
        "position": [0.0, 5.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
      },
      "properties": {
        "primitive": "box",
        "material": {
          "color": {"r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0}
        },
        "physics": {
          "type": "dynamic",
          "mass": 1.0,
          "friction": 0.5,
          "restitution": 0.3
        }
      }
    },
    {
      "id": "light",
      "type": "light",
      "name": "Main Light",
      "transform": {
        "position": [4.0, 8.0, 4.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 1.0, 1.0]
      },
      "properties": {
        "light_type": "point",
        "color": {"r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0},
        "intensity": 1500.0,
        "range": 20.0
      }
    }
  ]
}
```

## API Reference

### JigenWasmApp

Main WASM application class.

#### Methods

- `new()` - Create new application
- `load_scene(json: string)` - Load scene from JSON
- `update(deltaTime: number)` - Update simulation
- `apply_force(objectId: string, x: number, y: number, z: number)` - Apply force to object
- `set_position(objectId: string, x: number, y: number, z: number)` - Set object position
- `get_position(objectId: string)` - Get object position as JSON
- `add_spring(id: string, objectA: string, objectB: string, restLength: number, stiffness: number)` - Add spring constraint
- `set_gravity(x: number, y: number, z: number)` - Set gravity vector
- `clear_scene()` - Clear all objects

### Physics Features

- **Rigid Body Dynamics**: Static, dynamic, and kinematic bodies
- **Collision Detection**: Broad and narrow phase collision detection
- **Constraints**: Distance, hinge, ball-socket, and fixed joints
- **Forces**: Gravity, springs, damping, wind, custom forces
- **Materials**: Friction, restitution, density

### Graph Model

- **Vertices**: Scene objects with transform, physics, and visual properties
- **Edges**: Relationships (hierarchy, spatial proximity, forces, constraints)
- **Incidents**: Events (collisions, state changes, user interactions)

## Building

### Prerequisites

- Rust 1.70+
- wasm-pack (for WASM builds)
- Node.js/npm (for web examples)

### Build Commands

```bash
# Native build
cargo build --release

# WASM build
wasm-pack build --target web

# Build examples
cargo build --examples

# Run tests
cargo test
```

## Examples

See the `examples/` directory for:

- `basic_scene.rs` - Simple scene with physics
- `constraints.rs` - Joint and constraint examples
- `forces.rs` - Custom force implementations
- `web/` - JavaScript/web examples

## Contributing

Contributions welcome! Please:

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Ensure WASM compatibility

## License

Licensed under MIT or Apache 2.0.

## Acknowledgments

- [Bevy](https://bevyengine.org/) - Game engine framework
- [Rapier](https://rapier.rs/) - Physics engine
- [A-Frame](https://aframe.io/) - Declarative 3D inspiration
- [Three.js](https://threejs.org/) - WebGL framework

## Process Network Graph Model

Jigen implements a declarative process network based on Merkle DAG topology:

```
dag.jsonnet defines:
├── rust_toolchain (infrastructure)
├── bevy_core (library)
├── json_dsl_parser (library)
├── incident_graph (library)
├── physics_engine (library)
├── wasm_runtime (runtime)
├── scene_renderer (application)
└── dsl_compiler (tool)
```

All changes follow topological sort for build ordering and reverse topological sort for debugging/problem resolution.
