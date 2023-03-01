use std::cell::RefCell;
#[allow(dead_code)]
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use arr_macro::arr;
use byteorder::{ByteOrder, LittleEndian};
use glob::glob;
use regex::Regex;

#[derive(PartialEq)]
pub struct ParseError(String);

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", &self.0)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", &self.0)
    }
}

impl std::error::Error for ParseError {}

type Result<T> = std::result::Result<T, ParseError>;

struct Reader {
    buffer: Vec<u8>,
    position: Rc<RefCell<usize>>,
}

#[allow(dead_code)]
impl Reader {
    fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            position: Rc::new(RefCell::new(0)),
        }
    }

    fn read(&self) -> u8 {
        let p: usize = *self.position.borrow();
        let b = self.buffer[p];
        *self.position.borrow_mut() += 1;
        b
    }

    fn read_bytes(&self, n: usize) -> &[u8] {
        let p: usize = *self.position.borrow();
        let bs = &self.buffer[p..p + n];
        *self.position.borrow_mut() += n;
        bs
    }

    fn read_bool(&self) -> bool {
        self.read() == 1
    }

    fn read_string(&self, n: usize) -> String {
        let b = self.read_bytes(n);
        std::str::from_utf8(b)
            .expect("invalid utf-8 sequence in string")
            .to_string()
    }

    fn read_variable_quantity(&self) -> usize {
        let mut bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            let b = self.read();
            bytes[i] = b & 0b01111111;
            if b & 0b10000000 == 0 {
                break;
            }
            // If we're in our last loop, we shouldn't make it this far:
            if i == 3 {
                panic!("More bytes than expected in a variable quantity")
            }
        }

        bytes
            .iter()
            .enumerate()
            .fold(0, |r, (i, &b)| r + ((b as usize) << (i * 7)))
    }

    fn pos(&self) -> usize {
        *self.position.borrow()
    }

    fn set_pos(&self, n: usize) {
        *self.position.borrow_mut() = n;
    }

    fn step_back(&self) {
        *self.position.borrow_mut() -= 1;
    }

    fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    fn rest(&self) -> Vec<u8> {
        let p: usize = *self.position.borrow();
        self.buffer[p..].to_vec()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Project {
    pub settings: Settings,
    pub samples: Samples,
    pub patterns: Vec<Pattern>,
}

impl Project {
    pub fn read(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            return Err(ParseError(format!(
                "Provided project dir {:?} is not a directory",
                &path
            )));
        }
        let settings = Settings::read(&path.join("settings"))?;
        let samples = Samples::read(&path.join("samples").join("samplesMetadata"))?;
        let patterns = Pattern::read_patterns(&path.join("patterns"))?;

        Ok(Self {
            settings,
            samples,
            patterns,
        })
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct Settings {
    pub name: String,
    pub directory: String,
    pub bpm: f32,
    pub jack_cc_mapping: Vec<CCMapping>,
    pub usb_cc_mapping: Vec<CCMapping>,
    // TODO
    pub x20: Vec<u8>, // Unknown 5 bytes
    pub xa8: Vec<u8>, // Unknown 2 bytes
    pub xb0: Vec<u8>, // Unknown 2 bytes
    pub x90: Vec<u8>, // Unknown 11? bytes
}

impl Settings {
    pub fn read(path: &Path) -> Result<Self> {
        let mut file =
            File::open(path).map_err(|_| ParseError(format!("No settings file present")))?;

        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let mut attrs = Self::default();
        Self::attrs_from_reader(&reader, &mut attrs)?;

        attrs.jack_cc_mapping = (0..16)
            .map(|_| CCMapping::from_reader(&reader))
            .collect::<Result<Vec<CCMapping>>>()?;
        attrs.usb_cc_mapping = (0..16)
            .map(|_| CCMapping::from_reader(&reader))
            .collect::<Result<Vec<CCMapping>>>()?;

        Ok(attrs)
    }

    fn attrs_from_reader(reader: &Reader, settings: &mut Self) -> Result<()> {
        let mut tag = reader.read();
        while tag != 0xc2 {
            // Elements in the CCMapping begin with 0xC2
            match tag {
                // name
                0x12 => settings.name = reader.read_string(reader.read() as usize),
                // directory
                0x62 => settings.directory = reader.read_string(reader.read() as usize),
                // bpm
                0x85 => {
                    reader.read(); // TODO Unknown byte (always 01?)
                    settings.bpm = LittleEndian::read_f32(reader.read_bytes(4));
                }
                // unknowns TODO
                0x20 => settings.x20 = reader.read_bytes(5).to_vec(),
                0x90 => settings.x90 = reader.read_bytes(11).to_vec(),
                0xA8 => settings.xa8 = reader.read_bytes(2).to_vec(),
                0xB0 => settings.xb0 = reader.read_bytes(2).to_vec(),
                t => panic!("Unknown tag: {}", t),
            }
            tag = reader.read();
        }
        reader.step_back(); // Replace the last 0xC2
        Ok(())
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("name", &self.name)
            .field("directory", &self.directory)
            .field("bpm", &self.bpm)
            // .field("x20", &self.x20)
            // .field("xa8", &self.xa8)
            // .field("xb0", &self.xb0)
            // .field("x90", &self.x90)
            .finish()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct CCMapping {
    pub u_first_bytes: [u8; 4], // TODO
    pub cutoff: u8,
    pub resonance: u8,
    pub sample_attack: u8,
    pub sample_decay: u8,
    pub reverb_send: u8,
    pub delay_send: u8,
    pub overdrive: u8,
    pub bit_depth: u8,
}

impl CCMapping {
    fn from_reader(reader: &Reader) -> Result<Self> {
        assert_eq!(reader.read(), 0xC2); // First byte probably tag (0xC2)
        Ok(Self {
            u_first_bytes: reader.read_bytes(4).try_into().unwrap(),
            cutoff: reader.read(),
            resonance: reader.read(),
            sample_attack: reader.read(),
            sample_decay: reader.read(),
            reverb_send: reader.read(),
            delay_send: reader.read(),
            overdrive: reader.read(),
            bit_depth: reader.read(),
        })
    }
}

#[derive(PartialEq, Clone)]
pub struct Samples {
    pub rest: Vec<u8>, // TODO
}
impl Samples {
    pub fn read(path: &Path) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("Cannot read sample file: {:?}", &path)))?;

        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let rest = reader.rest();

        Ok(Self { rest })
    }
}

impl fmt::Debug for Samples {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Samples")
            // .field(
            //     "rest",
            //     &format!(
            //         "{} bytes: {:?}...",
            //         self.rest.len(),
            //         &&self.rest[0..10.min(self.rest.len())]
            //     ),
            // )
            .finish()
    }
}


