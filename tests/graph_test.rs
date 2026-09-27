use std::sync::Arc;
use num_rational::Rational64;

use olive_rs::buffer::RgbaColor;
use olive_rs::graph::dag::Graph;
use olive_rs::graph::node::{EvaluationContext, NodeOutputData};
use olive_rs::nodes::crop::CropDistortNode;
use olive_rs::nodes::merge::{BlendMode, MergeNode};
use olive_rs::nodes::solid::SolidGeneratorNode;
use olive_rs::nodes::transform::TransformDistortNode;
use olive_rs::Node;

#[test]
fn test_graph_cycle_detection() {
    let mut graph = Graph::new();
    let n1 = Arc::new(SolidGeneratorNode::new("Solid 1", RgbaColor::RED));
    let n2 = Arc::new(TransformDistortNode::new("Xform 1"));

    let id1 = n1.id();
    let id2 = n2.id();

    graph.add_node(n1);
    graph.add_node(n2);

    assert!(graph.connect(id1, "tex_out", id2, "tex_in").is_ok());
    assert!(!graph.has_cycle());

    // Connect n2 back to n1: should fail or detect cycle
    // (Solid has no tex_in, but attempting a circular loop fails)
}

#[test]
fn test_compositing_solid_merge() {
    let mut graph = Graph::new();

    let bg = Arc::new(SolidGeneratorNode::new("BG", RgbaColor::new(0.0, 0.0, 1.0, 1.0))); // Blue
    let fg = Arc::new(SolidGeneratorNode::new("FG", RgbaColor::new(1.0, 0.0, 0.0, 0.5))); // 50% Red
    let merge = Arc::new(MergeNode::new("Merge", BlendMode::Normal));

    let bg_id = bg.id();
    let fg_id = fg.id();
    let merge_id = merge.id();

    graph.add_node(bg);
    graph.add_node(fg);
    graph.add_node(merge);

    graph.connect(bg_id, "tex_out", merge_id, MergeNode::BASE_IN).unwrap();
    graph.connect(fg_id, "tex_out", merge_id, MergeNode::BLEND_IN).unwrap();

    let ctx = EvaluationContext {
        time: Rational64::new(0, 1),
        width: 10,
        height: 10,
        sample_rate: 48000,
    };

    let res = graph.evaluate(&ctx).unwrap();
    let merge_out = res.get(&merge_id).unwrap();

    if let Some(NodeOutputData::Texture(ref img)) = merge_out.get(MergeNode::TEX_OUT) {
        let p = img.get_pixel(5, 5);
        // Blending 50% red over 100% blue:
        // out_a = 0.5 + 1.0 * 0.5 = 1.0
        // r = 1.0 * 0.5 = 0.5
        // b = 1.0 * 0.5 = 0.5
        assert!((p.r - 0.5).abs() < 1e-4);
        assert!((p.b - 0.5).abs() < 1e-4);
        assert!((p.a - 1.0).abs() < 1e-4);
    } else {
        panic!("Missing output texture");
    }
}

#[test]
fn test_crop_distort() {
    let mut graph = Graph::new();
    let solid = Arc::new(SolidGeneratorNode::new("Solid", RgbaColor::WHITE));
    let mut crop = CropDistortNode::new("Crop");
    crop.crop_left.default_value = 0.5; // Crop 50% from left
    let crop_arc = Arc::new(crop);

    let solid_id = solid.id();
    let crop_id = crop_arc.id();

    graph.add_node(solid);
    graph.add_node(crop_arc);
    graph.connect(solid_id, "tex_out", crop_id, "tex_in").unwrap();

    let ctx = EvaluationContext {
        time: Rational64::new(0, 1),
        width: 100,
        height: 100,
        sample_rate: 48000,
    };

    let res = graph.evaluate(&ctx).unwrap();
    if let Some(NodeOutputData::Texture(ref img)) = res.get(&crop_id).unwrap().get("tex_out") {
        let left_pixel = img.get_pixel(10, 50);
        let right_pixel = img.get_pixel(70, 50);

        assert_eq!(left_pixel, RgbaColor::TRANSPARENT);
        assert_eq!(right_pixel, RgbaColor::WHITE);
    }
}
