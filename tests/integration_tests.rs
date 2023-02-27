use play_files::*;
use std::path::Path;
use slice_diff_patch::*;

use byteorder::{ByteOrder, LittleEndian, BigEndian};

const _400BPM: &str = "400 bpm";
const BELIEVE_IT: &str = "Believe It";
const BLANK: &str = "blank";
const C4_ON_1: &str = "c4 on 1";
const EMPTY_NOTES_ON_1_3: &str = "empty notes on 1+3";
const SINGLE_EMPTY_NOTE: &str = "single empty note";
const THE_DEMO: &str = "The demo";

fn project(name: &str) -> Project {
    Project::read(Path::new(&format!("./examples/projects/{}/", name))).unwrap()
}

#[test]
fn it_works() {
    let blank = project(BLANK);
    let _400_bpm = project(_400BPM);
    dbg!(&blank);
    dbg!(&_400_bpm);

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
    dbg!(project(THE_DEMO).settings);
    assert_eq!(4, 4);
}