type TrackVariations = [Option<Track>; 16];

#[derive(PartialEq, Clone)]
pub struct Pattern {
    pub number: u8,
    pub audio_tracks: [TrackVariations; 8],
    pub midi_tracks: [TrackVariations; 8],
    pub rest: Vec<u8>, // TODO
}
impl Pattern {
    /// Read a pattern directory
    pub fn read_patterns(path: &Path) -> Result<Vec<Self>> {
        if !path.is_dir() {
            return Err(ParseError(format!(
                "Provided patterns dir {:?} is not a directory",
                &path
            )));
        }

        let mut patterns = vec![];
        for entry in glob(&format!("{}/*.pattern", path.to_str().unwrap()))
            .map_err(|_| ParseError(format!("Could not read pattern dir")))?
        {
            match entry {
                Ok(path) => {
                    let re = Regex::new(r"(\d+).pattern$").unwrap();
                    let pattern_number = if let Some(n) = re.captures(path.to_str().unwrap()) {
                        n.get(1).unwrap().as_str().parse().map_err(|_| {
                            ParseError(format!("Invalid pattern file name: {:?}", &path))
                        })?
                    } else {
                        return Err(ParseError(format!(
                            "Invalid pattern file name: {:?}",
                            &path
                        )));
                    };
                    patterns.push(Self::read(&path, pattern_number)?);
                }
                _ => return Err(ParseError(format!("Could not read pattern dir"))),
            }
        }

        Ok(patterns)
    }

