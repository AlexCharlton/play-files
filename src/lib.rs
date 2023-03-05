#[allow(dead_code)]
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use arr_macro::arr;
use byteorder::{ByteOrder, LittleEndian};
use glob::glob;
use regex::Regex;

mod reader;
use reader::Reader;

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

type AudioTrackVariations = [Option<Track<Step>>; 16];
type MidiTrackVariations = [Option<Track<MidiStep>>; 16];

#[derive(PartialEq, Clone)]
pub struct Pattern {
    pub number: u8,
    pub audio_tracks: [AudioTrackVariations; 8],
    pub midi_tracks: [MidiTrackVariations; 8],
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
            let t = Track::from_reader(&reader, track, 0, false)?;
            let v = t.variation;
            audio_tracks[track][v] = Some(t);
        }
        for track in 0..8 {
            let t = Track::from_reader(&reader, track, 0, false)?;
            let v = t.variation;
            midi_tracks[track][v] = Some(t);
        }

        let rest = reader.rest();

        let (audio_variation, midi_variation) =
            Self::read_variations(&path.parent().unwrap(), number)?;
        for variation in audio_variation {
            let track = variation.number;
            let v = variation.variation;
            if audio_tracks[track][v].is_some() {
                continue;
            }
            audio_tracks[track][v] = Some(variation);
        }
        for variation in midi_variation {
            let track = variation.number;
            let v = variation.variation;
            if midi_tracks[track][v].is_some() {
                continue;
            }
            midi_tracks[track][v] = Some(variation);
        }

        Ok(Self {
            number,
            audio_tracks: audio_tracks.try_into().unwrap(),
            midi_tracks: midi_tracks.try_into().unwrap(),
            rest,
        })
    }

    fn read_variations(
        path: &Path,
        pattern_number: u8,
    ) -> Result<(Vec<Track<Step>>, Vec<Track<MidiStep>>)> {
        let mut audio: Vec<Track<Step>> = vec![];
        let mut midi: Vec<Track<MidiStep>> = vec![];
        for track in 0..16 {
            let mut track_files = glob(&format!(
                "{}/{}-{}-*.track",
                path.to_str().unwrap(),
                pattern_number,
                track
            ))
            .map_err(|_| ParseError(format!("Could not read track dir")))?;

            if track_files.next().is_some() {
                for variation in 0..16 {
                    let track_path =
                        path.join(&format!("{}-{}-{}.track", pattern_number, track, variation));
                    if track_path.is_file() {
                        if track < 8 {
                            audio.push(Track::read(&track_path, track, variation)?);
                        } else {
                            midi.push(Track::read(&track_path, track - 8, variation)?);
                        }
                    }
                }
            }
        }
        Ok((audio, midi))
    }

    /// Get the first variation of a track
    pub fn audio_track(&self, n: usize) -> &Track<Step> {
        self.audio_tracks[n][0].as_ref().unwrap()
    }

    /// Get the first variation of a track
    pub fn midi_track(&self, n: usize) -> &Track<MidiStep> {
        self.midi_tracks[n][0].as_ref().unwrap()
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

#[derive(PartialEq, Clone)]
pub struct Track<S: TrackStep + Clone> {
    pub number: usize,
    pub variation: usize,
    pub steps: Vec<S>,
    // Percentage 25-75
    pub swing: u8,
    pub play_mode: u8,
    pub track_speed: TrackSpeed,
    pub is_default: bool,
    attrs: TrackAttrs,
}

impl<S: TrackStep + Clone> Track<S> {
    fn from_reader(
        reader: &Reader,
        number: usize,
        variation: usize,
        from_file: bool,
    ) -> Result<Self> {
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
            .map(|step| S::from_reader(reader, step))
            .collect::<Result<Vec<S>>>()?;

        let attrs = TrackAttrs::from_reader(reader, track_len + start_pos)?;
        assert!(attrs.num_steps > 0 && attrs.num_steps < 65);

        let bytes_advanced = reader.pos() - start_pos;
        assert_eq!(bytes_advanced, track_len);
        Ok(Self {
            number,
            variation: if from_file {
                variation
            } else {
                attrs.variation as usize
            },
            steps: steps[0..(attrs.num_steps as usize)].try_into().unwrap(),
            swing: attrs.swing,
            play_mode: attrs.play_mode,
            track_speed: attrs.track_speed,
            is_default: !from_file,
            attrs,
        })
    }

    pub fn read(path: &Path, track_number: usize, variation_number: usize) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("Cannot read track file: {:?}", &path)))?;

        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        Self::from_reader(&reader, track_number, variation_number, true)
    }
}

