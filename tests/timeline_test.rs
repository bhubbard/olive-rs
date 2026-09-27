use num_rational::Rational64;
use olive_rs::timeline::{
    Block, BlockTrimCommand, Sequence, Track, TrackList, TrackListInsertGaps,
    TrackReplaceBlockWithGapCommand, TrackType, TrimMode,
};

#[test]
fn test_sequence_defaults() {
    let mut sequence = Sequence::new("Test Sequence", 1920, 1080, Rational64::new(30, 1));
    sequence.add_default_nodes();

    assert_eq!(sequence.get_tracks().len(), 2);
    assert!(sequence.connected_texture_output.is_some());
    assert!(sequence.connected_samples_output.is_some());
    assert_ne!(
        sequence.connected_texture_output,
        sequence.connected_samples_output
    );
}

#[test]
fn test_add_track() {
    let mut sequence = Sequence::new("Test Sequence", 1920, 1080, Rational64::new(30, 1));

    // First video track
    let first_v = sequence.add_track(TrackType::Video, false);
    assert_eq!(sequence.connected_texture_output, Some(first_v));
    assert_eq!(sequence.track_list(TrackType::Video).get_track_count(), 1);

    // First audio track
    let first_a = sequence.add_track(TrackType::Audio, false);
    assert_eq!(sequence.connected_samples_output, Some(first_a));
    assert_eq!(sequence.track_list(TrackType::Audio).get_track_count(), 1);

    // Second video track with merge
    let second_v = sequence.add_track(TrackType::Video, true);
    assert_ne!(sequence.connected_texture_output, Some(first_v));
    assert_ne!(sequence.connected_texture_output, Some(second_v));
    assert_eq!(sequence.track_list(TrackType::Video).get_track_count(), 2);

    // Second audio track with merge
    let second_a = sequence.add_track(TrackType::Audio, true);
    assert_ne!(sequence.connected_samples_output, Some(first_a));
    assert_ne!(sequence.connected_samples_output, Some(second_a));
    assert_eq!(sequence.track_list(TrackType::Audio).get_track_count(), 2);
}

#[test]
fn test_trim() {
    let mut track = Track::new("Video Track", TrackType::Video);
    let mut b1 = Block::new_clip("Clip 1", Rational64::new(2, 1));
    b1.set_length_and_media_out(Rational64::new(2, 1));
    let b1_id = b1.id;
    track.append_block(b1);

    let mut b2 = Block::new_clip("Clip 2", Rational64::new(2, 1));
    b2.set_length_and_media_out(Rational64::new(2, 1));
    let b2_id = b2.id;
    track.append_block(b2);

    assert_eq!(track.blocks().len(), 2);

    // 1. Trim out point of second block
    {
        let mut cmd = BlockTrimCommand::new(b2_id, Rational64::new(1, 1), TrimMode::TrimOut);
        cmd.redo(&mut track);

        // No gap should have been added because block2 is at the end of the track
        assert_eq!(track.blocks().len(), 2);
        assert_eq!(track.blocks()[0].length, Rational64::new(2, 1));
        assert_eq!(track.blocks()[1].length, Rational64::new(1, 1));

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 2);
        assert_eq!(track.blocks()[1].length, Rational64::new(2, 1));
    }

    // 2. Trim in point of second block
    {
        let mut cmd = BlockTrimCommand::new(b2_id, Rational64::new(1, 1), TrimMode::TrimIn);
        cmd.redo(&mut track);

        // Gap should be inserted in between
        assert_eq!(track.blocks().len(), 3);
        assert!(track.blocks()[1].is_gap());
        assert_eq!(track.blocks()[1].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[0].length, Rational64::new(2, 1));
        assert_eq!(track.blocks()[2].length, Rational64::new(1, 1));

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 2);
        assert_eq!(track.blocks()[1].length, Rational64::new(2, 1));
    }

    // 3. Trim out point of first block
    {
        let mut cmd = BlockTrimCommand::new(b1_id, Rational64::new(1, 1), TrimMode::TrimOut);
        cmd.redo(&mut track);

        // Gap should be inserted in between
        assert_eq!(track.blocks().len(), 3);
        assert!(track.blocks()[1].is_gap());
        assert_eq!(track.blocks()[1].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[0].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[2].length, Rational64::new(2, 1));

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 2);
        assert_eq!(track.blocks()[0].length, Rational64::new(2, 1));
    }

    // 4. Trim in point of first block
    {
        let mut cmd = BlockTrimCommand::new(b1_id, Rational64::new(1, 1), TrimMode::TrimIn);
        cmd.redo(&mut track);

        // Gap should be prepended to the start
        assert_eq!(track.blocks().len(), 3);
        assert!(track.blocks()[0].is_gap());
        assert_eq!(track.blocks()[0].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[1].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[2].length, Rational64::new(2, 1));

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 2);
        assert_eq!(track.blocks()[0].length, Rational64::new(2, 1));
    }
}