    /// Read a particular pattern file. Will also read any track files that match the pattern number
    pub fn read(path: &Path, number: u8) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("Cannot read pattern file: {:?}", &path)))?;

        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let mut audio_tracks = arr![arr![None; 16]; 8];
        let mut midi_tracks = arr![arr![None; 16]; 8];
        // dbg!(path);
        for track in 0..8 {
            audio_tracks[track][0] = Some(Track::from_reader(&reader, track, 0, TrackType::Audio, false)?);
        }
        for track in 0..8 {
            midi_tracks[track][0] = Some(Track::from_reader(&reader, track, 0, TrackType::Midi, false)?);
        }

        let rest = reader.rest();

        for mut variation in Self::read_variations(&path.parent().unwrap(), number)? {
            let track = variation.number;
            let v = variation.variation;
            if track < 8 {
                audio_tracks[track][v] = Some(variation);
            } else {
                variation.number -= 8;
                midi_tracks[track - 8][v] = Some(variation);
            }
        }

        Ok(Self {
            number,
            audio_tracks: audio_tracks.try_into().unwrap(),
            midi_tracks: midi_tracks.try_into().unwrap(),
            rest,
        })
    }

    fn read_variations(path: &Path, pattern_number: u8) -> Result<Vec<Track>> {
        let mut ret: Vec<Track> = vec![];
        for track in 0..8 {
            let mut track_files = glob(&format!(
                "{}/{}-{}-*.track",
                path.to_str().unwrap(),
                pattern_number,
                track
            )).map_err(|_| ParseError(format!("Could not read track dir")))?;

            if track_files.next().is_some() {

                for variation in 0..16 {
                    let track_path = path.join(&format!(
                        "{}-{}-{}.track",
                        pattern_number, track, variation
                    ));
                    if track_path.is_file() {
                        ret.push(Track::read(&track_path, track, variation)?);
                    }
                }
            }
        }
        Ok(ret)
    }

    /// Get the first variation of a track
    pub fn audio_track(&self, n: usize) -> &Track {
        self.audio_tracks[n][0].as_ref().unwrap()
    }
}

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pattern")
            .field("number", &self.number)
            .field("audio_tracks", &self.audio_tracks)
            .field("midi_tracks", &self.midi_tracks)
            // .field(
            //     "rest",
            //     &format!(
            //         "{} bytes: {:?}...",
            //         self.rest.len(),
            //         &&self.rest[0..10.min(self.rest.len())]
            //     ),
            // )
            .finish()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TrackType {
    Audio,
    Midi,
}

#[derive(PartialEq, Clone)]
pub struct Track {
    pub ty: TrackType,
    pub number: usize,
    pub variation: usize,
    pub steps: Vec<Step>,
    // Percentage 25-75
    pub swing: u8,
    pub play_mode: u8,
    pub track_speed: TrackSpeed,
    attrs: TrackAttrs,
}

impl Track {
    fn from_reader(reader: &Reader, number: usize, variation: usize, ty: TrackType, from_file: bool) -> Result<Self> {
        let track_len = {
            if from_file {
                reader.buffer_len()
            } else {
                assert_eq!(reader.read(), 0x0A); // First tag (0x0A)
                reader.read_variable_quantity()
            }
        };
        // println!("Reading track {:?} {} with len {}", ty, number, track_len);

        let start_pos = reader.pos();
        let steps = (0..64)
            .map(|step| Step::from_reader(reader, step))
            .collect::<Result<Vec<Step>>>()?;

        let attrs = TrackAttrs::from_reader(reader, track_len + start_pos)?;
        assert!(attrs.num_steps > 0 && attrs.num_steps < 65);

        let bytes_advanced = reader.pos() - start_pos;
        assert_eq!(bytes_advanced, track_len);
        Ok(Self {
            ty,
            number,
            variation,
            steps: steps[0..(attrs.num_steps as usize)].try_into().unwrap(),
            swing: attrs.swing,
            play_mode: attrs.play_mode,
            track_speed: attrs.track_speed,
            attrs,
        })
    }

    pub fn read(path: &Path, track_number: usize, variation_number: usize) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("Cannot read track file: {:?}", &path)))?;

        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let track_type = if track_number < 8 { TrackType::Audio } else { TrackType::Midi };
        Track::from_reader(&reader, track_number, variation_number, track_type, true)
    }
}

impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Track")
            .field("type", &self.ty)
            .field("number", &self.number)
            .field("variation", &self.variation)
            .field("steps", &self.steps)
            .field("swing", &self.swing)
            .field("track_speed", &self.track_speed)
            .field("play_mode", &self.play_mode)
            // .field("attrs", &self.attrs)
            .finish()
    }
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct TrackAttrs {
    num_steps: u8,
    // Percentage 25-75
    swing: u8,
    play_mode: u8,
    track_speed: TrackSpeed,
    // TODO, unknown values:
    ux18: u8,
    ux30: u8,
    ux4a: Vec<u8>,
}

