use std::path::Path;

use play_files::*;

// use slice_diff_patch::*;
// use byteorder::{ByteOrder, LittleEndian, BigEndian};
use lazy_static::lazy_static;

fn load_example(name: &str) -> Project {
    Project::read(Path::new(&format!("./examples/projects/{}/", name))).unwrap()
}

lazy_static! {
    static ref BLANK: Project = load_example("blank");
    static ref _400BPM: Project = load_example("400 bpm");
    static ref C4_ON_1: Project = load_example("c4 on 1");
    static ref EMPTY_NOTES_ON_1_3: Project = load_example("empty notes on 1+3");
    static ref SINGLE_EMPTY_NOTE: Project = load_example("single empty note");
    static ref SAMPLE_ST_2_TRK_1_2: Project = load_example("sample st 2 trk 1+2");
    static ref TEST_1: Project = load_example("test 1");
    static ref BELIEVE_IT: Project = load_example("Believe It");
    static ref THE_DEMO: Project = load_example("The demo");
}

#[test]
fn it_works() {
    //dbg!(&*BLANK);
    //dbg!(&*_400BPM);
    //dbg!(&*SAMPLE_ST_2_TRK_1_2);
    //dbg!(&*BELIEVE_IT);
    //dbg!(&*THE_DEMO);
    //dbg!(&*C4_ON_1);
    //dbg!(&*TEST_1);

    // dbg!(diff_diff(&BLANK.settings.rest, &_400_BPM.settings.rest));

    // let mut buf = [0; 4];
    // LittleEndian::write_f32(&mut buf, 120.0);

    // dbg!(format!("{:02x?}", &buf));
    // LittleEndian::write_f32(&mut buf, 400.0);
    // dbg!(format!("{:02x?}", &buf));
    // BigEndian::write_f32(&mut buf, 120.0);
    // dbg!(format!("{:02x?}", &buf));
    // BigEndian::write_f32(&mut buf, 400.0);
    // dbg!(format!("{:02x?}", &buf));

    // let s = 2;
    // dbg!(LittleEndian::read_u16(&TEST_1.patterns[0].audio_track(4).rest[s..s+2]));
    // dbg!(LittleEndian::read_u16(&TEST_1.patterns[0].audio_track(5).rest[s..s+2]));
    // dbg!(LittleEndian::read_u16(&TEST_1.patterns[0].audio_track(6).rest[s..s+2]));
    // dbg!(LittleEndian::read_u16(&TEST_1.patterns[0].audio_track(7).rest[s..s+2]));
}

#[test]
fn test_bpm() {
    assert_eq!(BLANK.settings.bpm, 120.0);
    assert_eq!(_400BPM.settings.bpm, 400.0);
    assert_eq!(BELIEVE_IT.settings.bpm, 162.0);
}

#[test]
fn test_names() {
    assert_eq!(&BLANK.settings.name, "blank");
    assert_eq!(&_400BPM.settings.name, "400 bpm");
    assert_eq!(&BELIEVE_IT.settings.name, "Believe It");
    assert_eq!(&BLANK.settings.directory, "/Projects");
    assert_eq!(&_400BPM.settings.directory, "/Projects");
}

#[test]
fn test_midi_cc_mapping() {
    assert_eq!(BLANK.settings.jack_cc_mapping.len(), 16);
    assert_eq!(BLANK.settings.usb_cc_mapping.len(), 16);
    assert_eq!(BLANK.settings.jack_cc_mapping[0].cutoff, 74);
}

