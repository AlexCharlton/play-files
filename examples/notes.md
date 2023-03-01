## Example Projects
All v1.0.1:
- blank
  - ""New project""
- empty notes on 1&3
  - 4 blank notes: on steps 0&8 on tracks 1&2
- single empty note
  - 1 empty note on step 1 track 1
- c4 on 1
  - 1 empty midi c4 (jack 1) note on step 1 track 1
- sample st 2 trk 1+2
  - Samples loaded
  - Added a note on step 2 of tracks 1+2. 
  - Track 1 has sample 1, C4
  - track 2 has sample 2, C#4, volume 2db, pan 3R, Reverb send 4%, Delay 5%
- 400 bpm
  - Empty project set to 400 bpm
- Believe It
- The demo

v1.2.0:
- Test 1
  - Track 1: Row of C4, First sample in bass, nothing else modified
    - Step 4 was accidentally micromoved +100
  - Track 2:
    - First 4 steps C4, B8 (max note), C0 (min note), C4 microtune +100
    - Second 4 steps: max bass sample:  C4 microtune -100, C4, C4, C4
  - Track 3: First kick sample with sample folder selected
    - step 1: micro move +11/24
    - step 2: micro move -11/24
    - step 3: repeat type: Straight (first); Repeat grid: 2 hits | 1 step (first)
    - step 4: repeat type: Down and up (last of 17); Repeat grid: 32 hits | 8 steps (last of 16)
    - step 5: Sample start 1500/1500 ms; sample end 0/1500ms
    - step 6: Sample attack 100%
    - step 7: Sample decay 100%
  - Track 4: First kick sample without sample folder selected
    - step 1: reverb send 100%
    - step 2: delay send 100%
    - step 3: 90% chance (first option); play step (first option)
    - step 4: skip 4 | Play 5 (last option of 43 including Always); action: humanize (last option of 10)
    - step 5: Volume +12dB
    - step 6: Volume -infdB
    - step 7: Volume -12dB
  - Track 5: Track length 12; Play mode Thumper (6th option)
    - Step 1: Volume -25; Pan 100L
    - Step 2: Pan 100R
    - Step 3: Filter HP100, Resonance 100%
    - Step 4: Filter LP100, Resonance 50%
  - Track 6: Track Play mode Reverse (2nd option); speed 8/1 (max value); Track swing: 25% (min value)
    - Step 1: Overdrive 100% 4 bits
    - Step 2: Overdrive 90%; 8 bits
- Track 7: Empty, soloed; Track Speed: Pause (min value); Track swing: 75% (max value)
- Track 8: Empty, muted; Track Speed: 1/16 (second to min value); Track swing: 75% (max value)

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
```
c2, 01, 0a, 0a, 08, 4a, 47, 16, 4b, 11, 13, 0c, 0d,
c2, 01, 0a, 0a, 08, 4a, 47, 16, 4b, 11, 13, 0c, 0d,
```

What are their 32 of? 
- Midi> CC Mappings

Looking at the first bits of the file:

Blank: (120bpm)
`8501 0000 f042 a801 01`
400 bpm:
`8501 0000 c843 a801 01`
Believe it: (162 bpm)
`8501 0000 2243 a801 01b0 0101`
The demo: (139 bpm)
`8501 0000 0b43 9001 e9ffff ffff ffff ffff 01a8 0101`

`0xf042`:
`1111 0000 0100 0010`
120 (bpm) in binary: 0111 1000

`0xc843` 
`1100 1000 0100 0011`
400 (bpm) in binary: 1 1001 0000

`0x2243` 
`0010 0010 0100 0011`
162 (bpm) in binary: 1010 0010

`0x0b43` 
`0000 1011 0100 0011`
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

These match up with the final 8 values. What about the first four bytes? 
I have not seen any other values other than `0x010a 0a08`.
Bytes 3+4 Maybe be a variable length quantity preceded by its apparent tag, `0x0a`.

### patterns
Looking at files with no note data:

File seems to have repeating 48 byte chunks, eg:
```
00000ab0: 0000 000a 2e0a 2c00 0000 0000 0000 0000  ......,.........
00000ac0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000ad0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000ae0: 0000 000a 2e0a 2c00 0000 0000 0000 0000  ......,.........
00000af0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000b00: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000b10: 0000 000a 2e0a 2c00 0000 0000 0000 0000  ......,.........
00000b20: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000b30: 0000 0000 0000 0000 0000 0000 0000 0000  ................
```

After 64 repetitions (max number of steps in a track), there is a 31 byte (correction: 28 byte) separator, then more 48 byte repetitions. e.g. separator (not totally clear where boundaries lie):

