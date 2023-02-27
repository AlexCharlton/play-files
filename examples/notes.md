## Example Projects
- blank
  - ""New project""
- empty notes on 1&3
  - 4 blank notes: on steps 0&8 on tracks 1&2
- single empty note
  - 1 empty note on step 1 track 1
- c4 on 1
  - 1 empty midi c4 (jack 1) note on step 1 track 1
- 400 bpm
  - Empty project set to 400 bpm
- Believe It
- The demo

## Observations
Believe It project structure:
```
project/
  patterns/
    0-0-1.track
    0-0-2.track
    0-0-3.track
    0-1-1.track
    0-1-2.track
    0-1-3.track
    0-2-0.track
    0-2-2.track
    0.pattern
  samples/
    [NNN] <sample name>.wav
    samplesMetadata
  settings
```

### settings
[two bytes][name][two bytes][Directory]

- blank: name is 5 chars
- 'empty notes on 1+3 ': 19 chars
- '400 bpm': 7 chars
- 'Believe It': 10 chars

At least the first part of the file seems to take the pattern of [<1 byte tag><attr>]+
where attributes can show up in any order.

The last part of the file seems to be a 32 element array of 13 byte values that are all the same in the blank case:
c2, 01, 0a, 0a, 08, 4a, 47, 16, 4b, 11, 13, 0c, 0d,
c2, 01, 0a, 0a, 08, 4a, 47, 16, 4b, 11, 13, 0c, 0d,

What are their 32 of? 
- Midi> CC Mappings


Looking at the first bits:

Blank: (120bpm)
8501 0000 f042 a801 01
400 bpm:
8501 0000 c843 a801 01
Believe it: (162 bpm)
8501 0000 2243 a801 01b0 0101
The demo: (139 bpm)
8501 0000 0b43 9001 e9ffff ffff ffff ffff 01a8 0101

0xf042:
1111 0000 0100 0010
120 (bpm) in binary: 0111 1000

0xc843 
1100 1000 0100 0011
400 (bpm) in binary: 1 1001 0000

0x2243 
0010 0010 0100 0011
162 (bpm) in binary: 1010 0010

0x0b43 
0000 1011 0100 0011
139 (bpm) in binary: 1000 1011

These are LE 32 bit floats:  <0000 XXXX>

Midi CC Mappings
Default: 
- CC Cutoff 74
- CC resonance 71
- CC Sample Attach 22
- CC Sample Decay 75
- CC Reverb Send 17
- CC Delay Send 19
- CC Overdrive 12
- CC Bit depth 13

These match up with the final 8 values

### patterns

#### track files
Name: ``<patern_n>-<trackn>-<variation_n>.track``

Not all patterns have track files. E.g. 'The demo':Pattern 9 or 'single empty note'

### samplesMetadata
Category names stored at the end
