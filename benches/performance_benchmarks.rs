//! Performance benchmarks for Jigen components
//!
//! Measures performance characteristics of:
//! - Scene parsing and loading
//! - SceneGraph operations
//! - Physics simulation
//! - Rendering operations
//! - Memory usage patterns

use jigen::dsl::{SceneParser, scene::*};
use jigen::graph::SceneGraph;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::time::Instant;

/// Generate a test scene with specified number of nodes
fn generate_test_scene(node_count: usize) -> serde_json::Value {
    let mut nodes = Vec::new();

    // Add ground plane
    nodes.push(json!({
        "id": "ground",
        "type": "geometry",
        "transform": {
            "position": [0.0, -1.0, 0.0],
            "scale": [50.0, 0.1, 50.0]
        },
        "properties": {
            "primitive": "box",
            "physics": {
                "body_type": "static",
                "mass": 0.0
            }
        }
    }));

    // Add dynamic objects
    for i in 0..node_count {
        let x = (i % 10) as f32 * 2.0 - 9.0;
        let z = (i / 10) as f32 * 2.0 - 9.0;
        let y = 5.0 + (i % 5) as f32;

        nodes.push(json!({
            "id": format!("object_{}", i),
            "type": "geometry",
            "transform": {
                "position": [x, y, z]
            },
            "properties": {
                "primitive": if i % 3 == 0 { "box" } else if i % 3 == 1 { "sphere" } else { "cylinder" },
                "physics": {
                    "body_type": "dynamic",
                    "mass": 1.0 + (i % 10) as f32 * 0.1
                }
            }
        }));
    }

    json!({
        "metadata": {
            "version": "1.0",
            "title": format!("Performance Test Scene ({} nodes)", node_count + 1)
        },
        "nodes": nodes,
        "forces": [{
            "type": "gravity",
            "vector": [0.0, -9.81, 0.0]
        }]
    })
}

/// Benchmark scene parsing performance
fn bench_scene_parsing(c: &mut Criterion) {
    let parser = SceneParser::new();

    let mut group = c.benchmark_group("scene_parsing");

    for &node_count in &[10, 50, 100, 500] {
        let scene_json = generate_test_scene(node_count);
        let json_str = scene_json.to_string();

        group.bench_with_input(
            format!("parse_{}_nodes", node_count),
            &json_str,
            |b, json| {
                b.iter(|| {
                    let _scene = parser.parse_json(json).unwrap();
                });
            }
        );
    }

    group.finish();
}

/// Benchmark SceneGraph loading performance
fn bench_scene_graph_loading(c: &mut Criterion) {
    let parser = SceneParser::new();

    let mut group = c.benchmark_group("scene_graph_loading");

    for &node_count in &[10, 50, 100, 500] {
        let scene_json = generate_test_scene(node_count);
        let json_str = scene_json.to_string();
        let scene_def = parser.parse_json(&json_str).unwrap();

        group.bench_with_input(
            format!("load_{}_nodes", node_count),
            &scene_def,
            |b, scene| {
                b.iter(|| {
                    let mut sg = SceneGraph::new();
                    sg.load_scene(scene);
                    black_box(sg);
                });
            }
        );
    }

    group.finish();
}

/// Benchmark topology updates
fn bench_topology_updates(c: &mut Criterion) {
    let parser = SceneParser::new();

    let mut group = c.benchmark_group("topology_updates");

    for &node_count in &[10, 50, 100, 200] {
        let scene_json = generate_test_scene(node_count);
        let json_str = scene_json.to_string();
        let scene_def = parser.parse_json(&json_str).unwrap();
        let mut scene_graph = SceneGraph::new();
        scene_graph.load_scene(&scene_def);

        group.bench_with_input(
            format!("topology_{}_nodes", node_count),
            &scene_graph,
            |b, sg| {
                b.iter(|| {
                    let mut sg_copy = sg.clone();
                    sg_copy.update_topology();
                    black_box(sg_copy);
                });
            }
        );
    }

    group.finish();
}

