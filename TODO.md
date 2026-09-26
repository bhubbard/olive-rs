# TODO: `olive-rs` 🫒🦀

A high-performance, memory-safe Rust port and headless node-based video compositing engine inspired by [olive-editor/olive](https://github.com/olive-editor/olive).

---

## 🎯 Mission & Goals

- **Node-Based Compositing Architecture**: Replace traditional linear layer stacks with a Directed Acyclic Graph (DAG) for non-linear video editing and visual effects.
- **Parametric Keyframing & Animation**: Support rich Bezier curve keyframing, spatial transforms, ease-in/ease-out interpolation, and continuous tangents.
- **Color Pipeline**: Built-in linear color space workflows, OCIO (OpenColorIO) integration, and 32-bit floating point image buffers.
- **Headless Cloud & Edge Rendering**: Fully independent rendering graph that evaluates and renders Olive project files without requiring a Qt/GUI display server.

---

## 🏗️ Crates Architecture Plan

- [ ] `olive-graph`: Core DAG engine (`Node`, `Pin`, `DataFlowEdge`, `GraphCompiler`, cycle detection, topological sort).
- [ ] `olive-keyframe`: Bezier curve evaluation, interpolation modes, and animated property tracks.
- [ ] `olive-nodes`: Standard node library (Media In, Transform, Blend/Merge, Color Grade, Blur, Text, Matte, Audio Mix).
- [ ] `olive-project`: Serializer and deserializer for Olive 0.2 project schema (`.olive`).
- [ ] `olive-render`: Multi-threaded frame evaluation, GPU/CPU render pipeline, and tile-based rasterizer.
- [ ] `olive-cli`: Headless CLI renderer (`olive-render-rs`).

---

## 📋 Implementation Checklist

### Phase 1: Directed Acyclic Graph (DAG) Engine
- [ ] Implement node and pin data structures:
  ```rust
  pub trait Node: Send + Sync {
      fn id(&self) -> Uuid;
      fn name(&self) -> &str;
      fn inputs(&self) -> &[InputPin];
      fn outputs(&self) -> &[OutputPin];
      fn evaluate(&self, ctx: &EvaluationContext) -> Result<NodeOutput, GraphError>;
  }
  ```
- [ ] Graph validator: cycle detection using Tarjan's or Kahn's topological sort.
- [ ] Dead-branch pruning: optimize rendering by evaluating only nodes that connect to the final Output/Viewer pin.
- [ ] Dynamic parameter evaluation at sub-frame rational timecodes (`Timecode`).

### Phase 2: Keyframing & Curve Interpolation
- [ ] Implement keyframe interpolation modes:
  - [ ] Linear interpolation (`lerp`).
  - [ ] Hold (step).
  - [ ] Cubic Bezier curve with custom in/out handle control points.
- [ ] Spatial motion paths: 2D/3D coordinate splines with auto-tangents.
- [ ] Keyframe track serialization.

### Phase 3: Core Node Library
- [ ] **Input Nodes**:
  - [ ] Video/Image In (FFmpeg frame reader with cache).
  - [ ] Solid Color / Gradient Generator.
  - [ ] Text generator with font metrics and alignment.
- [ ] **Transform Nodes**:
  - [ ] 2D Transform: Translation ($x, y$), Scale ($s_x, s_y$), Rotation ($\theta$), Anchor point, and Motion Blur.
  - [ ] Crop and Corner Pin.
- [ ] **Compositing Nodes**:
  - [ ] Merge / Over / Alpha Blending.
  - [ ] Blend modes: Multiply, Screen, Overlay, Soft Light, Add, Subtract.
  - [ ] Chroma Keyer & Luma Keyer.
- [ ] **Color Grading**:
  - [ ] Lift/Gamma/Gain color wheels.
  - [ ] LUT application (3D `.cube` parser).

### Phase 4: Project Schema & Compatibility
- [ ] Parse Olive 0.2 XML/JSON project schema (`.olive` files).
- [ ] Map Olive node hierarchy to `olive-graph` primitives.
- [ ] Round-trip project generator to save modified projects.

### Phase 5: Audio Node Graph
- [ ] Audio track routing and mixing pins.
- [ ] Volume, stereo pan, and audio fade curves.
- [ ] Multi-channel audio sum and limiter.

### Phase 6: Headless Renderer & CLI
- [ ] Build headless renderer CLI:
  ```bash
  olive-render-rs project.olive --output render.mp4 --preset 4k-prores
  ```
- [ ] Multi-threaded tile-based or scanline render scheduler.
- [ ] Export frame sequences (PNG, OpenEXR) or encoded video files.

### Phase 7: Benchmarks & Parity Tests
- [ ] Compare render performance between Olive C++ and `olive-rs` on identical node graphs.
- [ ] Visual regression test suite validating sub-pixel accuracy of composite nodes.