impl<S: TrackStep + Clone + fmt::Debug> fmt::Debug for Track<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Track")
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
    // This is the default variation when the track was saved
    variation: u8,
    // This is a map of what variations existed when this track was saved. Not sure why it exists.
    variations: Vec<bool>,
    // TODO, unknown value:
    ux18: u8,
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
                            n => TrackSpeed::Fraction(n, d),
                        };
                    } else {
                        attrs.track_speed = TrackSpeed::Fraction(reader.read(), 1)
                    }
                }
                0x28 => {
                    if let TrackSpeed::Fraction(n, _) = attrs.track_speed {
                        attrs.track_speed = match reader.read() {
                            0 => TrackSpeed::Paused,
                            d => TrackSpeed::Fraction(n, d),
                        };
                    } else {
                        reader.read(); // discard
                    }
                }
                0x30 => attrs.variation = reader.read(),
                0x4a => {
                    let len = reader.read();
                    attrs.variations = reader
                        .read_bytes(len as usize)
                        .iter()
                        .map(|&x| x != 0)
                        .collect();
                }
                0x18 => attrs.ux18 = reader.read(),
                x => {
                    println!(
                        "Warning: encountered unknown track tag {:02X} with value {}",
                        x,
                        reader.read()
                    );
                }
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

pub trait TrackStep {
    fn from_reader(reader: &Reader, number: usize) -> Result<Self>
    where
        Self: Sized;
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

impl TrackStep for Step {
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
        // Rest appears to be two empty bytes for empty notes and 8 bytes otherwise
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

#[derive(PartialEq, Clone)]
pub struct MidiStep {
    /// Step number, 0 indexed
    pub number: usize,
    /// Sample number
    pub channel: MidiChannel,
    pub program: Option<u8>,
    /// Midi note number
    pub note: u8,
    pub velocity: u8,
    /// Note length in 60ths of a quarter note.
    pub note_length: u16,
    // Chord enum value
    pub chord: i16,
    /// -10000 is -11/24; 10000 is +11/24
    pub micro_move: i16,
    /// -10000 is -100 cents; 10000 is +100 cents; 100 = 1 cent
    pub pitch_bend: Option<i16>,

    /// Midi CC values
    pub cc12: Option<u8>,
    pub cc13: Option<u8>,
    pub cc17: Option<u8>,
    pub cc19: Option<u8>,
    pub cc22: Option<u8>,
    pub cc71: Option<u8>,
    pub cc74: Option<u8>,
    pub cc75: Option<u8>,

    /// Used for display/randomize only; 0xFFFF = All samples
    /// 0 = Off
    pub repeat_type: u16,
    pub repeat_grid: u16,
    /// 0 = Always
    pub chance_type: u16,
    /// 0 = Play Step
    pub chance_action: u16,

