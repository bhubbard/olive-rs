use std::path::PathBuf;
use std::sync::Arc;
use clap::{Parser, Subcommand};
use num_rational::Rational64;

use olive_rs::buffer::RgbaColor;
use olive_rs::graph::dag::Graph;
use olive_rs::graph::node::{EvaluationContext, NodeOutputData};
use olive_rs::nodes::merge::{BlendMode, MergeNode};
use olive_rs::nodes::solid::SolidGeneratorNode;
use olive_rs::nodes::transform::TransformDistortNode;
use olive_rs::project::OliveProject;
use olive_rs::Node;

#[derive(Parser, Debug)]
#[command(name = "olive-render")]
#[command(about = "Headless node graph and timeline compositor for olive-rs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Render a single frame from a project or test node graph
    RenderFrame {
        #[arg(short, long, default_value = "frame.png")]
        output: PathBuf,

        #[arg(long, default_value_t = 1920)]
        width: u32,

        #[arg(long, default_value_t = 1080)]
        height: u32,

        #[arg(long, default_value_t = 0)]
        frame: i64,
    },
    /// Inspect an Olive project JSON file
    Inspect {
        #[arg(short, long)]
        project: PathBuf,
    },
    /// Initialize a new default Olive project file
    NewProject {
        #[arg(short, long, default_value = "project.ove.json")]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RenderFrame {
            output,
            width,
            height,
            frame,
        } => {
            println!("Rendering frame {} at {}x{} to {:?}...", frame, width, height, output);

            let mut graph = Graph::new();

            // Background solid (Navy Blue)
            let bg = Arc::new(SolidGeneratorNode::new(
                "Background",
                RgbaColor::new(0.05, 0.1, 0.25, 1.0),
            ));
            let bg_id = bg.id();
            graph.add_node(bg);

            // Foreground solid (Amber Yellow)
            let fg = Arc::new(SolidGeneratorNode::new(
                "Foreground Box",
                RgbaColor::new(0.95, 0.65, 0.15, 0.9),
            ));
            let fg_id = fg.id();
            graph.add_node(fg);

            // Transform on Foreground
            let mut xform = TransformDistortNode::new("Center Box Transform");
            xform.scale_x.default_value = 0.5;
            xform.scale_y.default_value = 0.5;
            xform.rotation.default_value = (frame as f64) * 2.0; // rotate with frame
            let xform_arc = Arc::new(xform);
            let xform_id = xform_arc.id();
            graph.add_node(xform_arc);

            // Merge node
            let merge = Arc::new(MergeNode::new("Compositor Merge", BlendMode::Normal));
            let merge_id = merge.id();
            graph.add_node(merge);

            // Connect graph
            graph.connect(fg_id, "tex_out", xform_id, "tex_in")?;
            graph.connect(bg_id, "tex_out", merge_id, MergeNode::BASE_IN)?;
            graph.connect(xform_id, "tex_out", merge_id, MergeNode::BLEND_IN)?;

            let ctx = EvaluationContext {
                time: Rational64::new(frame, 30),
                width,
                height,
                sample_rate: 48000,
            };

            let results = graph.evaluate(&ctx)?;
            if let Some(merge_outs) = results.get(&merge_id) {
                if let Some(NodeOutputData::Texture(ref img)) = merge_outs.get(MergeNode::TEX_OUT) {
                    let raw_bytes = img.to_rgba8_vec();
                    image::save_buffer(
                        &output,
                        &raw_bytes,
                        width,
                        height,
                        image::ExtendedColorType::Rgba8,
                    )?;
                    println!("Successfully rendered frame to {:?}", output);
                }
            }
        }
        Commands::Inspect { project } => {
            let content = std::fs::read_to_string(&project)?;
            let proj = OliveProject::from_json(&content)?;
            println!("Olive Project: version {}", proj.version);
            println!("Generator: {}", proj.generator);
            println!("Sequences: {}", proj.sequences.len());
            for seq in &proj.sequences {
                println!(
                    " - Sequence '{}' ({}x{} @ {}/{} fps) with {} video tracks, {} audio tracks",
                    seq.name,
                    seq.width,
                    seq.height,
                    seq.frame_rate_num,
                    seq.frame_rate_den,
                    seq.video_tracks.len(),
                    seq.audio_tracks.len()
                );
            }
            println!("Nodes: {}", proj.nodes.len());
            println!("Connections: {}", proj.connections.len());
        }
        Commands::NewProject { output } => {
            let proj = OliveProject::new();
            let json = proj.to_json()?;
            std::fs::write(&output, json)?;
            println!("Created new project file at {:?}", output);
        }
    }

    Ok(())
}