```
00000c00: 0000 0010 1018 0420 0128 0138 324a 1000
00000c10: 0000 0000 0000 0000 0000 0000 0000 0a
```

The files are long enough to hold 16 of these track chunks, plus 13 extra bits at the end. This lines up with the 16 available tracks per pattern (8 Audio + 8 Midi).

Adding note data lengthens the files.

Note data appears to start with `0aXX 0a2c`. This means that the start of files starts with an additional 3 bytes.

The number after the first 0a appears to be the number of bytes in the following 0a chunk. This is not true of the 0a at the very start of the file which is followed by 2 bytes of unknown meaning.

Actually these three bytes appear at the beginning of every track. The first byte appears to have something to do with the contents (always 9c in an empty track, different value in non-empty tracks).

Track start `0a XX 18` - Never seen anything other than 18

When the note is empty: `0a2e 0a2c` is followed by 44 bytes of zeroes. 

Looking at notes in `sample st 2 trk 1+2`:

Track 1, second note:
```
0a 34 0a 2c 
b0 1d 00 00 
00 00 00 00 
10 00 00 00
3c 00 00 00 
00 00 00 00
00 00 ff 7f 
00 00 00 00
00 00 ff ff 
00 00 00 00
00 00 00 00 
00 00 ae ff
10 01 18 ff 
ff 7f
```

Track 2, second note:
 - track 2 has sample 2, C#4, volume 2db, pan 3R, Reverb send 4%, Delay 5%
```
0a 34 0a 2c 
40 1f 2c 01 < different
00 00 00 00 
10 00 00 00 
3d 00 f4 01 < diff
90 01 01 00 < diff
00 00 ff 7f 
00 00 00 00 
00 00 ff ff 
00 00 00 00 
00 00 00 00 
00 00 ae ff 
10 01 18 ff 
ff 7f 
```

Looking back to the first bytes on the track. It appears to have something to do with track length.

Value is `[0x9c, 0x18]` when empty (Byte count of 3100)
`10011100 00011000`
Value is `[0xa2, 0x18]` when byte count is 3106
`10100010 00011000` 

Looks like this is a little-endian variation of [variable length quantity](https://en.wikipedia.org/wiki/Variable-length_quantity) - i.e. the LSB comes first in the stream!

Looking at how steps start:
The first character after the second `0x0a` always seems to be `0x2c` = 44, which is the number of 0s contained in an empty step. Is this supposed to be a length? Number of elements? It's unclear why else we would have two `0x0a` values one after another.

In `sample st 2 trk 1+2` the tracks with extra length have 6 extra bytes. Looking at more examples, I don't see any non-empty tracks that have a length other than 50.

4th byte is sample number? 13th byte is note?

Soloing, muting, and selection do not carry over from a load.

Footer for track files:
[21, 0, 0, 240, 66]

#### `test 1** Project
Using this to determine step values. 

***

First 2 bytes relate to volume.

LE value of -Inf is 0
LE value of -25dB is 2600
LE value of -12dB is 5200
LE value of 0dB is 7600
LE value of +12dB is 1000

1dB = 200

***

Second 2 bytes relate to panning
LE value of 100L is -10000
LE value of 100R is 10000

1% pan = 100

***

Next 4 bytes relate to filter

First 2 are the cutoff with HP100 = 10000, LP100 = 10000
1% filter = 100
Second 2 are the amount of resonance with 0 being 0 and 10000 = 100%
1% resonance = 100

****

Next 4 bytes: Bit depth and overdrive
Unknown: Always `16 0 0 0 `


*** 

Next 2 bytes: note
Midi note number (second byte is 0)

***

Next 4 bytes: Reverb send; delay send
First 2 negative: 10000 = -11/24
Next 2 positive: 10000 = 11/24

***

Next 2 bytes: Sample number

***

Next 4 bytes: Sample start, sample end

Min value: 0, Max value: 32767 (0x7FFF)

***

Next 2 bytes: microtune
-10000-10000; 100 = 1 cent

***

Next 4 bytes: sample attack/decay
0-10000 = 0-100%

***

Skipped a few parameters, since this is getting repetitive. We do this 22 times

***

Final 6 bytes: Unknown
Always `16, 1, 24, 255, 255, 127` 

#### track footers
Differ in length depending on contents. Eg. setting one of track length or play mode extended the footer 2 bytes

The second byte is track length. Are these tagged values? Yes!


#### track files
Name: ``<patern_n>-<trackn>-<variation_n>.track``

Not all patterns have track files. E.g. 'The demo':Pattern 9 or 'single empty note'. Are these only used for variations?

Track files look like they are long enough to hold a track chunk from a pattern file.

### samplesMetadata
Category names stored at the end