    pub rest: Vec<u8>, // TODO
}

impl TrackStep for MidiStep {
    fn from_reader(reader: &Reader, number: usize) -> Result<Self> {
        assert_eq!(reader.read(), 0x0A, "Error reading {}nth step", number); // first byte tag (0x0A)
        let len = reader.read_variable_quantity(); // Length of step data

        let start_pos = reader.pos();
        // println!("{}nth step, length {} ({:02x})", number, len, len);
        assert_eq!(reader.read(), 0x0A); // Second tag (0x0A)
        let num_elements = reader.read_variable_quantity(); // Length of step data
        assert_eq!(num_elements, 44); // I've never seen a value that's not 44

        let velocity = LittleEndian::read_u16(reader.read_bytes(2)) as u8;
        let note_length = LittleEndian::read_u16(reader.read_bytes(2));
        let mut cc74 = Some(LittleEndian::read_i16(reader.read_bytes(2)) as u8);
        let mut cc71 = Some(LittleEndian::read_u16(reader.read_bytes(2)) as u8);
        let mut cc13 = Some(LittleEndian::read_u16(reader.read_bytes(2)) as u8);
        let mut cc12 = Some(LittleEndian::read_u16(reader.read_bytes(2)) as u8);
        let note = LittleEndian::read_u16(reader.read_bytes(2)) as u8;
        let mut cc19 = Some(LittleEndian::read_i16(reader.read_bytes(2)) as u8);
        let mut cc17 = Some(LittleEndian::read_i16(reader.read_bytes(2)) as u8);
        let channel = MidiChannel::from(LittleEndian::read_u16(reader.read_bytes(2)));
        let chord = LittleEndian::read_i16(reader.read_bytes(2));
        let _sample_end = LittleEndian::read_i16(reader.read_bytes(2)); // unused
        let mut pitch_bend = Some(LittleEndian::read_i16(reader.read_bytes(2)));
        let mut cc22 = Some(LittleEndian::read_u16(reader.read_bytes(2)) as u8);
        let mut cc75 = Some(LittleEndian::read_u16(reader.read_bytes(2)) as u8);
        let mut program = Some(LittleEndian::read_u16(reader.read_bytes(2)) as u8);
        let mut repeat_type = LittleEndian::read_u16(reader.read_bytes(2));
        let mut repeat_grid = LittleEndian::read_u16(reader.read_bytes(2));
        let chance_type = LittleEndian::read_u16(reader.read_bytes(2));
        let chance_action = LittleEndian::read_u16(reader.read_bytes(2));
        let micro_move = LittleEndian::read_i16(reader.read_bytes(2));

        let bytes_advanced = reader.pos() - start_pos;
        // First five bytes are unknown. Last 3 are a bitmask when the note exists
        let rest = reader.read_bytes(len - bytes_advanced);

        if rest.len() > 2 {
            let m1 = rest[5];
            let m2 = rest[6];
            let m3 = rest[7];

            // This is madness :S
            // And also not really ideal. We probably shouldn't default to 0?
            if ((m1 >> 5) & 1) == 0 {
                cc12 = None
            }
            if ((m1 >> 4) & 1) == 0 {
                cc13 = None
            }
            if ((m1 >> 3) & 1) == 0 {
                cc71 = None
            }
            if ((m1 >> 2) & 1) == 0 {
                cc74 = None
            }

            if ((m2 >> 6) & 1) == 0 {
                cc22 = None
            }
            if ((m2 >> 5) & 1) == 0 {
                pitch_bend = None
            }
            if ((m2 >> 1) & 1) == 0 {
                cc17 = None
            }
            if ((m2 >> 0) & 1) == 0 {
                cc19 = None
            }

            // This is "Off", so I don't think None is necessary
            if ((m3 >> 3) & 1) == 0 {
                repeat_grid = 0
            }
            if ((m3 >> 2) & 1) == 0 {
                repeat_type = 0
            }
            if ((m3 >> 1) & 1) == 0 {
                program = None
            }
            if ((m3 >> 0) & 1) == 0 {
                cc75 = None
            }

            // There are 10 bytes that can't be unset, and thus can't be inferred
        } else {
            cc12 = None;
            cc13 = None;
            cc17 = None;
            cc19 = None;
            cc22 = None;
            cc71 = None;
            cc74 = None;
            cc75 = None;
            program = None;
            pitch_bend = None;
        }

        Ok(Self {
            number,
            channel,
            program,
            note,
            velocity,
            note_length,
            micro_move,
            pitch_bend,
            chord,
            cc12,
            cc13,
            cc17,
            cc19,
            cc22,
            cc71,
            cc74,
            cc75,
            repeat_type,
            repeat_grid,
            chance_type,
            chance_action,
            rest: rest.to_vec(),
        })
    }
}

impl fmt::Debug for MidiStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // f.debug_struct("MidiStep")
        //     .field("number", &self.number)
        //     .field("note", &self.note)
        //     .field("velocity", &self.velocity)
        //     .field("channel", &self.channel)
        //     .field("program", &self.program)
        //     .field("note_length", &self.note_length)
        //     .field("micro_move", &self.micro_move)
        //     .field("pitch_bend", &self.bitch_bend)
        //     .field("cc12", &self.cc12)
        //     .field("cc13", &self.cc13)
        //     .field("cc17", &self.cc17)
        //     .field("cc19", &self.cc19)
        //     .field("cc22", &self.cc22)
        //     .field("cc71", &self.cc71)
        //     .field("cc74", &self.cc74)
        //     .field("cc75", &self.cc75)
        //     .field("repeat_type", &self.repeat_type)
        //     .field("repeat_grid", &self.repeat_grid)
        //     .field("chance_type", &self.chance_type)
        //     .field("chance_action", &self.chance_action)
        //     .finish()

        // Alternate, compact format
        write!(f, "MidiStep {}: note({}) velocity({}) channel({:?}) program({:?}) note_length({}) micromove({}) pitch_bend({:?}) CC(12:{:?}|13:{:?}|17:{:?}|19:{:?}|22:{:?}|71:{:?}|74:{:?}|75:{:?})  repeat/type-grid({}-{}) chance/type-action({}-{})", // rest: {:?} (len: {}) \n{:b} {:b} {:b}",
               self.number,
               self.note,
               self.velocity,
               self.channel,
               self.program,
               self.note_length,
               self.micro_move,
               self.pitch_bend,
               self.cc12,
               self.cc13,
               self.cc17,
               self.cc19,
               self.cc22,
               self.cc71,
               self.cc74,
               self.cc75,
               self.repeat_type,
               self.repeat_grid,
               self.chance_type,
               self.chance_action,
               // &self.rest, self.rest.len(),
               // self.rest[5.min(self.rest.len()-1)],
               // self.rest[6.min(self.rest.len()-1)],
               // self.rest[7.min(self.rest.len()-1)],
        )
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MidiChannel {
    Jack(u8),
    Usb(u8),
}

impl From<u16> for MidiChannel {
    fn from(x: u16) -> Self {
        if x < 16 {
            MidiChannel::Jack(x as u8 + 1)
        } else {
            MidiChannel::Usb(x as u8 + 1 - 16)
        }
    }
}