impl TrackAttrs {
    fn from_reader(reader: &Reader, max_pos: usize) -> Result<Self> {
        let mut attrs = TrackAttrs::default();

        while reader.pos() < max_pos {
            let val = reader.read();
            match val {
                0x10 => attrs.num_steps = reader.read(),
                0x38 => attrs.swing = reader.read(),
                0x40 => attrs.play_mode = reader.read(),
                0x20 => {
                    if let TrackSpeed::Fraction(_, d) = attrs.track_speed {
                        attrs.track_speed = match reader.read() {
                            0 => TrackSpeed::Paused,
                            n => TrackSpeed::Fraction(n, d)
                        };
                    } else {
                        attrs.track_speed = TrackSpeed::Fraction(reader.read(), 1)
                    }
                },
                0x28 => {
                    if let TrackSpeed::Fraction(n, _) = attrs.track_speed {
                        attrs.track_speed = match reader.read() {
                            0 => TrackSpeed::Paused,
                            d => TrackSpeed::Fraction(n, d)
                        };
                    } else {
                        reader.read(); // discard
                    }
                }
                0x18 => attrs.ux18 = reader.read(),
                0x30 => attrs.ux30 = reader.read(),
                0x4a => {
                    let len = reader.read();
                    attrs.ux4a = reader.read_bytes(len as usize).to_vec();
                },
                x => {
                    reader.read(); // Discard
                    println!("Warning: encountered unknown track tag {:02X}", x);
                },
            }
        }

        Ok(attrs)
    }
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum TrackSpeed {
    /// Numerator, Denominator
    Fraction(u8, u8),
    Paused,
}

impl Default for TrackSpeed {
    fn default() -> Self {
        Self::Paused
    }
}

#[derive(PartialEq, Clone)]
pub struct Step {
    /// Step number, 0 indexed
    pub number: usize,
    /// Sample number
    pub sample: u16,
    /// Midi note number
    pub note: u8,
    /// 0db at 7600; 200 = 1db
    pub volume: u16,
    /// -10000 is hard L, 10000 is hard right; 100 = 1%
    pub pan: i16,
    /// -10000 is LP100; 10000 is HP100; 100 = 1%
    pub filter_cutoff: i16,
    /// 10000 is 100%; 100 = 1%
    pub filter_resonance: u16,
    /// 10000 is 100%; 100 = 1%
    pub overdrive: u16,
    /// 4-16
    pub bit_depth: u8,
    /// -10000 is -11/24; 10000 is +11/24
    pub micro_move: i16,
    /// 10000 is 100%; 100 = 1%
    pub reverb: i16,
    pub delay: i16,
    /// 0: start of sample; 32767: end of sample
    pub sample_start: i16,
    pub sample_end: i16,
    /// 10000 is 100%; 100 = 1%
    pub sample_attack: u16,
    pub sample_decay: u16,
    /// Used for display/randomize only; 0xFFFF = All samples
    pub sample_folder: u16,
    /// 0 = Off
    pub repeat_type: u16,
    pub repeat_grid: u16,
    /// 0 = Always
    pub chance_type: u16,
    /// 0 = Play Step
    pub chance_action: u16,
    /// -10000 is -100 cents; 10000 is +100 cents; 100 = 1 cent
    pub micro_tune: i16,