#[test]
fn test_replace_block_with_gap_clips_only() {
    let mut track = Track::new("Video", TrackType::Video);

    let a = Block::new_clip("A", Rational64::new(1, 1));
    let b = Block::new_clip("B", Rational64::new(1, 1));
    let c = Block::new_clip("C", Rational64::new(1, 1));
    let a_id = a.id;
    let b_id = b.id;
    let c_id = c.id;

    track.append_block(a);
    track.append_block(b);
    track.append_block(c);

    // Replace clip C (end of track) with gap: should be removed completely
    {
        let mut cmd = TrackReplaceBlockWithGapCommand::new(c_id);
        cmd.redo(&mut track);

        assert_eq!(track.blocks().len(), 2);
        assert_eq!(track.blocks()[0].id, a_id);
        assert_eq!(track.blocks()[1].id, b_id);

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 3);
        assert_eq!(track.blocks()[2].id, c_id);
    }

    // Replace clip B (middle of track) with gap: replaced with Gap
    {
        let mut cmd = TrackReplaceBlockWithGapCommand::new(b_id);
        cmd.redo(&mut track);

        assert_eq!(track.blocks().len(), 3);
        assert_eq!(track.blocks()[0].id, a_id);
        assert!(track.blocks()[1].is_gap());
        assert_eq!(track.blocks()[1].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[2].id, c_id);

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 3);
        assert_eq!(track.blocks()[1].id, b_id);
    }
}

#[test]
fn test_replace_block_with_gap_clips_and_gaps() {
    // A (clip) -> B (gap) -> C (clip) -> D (gap) -> E (clip)
    let mut track = Track::new("Video", TrackType::Video);

    let a = Block::new_clip("A", Rational64::new(1, 1));
    let b = Block::new_gap(Rational64::new(1, 1));
    let c = Block::new_clip("C", Rational64::new(1, 1));
    let d = Block::new_gap(Rational64::new(1, 1));
    let e = Block::new_clip("E", Rational64::new(1, 1));

    let a_id = a.id;
    let b_id = b.id;
    let c_id = c.id;
    let e_id = e.id;

    track.append_block(a);
    track.append_block(b);
    track.append_block(c);
    track.append_block(d);
    track.append_block(e);

    // 1. Replace clip E with a gap: both D and E should be removed because of trailing gap removal
    {
        let mut cmd = TrackReplaceBlockWithGapCommand::new(e_id);
        cmd.redo(&mut track);

        assert_eq!(track.blocks().len(), 3);
        assert_eq!(track.blocks()[0].id, a_id);
        assert_eq!(track.blocks()[1].id, b_id);
        assert_eq!(track.blocks()[2].id, c_id);

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 5);
        assert_eq!(track.blocks()[4].id, e_id);
    }

    // 2. Replace clip A with a gap: A should merge with B into single gap of length 2
    {
        let mut cmd = TrackReplaceBlockWithGapCommand::new(a_id);
        cmd.redo(&mut track);

        assert_eq!(track.blocks().len(), 4);
        assert!(track.blocks()[0].is_gap());
        assert_eq!(track.blocks()[0].length, Rational64::new(2, 1));
        assert_eq!(track.blocks()[1].id, c_id);

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 5);
        assert_eq!(track.blocks()[0].id, a_id);
        assert_eq!(track.blocks()[0].length, Rational64::new(1, 1));
    }

    // 3. Replace clip C with a gap: B, C, D all merge into a single gap of length 3
    {
        let mut cmd = TrackReplaceBlockWithGapCommand::new(c_id);
        cmd.redo(&mut track);

        assert_eq!(track.blocks().len(), 3);
        assert_eq!(track.blocks()[0].id, a_id);
        assert!(track.blocks()[1].is_gap());
        assert_eq!(track.blocks()[1].length, Rational64::new(3, 1));
        assert_eq!(track.blocks()[2].id, e_id);

        cmd.undo(&mut track);
        assert_eq!(track.blocks().len(), 5);
        assert_eq!(track.blocks()[2].id, c_id);
        assert_eq!(track.blocks()[1].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[2].length, Rational64::new(1, 1));
        assert_eq!(track.blocks()[3].length, Rational64::new(1, 1));
    }
}

