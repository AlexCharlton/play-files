use play_files::*;
use std::path::Path;
use slice_diff_patch::*;

use byteorder::{ByteOrder, LittleEndian, BigEndian};
use lazy_static::lazy_static;

struct ExampleProjects {
    blank: Project,
    _400bpm: Project,
    c4_on_1: Project,
    empty_notes_on_1_3: Project,
    single_empty_note: Project,
    believe_it: Project,
    the_demo: Project,
}
impl ExampleProjects {
    const _400BPM: &str = "400 bpm";
    const BELIEVE_IT: &str = "Believe It";
    const BLANK: &str = "blank";
    const C4_ON_1: &str = "c4 on 1";
    const EMPTY_NOTES_ON_1_3: &str = "empty notes on 1+3";
    const SINGLE_EMPTY_NOTE: &str = "single empty note";
    const THE_DEMO: &str = "The demo";

    fn load() -> Self {
        Self {
            blank: Self::project(Self::BLANK),
            _400bpm: Self::project(Self::_400BPM),
            c4_on_1: Self::project(Self::C4_ON_1),
            empty_notes_on_1_3: Self::project(Self::EMPTY_NOTES_ON_1_3),
            single_empty_note: Self::project(Self::SINGLE_EMPTY_NOTE),
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
    dbg!(&PROJECTS.blank);
    dbg!(&PROJECTS._400bpm);

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

    // dbg!(project(C4_ON_1));
    // dbg!(project(EMPTY_NOTES_ON_1_3));
    // dbg!(project(SINGLE_EMPTY_NOTE));
    // dbg!(project(BELIEVE_IT));
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
