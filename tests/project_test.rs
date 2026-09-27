use olive_rs::project::{OliveProject, SerializedNode, SerializedSequence, SerializedTrack};
use olive_rs::timeline::block::Block;
use olive_rs::timeline::track::TrackType;
use num_rational::Rational64;
use uuid::Uuid;

#[test]
fn test_project_serialization_roundtrip() {
    let mut project = OliveProject::new();

    let track = SerializedTrack {
        id: Uuid::new_v4(),
        name: "Video 1".into(),
        track_type: TrackType::Video,
        blocks: vec![Block::new_clip("Intro Clip", Rational64::new(60, 1))],
    };

    let sequence = SerializedSequence {
        id: Uuid::new_v4(),
        name: "Main Timeline".into(),
        width: 3840,
        height: 2160,
        frame_rate_num: 60,
        frame_rate_den: 1,
        video_tracks: vec![track],
        audio_tracks: vec![],
    };

    let node = SerializedNode {
        id: Uuid::new_v4(),
        name: "Color Corrector".into(),
        category: "color".into(),
        properties: std::collections::HashMap::from([("gamma".into(), "1.2".into())]),
    };

    project.sequences.push(sequence);
    project.nodes.push(node);

    let json = project.to_json().unwrap();
    let loaded = OliveProject::from_json(&json).unwrap();

    assert_eq!(loaded.version, OliveProject::CURRENT_VERSION);
    assert_eq!(loaded.sequences.len(), 1);
    assert_eq!(loaded.sequences[0].name, "Main Timeline");
    assert_eq!(loaded.sequences[0].width, 3840);
    assert_eq!(loaded.sequences[0].height, 2160);
    assert_eq!(loaded.sequences[0].video_tracks.len(), 1);
    assert_eq!(loaded.sequences[0].video_tracks[0].blocks.len(), 1);
    assert_eq!(loaded.nodes.len(), 1);
    assert_eq!(loaded.nodes[0].properties.get("gamma").unwrap(), "1.2");
}