#[test]
fn test_insert_gaps_single_track() {
    let mut track_list = TrackList::new(TrackType::Video);
    let mut track = Track::new("Video Track", TrackType::Video);

    let a = Block::new_clip("A", Rational64::new(1, 1));
    let b = Block::new_clip("B", Rational64::new(1, 1));
    let c = Block::new_clip("C", Rational64::new(1, 1));
    let a_id = a.id;
    let b_id = b.id;
    let c_id = c.id;

    track.append_block(a);
    track.append_block(b);
    track.append_block(c);
    track_list.add_track(track);

    // 1. Insert gap at start (time 0, length 2)
    {
        let mut cmd = TrackListInsertGaps::new(Rational64::new(0, 1), Rational64::new(2, 1));
        cmd.redo(&mut track_list);

        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 4);
        assert!(t.blocks()[0].is_gap());
        assert_eq!(t.blocks()[0].length, Rational64::new(2, 1));
        assert_eq!(t.blocks()[1].id, a_id);
        assert_eq!(t.blocks()[2].id, b_id);
        assert_eq!(t.blocks()[3].id, c_id);

        cmd.undo(&mut track_list);
        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 3);
        assert_eq!(t.blocks()[0].id, a_id);
    }

    // 2. Insert gap in middle of clip A (time 1/2, length 2)
    {
        let mut cmd = TrackListInsertGaps::new(Rational64::new(1, 2), Rational64::new(2, 1));
        cmd.redo(&mut track_list);

        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 5);
        assert_eq!(t.blocks()[0].id, a_id);
        assert_eq!(t.blocks()[0].length, Rational64::new(1, 2));
        assert!(t.blocks()[1].is_gap());
        assert_eq!(t.blocks()[1].length, Rational64::new(2, 1));
        assert_eq!(t.blocks()[2].length, Rational64::new(1, 2));
        assert_eq!(t.blocks()[3].id, b_id);
        assert_eq!(t.blocks()[4].id, c_id);

        cmd.undo(&mut track_list);
        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 3);
        assert_eq!(t.blocks()[0].id, a_id);
        assert_eq!(t.blocks()[0].length, Rational64::new(1, 1));
    }

    // 3. Insert gap between clips A and B (time 1, length 2)
    {
        let mut cmd = TrackListInsertGaps::new(Rational64::new(1, 1), Rational64::new(2, 1));
        cmd.redo(&mut track_list);

        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 4);
        assert_eq!(t.blocks()[0].id, a_id);
        assert!(t.blocks()[1].is_gap());
        assert_eq!(t.blocks()[1].length, Rational64::new(2, 1));
        assert_eq!(t.blocks()[2].id, b_id);
        assert_eq!(t.blocks()[3].id, c_id);

        cmd.undo(&mut track_list);
        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 3);
        assert_eq!(t.blocks()[0].id, a_id);
    }

    // 4. Insert gap at end (time 3, length 2) -> nothing added
    {
        let mut cmd = TrackListInsertGaps::new(Rational64::new(3, 1), Rational64::new(2, 1));
        cmd.redo(&mut track_list);

        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 3);
        assert_eq!(t.blocks()[0].id, a_id);
        assert_eq!(t.blocks()[1].id, b_id);
        assert_eq!(t.blocks()[2].id, c_id);

        cmd.undo(&mut track_list);
        let t = track_list.get_track(0).unwrap();
        assert_eq!(t.blocks().len(), 3);
    }
}
