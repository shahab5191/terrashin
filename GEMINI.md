# Terrashin: Node-Based Terrain Generation

Terrashin is a high-performance, node-based terrain generation tool built with Rust and WebGPU (`wgpu`). It employs a graph-based architecture to define and execute terrain generation workflows directly on the GPU.

## Project Overview

- **Main Technologies:** Rust (2024 Edition), `wgpu` (WebGPU), `winit` (Windowing), `glam` (Linear Algebra), `pollster`.
- **Core Architecture:**
    - **App Layer (`src/app.rs`):** Manages the `winit` event loop, window lifecycle, and orchestrates the rendering process.
    - **GPU Layer (`src/gpu/`):** Provides a structured interface to WebGPU primitives, including device context management, renderers, compute pipelines, and texture/buffer abstractions.
    - **Terrain Layer (`src/terrain/`):**
        - **Graph (`graph.rs`):** Manages a collection of nodes and their connections.
        - **Node (`node.rs`):** Defines the `Node` trait for various generation steps (e.g., `NoiseNode`, `SolidColorNode`).
        - **Executor (`executor.rs`):** Recursively evaluates the node graph, handles dependencies, and encodes GPU commands.
        - **Resource Registry (`resource_registry.rs`):** A central repository for GPU resources (textures, views) shared across nodes.
    - **Shaders (`src/shaders/`):** Contains WGSL compute and fragment shaders (e.g., noise generation, blurring).

## Building and Running

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (latest stable)
- A GPU supporting Vulkan, Metal, or DX12 (for `wgpu`)

### Commands
- **Run the application:**
  ```bash
  cargo run
  ```
- **Build in release mode:**
  ```bash
  cargo build --release
  ```
- **Run tests:**
  ```bash
  cargo test
  ```
- **Check for compilation errors:**
  ```bash
  cargo check
  ```

## Development Conventions

- **Node Implementation:** New terrain generation features should be implemented as structs that satisfy the `Node` trait in `src/terrain/node.rs`.
- **GPU Resources:** Always use the `ResourceRegistry` to manage intermediate textures and buffers between nodes to ensure proper dependency tracking.
- **Error Handling:** Use `anyhow` for high-level application logic and `thiserror` for library-level error definitions.
- **Async/Sync:** While `wgpu` is inherently asynchronous, the project uses `pollster` to block on GPU initialization in `App::resumed`.

## Key Files
- `src/main.rs`: Entry point.
- `src/app.rs`: Application logic and event handling.
- `src/gpu/context.rs`: WebGPU device and surface initialization.
- `src/terrain/node.rs`: The core `Node` trait and built-in node implementations.
- `src/terrain/executor.rs`: Logic for graph traversal and command submission.