#[test]
fn test_steps_mapping() {
    let pat = &TEST_1.patterns[0];
    assert_eq!(pat.audio_track(0).steps[0].note, 60);
    assert_eq!(pat.audio_track(1).steps[1].note, 119);
    assert_eq!(pat.audio_track(1).steps[2].note, 12);

    assert_eq!(pat.audio_track(0).steps[0].sample, 0);
    assert_eq!(pat.audio_track(1).steps[0].sample, 1);
    assert_eq!(pat.audio_track(0).steps[0].sample_start, 0);
    assert_eq!(pat.audio_track(0).steps[0].sample_end, 0x7FFF);
    assert_eq!(pat.audio_track(2).steps[4].sample_start, 0x7FFF);
    assert_eq!(pat.audio_track(2).steps[4].sample_end, 0);
    assert_eq!(pat.audio_track(0).steps[0].sample_attack, 0);
    assert_eq!(pat.audio_track(0).steps[0].sample_decay, 0);
    assert_eq!(pat.audio_track(2).steps[5].sample_attack, 10000);
    assert_eq!(pat.audio_track(2).steps[6].sample_decay, 10000);

    assert_eq!(pat.audio_track(0).steps[0].volume, 7600);
    assert_eq!(pat.audio_track(3).steps[4].volume, 10000);
    assert_eq!(pat.audio_track(3).steps[5].volume, 0);
    assert_eq!(pat.audio_track(0).steps[0].pan, 0);
    assert_eq!(pat.audio_track(4).steps[0].pan, -10000);
    assert_eq!(pat.audio_track(4).steps[1].pan, 10000);

    assert_eq!(pat.audio_track(0).steps[0].reverb, 0);
    assert_eq!(pat.audio_track(0).steps[0].delay, 0);
    assert_eq!(pat.audio_track(3).steps[0].reverb, 10000);
    assert_eq!(pat.audio_track(3).steps[1].delay, 10000);

    assert_eq!(pat.audio_track(0).steps[0].bit_depth, 16);
    assert_eq!(pat.audio_track(0).steps[0].overdrive, 0);
    assert_eq!(pat.audio_track(5).steps[0].bit_depth, 4);
    assert_eq!(pat.audio_track(5).steps[0].overdrive, 10000);
    assert_eq!(pat.audio_track(5).steps[1].bit_depth, 8);
    assert_eq!(pat.audio_track(5).steps[1].overdrive, 8000);
}

#[test]
fn test_track_attributes() {
    assert_eq!(TEST_1.patterns[0].audio_track(0).steps.len(), 16);
    assert_eq!(TEST_1.patterns[0].audio_track(4).steps.len(), 12);

    assert_eq!(TEST_1.patterns[0].audio_track(0).play_mode, 0);
    assert_eq!(TEST_1.patterns[0].audio_track(4).play_mode, 5);
    assert_eq!(TEST_1.patterns[0].audio_track(5).play_mode, 1);

    assert_eq!(TEST_1.patterns[0].audio_track(0).swing, 50);
    assert_eq!(TEST_1.patterns[0].audio_track(5).swing, 25);
    assert_eq!(TEST_1.patterns[0].audio_track(6).swing, 75);

    assert_eq!(TEST_1.patterns[0].audio_track(0).track_speed, TrackSpeed::Fraction(1, 1));
    assert_eq!(TEST_1.patterns[0].audio_track(5).track_speed, TrackSpeed::Fraction(8, 1));
    assert_eq!(TEST_1.patterns[0].audio_track(6).track_speed, TrackSpeed::Paused);
    assert_eq!(TEST_1.patterns[0].audio_track(7).track_speed, TrackSpeed::Fraction(1, 16));
}

#[test]
fn test_variations() {
    assert!(BELIEVE_IT.patterns[0].audio_tracks[0][0].is_some());
    assert!(BELIEVE_IT.patterns[0].audio_tracks[0][1].is_some());
    assert!(BELIEVE_IT.patterns[0].audio_tracks[0][2].is_some());
    assert!(BELIEVE_IT.patterns[0].audio_tracks[2][0].is_some());
    assert!(BELIEVE_IT.patterns[0].audio_tracks[2][1].is_none());
    assert!(BELIEVE_IT.patterns[0].audio_tracks[2][2].is_some());
}
