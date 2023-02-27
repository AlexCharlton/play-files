#[allow(dead_code)]

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::fs::File;
use std::io::Read;

// use arr_macro::arr;
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
    position: Rc<RefCell<usize>>
}

#[allow(dead_code)]
impl Reader {
    fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer, position: Rc::new(RefCell::new(0))
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
        let bs = &self.buffer[p..p+n];
        *self.position.borrow_mut() += n;
        bs
    }

    fn read_bool(&self) -> bool {
        self.read() == 1
    }

    fn read_string(&self, n: usize) -> String {
        let b = self.read_bytes(n);
        std::str::from_utf8(b).expect("invalid utf-8 sequence in string").to_string()
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
            return Err(ParseError(format!("Provided project dir {:?} is not a directory", &path)));
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
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("No settings file present")))?;

        let mut buf: Vec<u8> = vec!();
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let mut attrs = Self::default();
        Self::attrs_from_reader(&reader, &mut attrs)?;

        attrs.jack_cc_mapping = (0..16).map(|_| CCMapping::from_reader(&reader) )
            .collect::<Result<Vec<CCMapping>>>()?;
        attrs.usb_cc_mapping = (0..16).map(|_| CCMapping::from_reader(&reader) )
            .collect::<Result<Vec<CCMapping>>>()?;

        Ok(attrs)
    }

    fn attrs_from_reader(reader: &Reader, settings: &mut Self) -> Result<()> {
        let mut tag = reader.read();
        while tag != 0xc2 { // Guessing that elements of the array are tagged with 0xC2
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
        reader.step_back();
        Ok(())
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("name", &self.name)
            .field("directory", &self.directory)
            .field("bpm", &self.bpm)
            .field("x20", &self.x20)
            .field("xa8", &self.xa8)
            .field("xb0", &self.xb0)
            .field("x90", &self.x90)
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
        reader.read(); // First byte probably tag (0xC2)
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

        let mut buf: Vec<u8> = vec!();
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let rest = reader.rest();

        Ok(Self {
            rest,
        })
    }
}

impl fmt::Debug for Samples {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Samples")
            .field("rest", &format!("{} bytes: {:?}...", self.rest.len(), &&self.rest[0..10.min(self.rest.len())]))
            .finish()
    }
}

#[derive(PartialEq, Clone)]
pub struct Pattern {
    pub number: u8,
    pub rest: Vec<u8>, // TODO
    pub tracks: [Option<Track>; 8],
}
impl Pattern {
    /// Read a pattern directory
    pub fn read_patterns(path: &Path) -> Result<Vec<Self>> {
        if !path.is_dir() {
            return Err(ParseError(format!("Provided patterns dir {:?} is not a directory", &path)));
        }

        let mut patterns = vec![];
        for entry in glob(&format!("{}/*.pattern", path.to_str().unwrap()))
            .map_err(|_| ParseError(format!("Could not read pattern dir")))? {
                match entry {
                    Ok(path) => {
                        let re = Regex::new(r"(\d+).pattern$").unwrap();
                        let pattern_number = if let Some(n) = re.captures(path.to_str().unwrap()) {
                            n.get(1).unwrap().as_str().parse()
                                .map_err(|_| ParseError(format!("Invalid pattern file name: {:?}", &path)))?
                        } else {
                            return Err(ParseError(format!("Invalid pattern file name: {:?}", &path)));
                        };
                        patterns.push(Self::read(&path, pattern_number)?);
                    },
                    _ => return Err(ParseError(format!("Could not read pattern dir"))),
                }
        }

        Ok(patterns)
    }

    /// Read a particular pattern file. Will also read any tracks
    pub fn read(path: &Path, number: u8) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("Cannot read pattern file: {:?}", &path)))?;

        let mut buf: Vec<u8> = vec!();
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let rest = reader.rest();

        let tracks = Track::read_tracks(&path.parent().unwrap(), number)?;
           Ok(Self {
               number,
               rest,
               tracks,
           })
    }

}

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Samples")
            .field("number", &self.number)
            .field("tracks", &self.tracks)
            .field("rest", &format!("{} bytes: {:?}...", self.rest.len(), &&self.rest[0..10.min(self.rest.len())]))
            .finish()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Track {
    pub variations: [Option<TrackVariation>; 16],
}

impl Track {
    pub fn read_tracks(path: &Path, pattern_number: u8) -> Result<[Option<Self>; 8]> {
        Ok((0..8).map(|track| {
            let mut track_files = glob(&format!("{}/{}-{}-*.track",
                                                path.to_str().unwrap(), pattern_number, track))
                .map_err(|_| ParseError(format!("Could not read track dir")))?;

            if track_files.next().is_some() {
                Ok(Some(Self {
                    variations: (0..16).map(|variation| {
                        let track_path = path.join(&format!("{}-{}-{}.track",
                                                            pattern_number, track, variation));
                        if track_path.is_file() {
                            Ok(Some(TrackVariation::read(&track_path)?))
                        } else {
                            Ok(None)
                        }
                    }).collect::<Result<Vec<_>>>()?
                        .try_into().unwrap()
                }))
            } else {
                Ok(None)
            }

        }).collect::<Result<Vec<Option<Self>>>>()?
             .try_into().unwrap())
    }
}

#[derive(PartialEq, Clone)]
pub struct TrackVariation {
    pub rest: Vec<u8>, // TODO
}

impl TrackVariation {
    pub fn read(path: &Path) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|_| ParseError(format!("Cannot read track file: {:?}", &path)))?;

        let mut buf: Vec<u8> = vec!();
        file.read_to_end(&mut buf).unwrap();
        let reader = Reader::new(buf);

        let rest = reader.rest();

        Ok(Self {
            rest
        })
    }
}

impl fmt::Debug for TrackVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Samples")
            .field("rest", &format!("{} bytes: {:?}...", self.rest.len(), &&self.rest[0..10.min(self.rest.len())]))
            .finish()
    }
}