    pub rest: Vec<u8>, // TODO
}

impl Step {
    fn from_reader(reader: &Reader, number: usize) -> Result<Self> {
        assert_eq!(reader.read(), 0x0A, "Error reading {}nth step", number); // first byte tag (0x0A)
        let len = reader.read_variable_quantity(); // Length of step data

        let start_pos = reader.pos();
        // println!("{}nth step, length {} ({:02x})", number, len, len);
        assert_eq!(reader.read(), 0x0A); // Second tag (0x0A)
        let num_elements = reader.read_variable_quantity(); // Length of step data
        assert_eq!(num_elements, 44); // I've never seen a value that's not 44

        let volume = LittleEndian::read_u16(reader.read_bytes(2));
        let pan = LittleEndian::read_i16(reader.read_bytes(2));
        let filter_cutoff = LittleEndian::read_i16(reader.read_bytes(2));
        let filter_resonance = LittleEndian::read_u16(reader.read_bytes(2));
        let bit_depth = LittleEndian::read_u16(reader.read_bytes(2)) as u8;
        let overdrive = LittleEndian::read_u16(reader.read_bytes(2));
        let note = LittleEndian::read_u16(reader.read_bytes(2)) as u8;
        let delay = LittleEndian::read_i16(reader.read_bytes(2));
        let reverb = LittleEndian::read_i16(reader.read_bytes(2));
        let sample = LittleEndian::read_u16(reader.read_bytes(2));
        let sample_start = LittleEndian::read_i16(reader.read_bytes(2));
        let sample_end = LittleEndian::read_i16(reader.read_bytes(2));
        let micro_tune = LittleEndian::read_i16(reader.read_bytes(2));
        let sample_attack = LittleEndian::read_u16(reader.read_bytes(2));
        let sample_decay = LittleEndian::read_u16(reader.read_bytes(2));
        let sample_folder = LittleEndian::read_u16(reader.read_bytes(2));
        let repeat_type = LittleEndian::read_u16(reader.read_bytes(2));
        let repeat_grid = LittleEndian::read_u16(reader.read_bytes(2));
        let chance_type = LittleEndian::read_u16(reader.read_bytes(2));
        let chance_action = LittleEndian::read_u16(reader.read_bytes(2));
        let micro_move = LittleEndian::read_i16(reader.read_bytes(2));

        let bytes_advanced = reader.pos() - start_pos;
        // Rest appears to be always nothing for empty notes and a fixed set of 6 bytes otherwise
        let rest = reader.read_bytes(len - bytes_advanced); // Unknown data
        Ok(Self {
            number,
            sample,
            note,
            volume,
            pan,
            filter_cutoff,
            filter_resonance,
            micro_move,
            micro_tune,
            sample_start,
            sample_end,
            sample_attack,
            sample_decay,
            sample_folder,
            repeat_type,
            repeat_grid,
            chance_type,
            chance_action,
            reverb,
            delay,
            overdrive,
            bit_depth,
            rest: rest.to_vec(),
        })
    }
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // f.debug_struct("Step")
        //     .field("number", &self.number)
        //     .field("volume", &self.volume)
        //     .field("note", &self.note)
        //     .field("sample", &self.sample)
        //     .field("sample_start", &self.sample_start)
        //     .field("sample_end", &self.sample_end)
        //     .field("sample_attack", &self.sample_attack)
        //     .field("sample_decay", &self.sample_decay)
        //     .field("pan", &self.pan)
        //     .field("filter_cutoff", &self.filter_cutoff)
        //     .field("filter_resonance", &self.filter_resonance)
        //     .field("micro_move", &self.micro_move)
        //     .field("micro_tune", &self.micro_tune)
        //     .field("repeat_type", &self.repeat_type)
        //     .field("repeat_grid", &self.repeat_grid)
        //     .field("chance_type", &self.chance_type)
        //     .field("chance_action", &self.chance_action)
        //     .field("reverb", &self.reverb)
        //     .field("delay", &self.delay)
        //     .field("overdrive", &self.overdrive)
        //     .field("bit_depth", &self.bit_depth)
        //     .finish()

        // Alternate, compact format
        write!(f, "Step {}: volume({}) note({}) sample({}) start/end({}-{}) attack/decay({}-{}) pan({}) filter_cutoff({}) resonance({}) micromove({}) microtune({}) repeat/type-grid({}-{}) chance/type-action({}-{}) reverb/delay({}-{}) overdrive({}) bit-depth({})", //  rest: {:?} (len: {})
               self.number,
               self.volume,
               self.note,
               self.sample,
               self.sample_start,
               self.sample_end,
               self.sample_attack,
               self.sample_decay,
               self.pan,
               self.filter_cutoff,
               self.filter_resonance,
               self.micro_move,
               self.micro_tune,
               self.repeat_type,
               self.repeat_grid,
               self.chance_type,
               self.chance_action,
               self.reverb,
               self.delay,
               self.overdrive,
               self.bit_depth,
               // &self.rest, self.rest.len()
        )
    }
}