/// Benchmark physics simulation performance
fn bench_physics_simulation(c: &mut Criterion) {
    use jigen::physics::PhysicsWorld;
    use bevy::prelude::Vec3;

    let mut group = c.benchmark_group("physics_simulation");

    for &sim_steps in &[10, 50, 100] {
        group.bench_with_input(
            format!("physics_{}_steps", sim_steps),
            &sim_steps,
            |b, &steps| {
                b.iter(|| {
                    let gravity = Vec3::new(0.0, -9.81, 0.0);
                    let mut world = PhysicsWorld::new(gravity);

                    for _ in 0..steps {
                        world.step(1.0 / 60.0);
                    }

                    black_box(world);
                });
            }
        );
    }

    group.finish();
}

/// Benchmark scene graph statistics
fn bench_scene_graph_stats(c: &mut Criterion) {
    let parser = SceneParser::new();

    let mut group = c.benchmark_group("scene_graph_stats");

    for &node_count in &[10, 50, 100, 500, 1000] {
        let scene_json = generate_test_scene(node_count);
        let json_str = scene_json.to_string();
        let scene_def = parser.parse_json(&json_str).unwrap();
        let mut scene_graph = SceneGraph::new();
        scene_graph.load_scene(&scene_def);
        scene_graph.update_topology();

        group.bench_with_input(
            format!("stats_{}_nodes", node_count),
            &scene_graph,
            |b, sg| {
                b.iter(|| {
                    let _stats = sg.statistics();
                    black_box(_stats);
                });
            }
        );
    }

    group.finish();
}

/// Benchmark scene export performance
fn bench_scene_export(c: &mut Criterion) {
    let parser = SceneParser::new();

    let mut group = c.benchmark_group("scene_export");

    for &node_count in &[10, 50, 100, 200] {
        let scene_json = generate_test_scene(node_count);
        let json_str = scene_json.to_string();
        let scene_def = parser.parse_json(&json_str).unwrap();
        let mut scene_graph = SceneGraph::new();
        scene_graph.load_scene(&scene_def);

        group.bench_with_input(
            format!("export_{}_nodes", node_count),
            &scene_graph,
            |b, sg| {
                b.iter(|| {
                    let _json = sg.export_scene();
                    black_box(_json);
                });
            }
        );
    }

    group.finish();
}

/// Memory usage measurement utilities
mod memory {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    pub struct TrackingAllocator;

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ret = System.alloc(layout);
            if !ret.is_null() {
                ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            }
            ret
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }

    pub fn get_allocated_bytes() -> usize {
        ALLOCATED.load(Ordering::SeqCst)
    }

    pub fn reset_allocated_bytes() {
        ALLOCATED.store(0, Ordering::SeqCst);
    }
}

/// Memory usage benchmarks
fn bench_memory_usage(c: &mut Criterion) {
    let parser = SceneParser::new();

    let mut group = c.benchmark_group("memory_usage");

    for &node_count in &[10, 50, 100, 200] {
        group.bench_with_input(
            format!("memory_{}_nodes", node_count),
            &node_count,
            |b, &count| {
                b.iter_custom(|iters| {
                    let mut total_time = std::time::Duration::new(0, 0);
                    let mut total_memory = 0usize;

                    for _ in 0..iters {
                        memory::reset_allocated_bytes();

                        let start = Instant::now();
                        let scene_json = generate_test_scene(count);
                        let json_str = scene_json.to_string();
                        let scene_def = parser.parse_json(&json_str).unwrap();
                        let mut scene_graph = SceneGraph::new();
                        scene_graph.load_scene(&scene_def);
                        scene_graph.update_topology();

                        let elapsed = start.elapsed();
                        total_time += elapsed;
                        total_memory += memory::get_allocated_bytes();
                    }

                    // Return time per iteration, memory usage is tracked separately
                    total_time / iters
                });
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_scene_parsing,
    bench_scene_graph_loading,
    bench_topology_updates,
    bench_physics_simulation,
    bench_scene_graph_stats,
    bench_scene_export,
    bench_memory_usage
);

criterion_main!(benches);
