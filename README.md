# olive-rs 🫒🦀

[![CI](https://github.com/bhubbard/olive-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/olive-rs/actions/workflows/ci.yml)
[![Pages](https://github.com/bhubbard/olive-rs/actions/workflows/pages.yml/badge.svg)](http://code.brandonhubbard.com/olive-rs/)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.0.1-orange.svg)](https://crates.io)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A high-performance, memory-safe, headless node-based video compositing engine and non-linear editing (NLE) timeline architecture in pure Rust, inspired by [Olive Video Editor](https://github.com/olive-editor/olive).

Explore the live interactive node studio, timeline simulator, and project inspector:
👉 **[http://code.brandonhubbard.com/olive-rs/](http://code.brandonhubbard.com/olive-rs/)**

---

## ✨ Features

- **Directed Acyclic Graph (DAG) Compositing**: Topological evaluation order with cycle detection, multi-input/output pins, and dead-branch pruning.
- **Parametric Keyframing & Animation**: Continuous evaluation across time with Linear, Hold, and Cubic Bezier curve interpolation.
- **Node Library**:
  - `SolidGeneratorNode`: Solid color / alpha generator.
  - `TransformDistortNode`: Sub-pixel position ($x, y$), scale ($s_x, s_y$), and rotation ($\theta$).
  - `CropDistortNode`: Directional edge cropping.
  - `MergeNode`: Alpha Over compositing with blend modes (Normal, Multiply, Screen, Overlay, Add, Subtract, Darken, Lighten, Difference).
  - `ColorGradeNode`: Brightness, contrast, saturation, and gamma color correction.
  - `BlurNode`: Spatial box / gaussian blur filter.
  - `MathNode`: Audio sample mixing and scalar math.
- **Non-Linear Timeline Engine**:
  - Multi-track timeline supporting Video and Audio tracks.
  - Ordered block hierarchy (`ClipBlock`, `GapBlock`, `TransitionBlock`).
  - Editing operations with command history: `BlockTrimCommand` (Trim In, Trim Out), `TrackReplaceBlockWithGapCommand` (gap consolidation and adjacent merging), and `TrackListInsertGaps`.
- **Project Interchange**: Full `.ove` schema serialization and deserialization (Version 230220 / JSON format).
- **Headless CLI (`olive-render`)**: Render frames and sequences without any GUI or display server required.

---

## 🚀 Quick Start

### 1. Cargo Dependency

Add `olive-rs` to your `Cargo.toml`:

```toml
[dependencies]
olive-rs = "0.0.1"
```

### 2. Node Graph Compositing Example

```rust
use std::sync::Arc;
use num_rational::Rational64;
use olive_rs::{
    BlendMode, EvaluationContext, Graph, MergeNode, Node, NodeOutputData,
    RgbaColor, SolidGeneratorNode, TransformDistortNode,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::new();

    // 1. Background Solid (Navy Blue)
    let bg = Arc::new(SolidGeneratorNode::new("BG", RgbaColor::new(0.05, 0.1, 0.25, 1.0)));
    let bg_id = bg.id();
    graph.add_node(bg);

    // 2. Foreground Solid (Amber Yellow)
    let fg = Arc::new(SolidGeneratorNode::new("FG", RgbaColor::new(0.95, 0.65, 0.15, 0.9)));
    let fg_id = fg.id();
    graph.add_node(fg);

    // 3. 2D Transform
    let mut xform = TransformDistortNode::new("Transform");
    xform.scale_x.default_value = 0.5;
    xform.scale_y.default_value = 0.5;
    let xform_arc = Arc::new(xform);
    let xform_id = xform_arc.id();
    graph.add_node(xform_arc);

    // 4. Merge Node
    let merge = Arc::new(MergeNode::new("Merge", BlendMode::Normal));
    let merge_id = merge.id();
    graph.add_node(merge);

    // 5. Connect DAG
    graph.connect(fg_id, "tex_out", xform_id, "tex_in")?;
    graph.connect(bg_id, "tex_out", merge_id, MergeNode::BASE_IN)?;
    graph.connect(xform_id, "tex_out", merge_id, MergeNode::BLEND_IN)?;

    // 6. Evaluate frame at timestamp 0
    let ctx = EvaluationContext {
        time: Rational64::new(0, 1),
        width: 1920,
        height: 1080,
        sample_rate: 48000,
    };

    let outputs = graph.evaluate(&ctx)?;
    if let Some(res) = outputs.get(&merge_id) {
        if let Some(NodeOutputData::Texture(ref img)) = res.get(MergeNode::TEX_OUT) {
            println!("Rendered composite image: {}x{}", img.width, img.height);
        }
    }

    Ok(())
}
```

---

## 🖥️ CLI Usage: `olive-render`

```bash
# Render a single composite frame to PNG
cargo run --bin olive-render -- render-frame --output frame_000.png --width 1920 --height 1080 --frame 0

# Inspect an existing Olive project file
cargo run --bin olive-render -- inspect --project project.ove.json

# Generate a new Olive project template
cargo run --bin olive-render -- new-project --output new_composition.ove.json
```

---

## 🧪 Upstream Parity & Test Suite

`olive-rs` includes 15 automated integration tests verifying:
- Digit count algorithm (`GetDigitCount` parity)
- Track appending, sequence defaults, and track lists
- In/Out block trimming with gap insertion
- Single and adjacent gap consolidation
- Cross-dissolve transition handling
- Time-based gap splitting across multi-tracks
- Sub-pixel transform and crop distortions
- Alpha Over blending modes
- Project JSON serialization round-tripping

Run the test suite:
```bash
cargo test
```

---

## 📜 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
