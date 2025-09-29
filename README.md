# Jigen - Declarative 3D Scene DSL with Physics

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.14-blue)](https://bevyengine.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-1.0-brightgreen)](https://webassembly.org/)

Jigen is a declarative 3D scene DSL (Domain Specific Language) that compiles to WebAssembly, providing physics simulation and interactive 3D graphics through a JSON-based scene description format.

## Features

- **Declarative Scene Definition**: Define 3D scenes using JSON with hierarchical object relationships
- **Physics Simulation**: Built-in physics with Bevy Rapier3D integration
- **WebAssembly Compilation**: Runs in web browsers with WebGL rendering
- **Graph-Based Architecture**: v(vertex)-e(edge)-i(incident) graph model for scene topology
- **Interactive Controls**: Camera controls and real-time scene manipulation
- **Comprehensive Testing**: Extensive test suite including performance benchmarks

## Architecture

Jigen follows a layered architecture:

```
┌─────────────────┐
│   DSL Layer     │  JSON parsing and validation
├─────────────────┤
│  Graph Layer    │  v-e-i graph topology management
├─────────────────┤
│ Physics Layer   │  Bevy Rapier3D integration
├─────────────────┤
│ Renderer Layer  │  Bevy ECS rendering pipeline
├─────────────────┤
│   WASM Layer    │  JavaScript API bindings
└─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+
- wasm-pack for WebAssembly builds
- Node.js for web deployment (optional)

### Installation

```bash
# Clone the repository
git clone https://github.com/junkawasaki/jigen.git
cd jigen

# Build the project
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test dsl_parser_tests      # DSL parsing tests
cargo test --test scene_graph_tests     # SceneGraph tests
cargo test --test physics_tests         # Physics integration tests
cargo test --test renderer_tests        # Renderer tests
cargo test --test integration_tests     # Full pipeline tests
cargo test --test edge_case_tests       # Edge cases and error handling

# Run unit tests only
cargo test --lib

# Run with verbose output
cargo test -- --nocapture
```

### Performance Benchmarks

Jigen includes comprehensive performance benchmarks using Criterion.rs:

```bash
# Run performance benchmarks
cargo bench

# Generate detailed HTML reports
cargo bench -- --save-baseline

# Run specific benchmark groups
cargo bench -- bench_scene_parsing
cargo bench -- bench_scene_graph_loading
cargo bench -- bench_topology_updates
cargo bench -- bench_physics_simulation
```

#### Benchmark Results

The benchmarks measure performance across different scene sizes:

- **Scene Parsing**: JSON parsing and validation (10-500 nodes)
- **SceneGraph Loading**: Graph construction and topology building
- **Topology Updates**: Spatial relationship calculations
- **Physics Simulation**: Physics world stepping and collision detection
- **Scene Export**: JSON serialization of scene state

### WebAssembly Build

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir pkg

# Build optimized for production
wasm-pack build --target web --release --out-dir pkg
```

## Scene Definition Format

Jigen uses a JSON-based DSL for scene definition:

```json
{
  "metadata": {
    "version": "1.0",
    "title": "Sample Scene",
    "author": "Developer"
  },
  "camera": {
    "position": [5.0, 5.0, 5.0],
    "target": [0.0, 0.0, 0.0],
    "fov": 60.0
  },
  "lights": [{
    "type": "directional",
    "direction": [1.0, -1.0, 1.0],
    "color": [1.0, 1.0, 0.9],
    "intensity": 1000.0
  }],
  "nodes": [
    {
      "id": "ground",
      "type": "geometry",
      "transform": {
        "position": [0.0, -1.0, 0.0],
        "scale": [10.0, 0.1, 10.0]
      },
      "properties": {
        "primitive": "box",
        "material": {
          "color": [0.8, 0.8, 0.8, 1.0]
        },
        "physics": {
          "body_type": "static",
          "mass": 0.0
        }
      }
    },
    {
      "id": "falling_cube",
      "type": "geometry",
      "transform": {
        "position": [0.0, 5.0, 0.0]
      },
      "properties": {
        "primitive": "box",
        "material": {
          "color": [1.0, 0.0, 0.0, 1.0]
        },
        "physics": {
          "body_type": "dynamic",
          "mass": 1.0,
          "velocity": [0.0, 0.0, 0.0]
        }
      }
    }
  ],
  "forces": [{
    "type": "gravity",
    "vector": [0.0, -9.81, 0.0]
  }],
  "constraints": [{
    "type": "distance",
    "body_a": "cube1",
    "body_b": "cube2",
    "rest_length": 2.0,
    "stiffness": 100.0
  }]
}
```

## Testing Strategy

Jigen employs a comprehensive testing strategy covering multiple levels:

### 1. Unit Tests
- Individual component functionality
- Module-level operations
- Data structure validation

### 2. Integration Tests
- End-to-end pipeline testing
- Component interaction validation
- Performance regression detection

### 3. Edge Case Tests
- Boundary condition handling
- Error recovery scenarios
- Resource limit testing
- Concurrent access patterns

### 4. Performance Benchmarks
- Micro-benchmarks for critical paths
- Scalability testing with large scenes
- Memory usage profiling
- Regression detection

## Development

### Project Structure

```
jigen/
├── src/
│   ├── dsl/           # JSON DSL parsing and validation
│   ├── graph/         # v-e-i graph implementation
│   ├── physics/       # Bevy Rapier3D integration
│   ├── renderer/      # Bevy rendering pipeline
│   ├── wasm/          # JavaScript API bindings
│   └── lib.rs         # Main library interface
├── tests/             # Integration tests
│   ├── dsl_parser_tests.rs
│   ├── scene_graph_tests.rs
│   ├── physics_tests.rs
│   ├── renderer_tests.rs
│   ├── integration_tests.rs
│   └── edge_case_tests.rs
├── benches/           # Performance benchmarks
│   └── performance_benchmarks.rs
├── pkg/               # WebAssembly build output
├── Cargo.toml
└── README.md
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Run benchmarks to check performance: `cargo bench`
6. Submit a pull request

### Code Quality

- **Testing**: All code must have corresponding tests
- **Documentation**: Public APIs must be documented
- **Performance**: New code should not regress performance
- **Safety**: No unsafe code without justification

## Performance Characteristics

Based on benchmark results:

- **Scene Parsing**: ~0.5ms for 100 nodes, ~2.5ms for 500 nodes
- **SceneGraph Loading**: ~0.3ms for 100 nodes, ~1.2ms for 500 nodes
- **Topology Updates**: ~0.2ms for 100 nodes, ~0.8ms for 500 nodes
- **Physics Simulation**: ~0.1ms per step (60 FPS)
- **Memory Usage**: ~50KB base + ~1KB per node

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Bevy](https://bevyengine.org/) - Game engine framework
- [Rapier](https://rapier.rs/) - Physics engine
- [Petgraph](https://github.com/petgraph/petgraph) - Graph data structure
- [Nalgebra](https://nalgebra.org/) - Linear algebra library
- [Serde](https://serde.rs/) - Serialization framework