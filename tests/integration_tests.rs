use std::path::Path;

use play_files::*;

// use slice_diff_patch::*;
// use byteorder::{ByteOrder, LittleEndian, BigEndian};
use lazy_static::lazy_static;

#[allow(dead_code)]
struct ExampleProjects {
    blank: Project,
    _400bpm: Project,
    c4_on_1: Project,
    empty_notes_on_1_3: Project,
    single_empty_note: Project,
    sample_st_2_trk_1_2: Project,
    test_1: Project,
    believe_it: Project,
    the_demo: Project,
}
impl ExampleProjects {
    const BLANK: &str = "blank";
    const _400BPM: &str = "400 bpm";
    const C4_ON_1: &str = "c4 on 1";
    const EMPTY_NOTES_ON_1_3: &str = "empty notes on 1+3";
    const SINGLE_EMPTY_NOTE: &str = "single empty note";
    const SAMPLE_ST_2_TRK_1_2: &str = "sample st 2 trk 1+2";
    const TEST_1: &str = "test 1";
    const BELIEVE_IT: &str = "Believe It";
    const THE_DEMO: &str = "The demo";

    fn load() -> Self {
        Self {
            blank: Self::project(Self::BLANK),
            _400bpm: Self::project(Self::_400BPM),
            c4_on_1: Self::project(Self::C4_ON_1),
            empty_notes_on_1_3: Self::project(Self::EMPTY_NOTES_ON_1_3),
            single_empty_note: Self::project(Self::SINGLE_EMPTY_NOTE),
            sample_st_2_trk_1_2: Self::project(Self::SAMPLE_ST_2_TRK_1_2),
            test_1: Self::project(Self::TEST_1),
            believe_it: Self::project(Self::BELIEVE_IT),
            the_demo: Self::project(Self::THE_DEMO),
        }
    }

    fn project(name: &str) -> Project {
        Project::read(Path::new(&format!("./examples/projects/{}/", name))).unwrap()
    }
}

lazy_static! {
    static ref PROJECTS: ExampleProjects = ExampleProjects::load();
}

#[test]
fn it_works() {
    // dbg!(&PROJECTS.blank);
    //dbg!(&PROJECTS._400bpm);
    // dbg!(&PROJECTS.sample_st_2_trk_1_2);
    //dbg!(&PROJECTS.believe_it);
    //dbg!(&PROJECTS.the_demo);
    dbg!(&PROJECTS.test_1);

    // dbg!(diff_diff(&blank.settings.rest, &_400_bpm.settings.rest));

    // let mut buf = [0; 4];
    // LittleEndian::write_f32(&mut buf, 120.0);

    // dbg!(format!("{:02x?}", &buf));
    // LittleEndian::write_f32(&mut buf, 400.0);
    // dbg!(format!("{:02x?}", &buf));
    // BigEndian::write_f32(&mut buf, 120.0);
    // dbg!(format!("{:02x?}", &buf));
    // BigEndian::write_f32(&mut buf, 400.0);
    // dbg!(format!("{:02x?}", &buf));
}

#[test]
fn test_bpm() {
    assert_eq!(PROJECTS.blank.settings.bpm, 120.0);
    assert_eq!(PROJECTS._400bpm.settings.bpm, 400.0);
    assert_eq!(PROJECTS.believe_it.settings.bpm, 162.0);
}

#[test]
fn test_names() {
    assert_eq!(&PROJECTS.blank.settings.name, "blank");
    assert_eq!(&PROJECTS._400bpm.settings.name, "400 bpm");
    assert_eq!(&PROJECTS.believe_it.settings.name, "Believe It");
    assert_eq!(&PROJECTS.blank.settings.directory, "/Projects");
    assert_eq!(&PROJECTS._400bpm.settings.directory, "/Projects");
}

#[test]
fn test_midi_cc_mapping() {
    assert_eq!(PROJECTS.blank.settings.jack_cc_mapping.len(), 16);
    assert_eq!(PROJECTS.blank.settings.usb_cc_mapping.len(), 16);
    assert_eq!(PROJECTS.blank.settings.jack_cc_mapping[0].cutoff, 74);
}

#[test]
fn test_steps_mapping() {
    let pat = &PROJECTS.test_1.patterns[0];
    assert_eq!(pat.audio_tracks[0].steps[0].note, 60);
    assert_eq!(pat.audio_tracks[1].steps[1].note, 119);
    assert_eq!(pat.audio_tracks[1].steps[2].note, 12);

    assert_eq!(pat.audio_tracks[0].steps[0].sample, 0);
    assert_eq!(pat.audio_tracks[1].steps[0].sample, 1);
    assert_eq!(pat.audio_tracks[0].steps[0].sample_start, 0);
    assert_eq!(pat.audio_tracks[0].steps[0].sample_end, 0x7FFF);
    assert_eq!(pat.audio_tracks[2].steps[4].sample_start, 0x7FFF);
    assert_eq!(pat.audio_tracks[2].steps[4].sample_end, 0);
    assert_eq!(pat.audio_tracks[0].steps[0].sample_attack, 0);
    assert_eq!(pat.audio_tracks[0].steps[0].sample_decay, 0);
    assert_eq!(pat.audio_tracks[2].steps[5].sample_attack, 10000);
    assert_eq!(pat.audio_tracks[2].steps[6].sample_decay, 10000);

    assert_eq!(pat.audio_tracks[0].steps[0].volume, 7600);
    assert_eq!(pat.audio_tracks[3].steps[4].volume, 10000);
    assert_eq!(pat.audio_tracks[3].steps[5].volume, 0);
    assert_eq!(pat.audio_tracks[0].steps[0].pan, 0);
    assert_eq!(pat.audio_tracks[4].steps[0].pan, -10000);
    assert_eq!(pat.audio_tracks[4].steps[1].pan, 10000);

    assert_eq!(pat.audio_tracks[0].steps[0].reverb, 0);
    assert_eq!(pat.audio_tracks[0].steps[0].delay, 0);
    assert_eq!(pat.audio_tracks[3].steps[0].reverb, 10000);
    assert_eq!(pat.audio_tracks[3].steps[1].delay, 10000);

    assert_eq!(pat.audio_tracks[0].steps[0].bit_depth, 16);
    assert_eq!(pat.audio_tracks[0].steps[0].overdrive, 0);
    assert_eq!(pat.audio_tracks[5].steps[0].bit_depth, 4);
    assert_eq!(pat.audio_tracks[5].steps[0].overdrive, 10000);
    assert_eq!(pat.audio_tracks[5].steps[1].bit_depth, 8);
    assert_eq!(pat.audio_tracks[5].steps[1].overdrive, 8000);
}
