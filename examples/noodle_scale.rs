#![feature(non_ascii_idents)]
extern crate music_tools;
extern crate portaudio;
extern crate rand;
extern crate num;

use std::collections::HashMap;
use rand::prelude::*;
use portaudio as pa;
use music_tools::{Pitch, Scale, Chord};
use std::f64::consts::{PI};

type AudioSample = f32;

const CHANNELS: i32 = 1;
const FRAMES: u32 = 16;
const SAMPLE_HZ: f64 = 48_000.0;

trait Voice {
    fn sample(&mut self) -> f64;
    fn play_chord(&mut self, chord: &Chord);
    fn stop(&mut self);
}

struct Kick {
    amp: f64,
    since_event: f64,
    since_onset: f64,
    sounding: bool,
}

impl Voice for Kick {
    fn sample(&mut self) -> f64 {
        self.since_event += 1.0 / SAMPLE_HZ;
        self.since_onset += 1.0 / SAMPLE_HZ;
        let amp = if self.sounding {
            self.amp * ads(0.005, 0.005, 0.75, self.since_event)
        } else {
            self.amp * sr(0.75, 0.01, self.since_event)
        };
        ((90.0 * self.since_onset * 2.0 * PI)).sin() * amp
    }

    fn play_chord(&mut self, _: &Chord) {
        self.since_event = 0.0;
        self.since_onset = 0.0;
        self.sounding = true;
    }

    fn stop(&mut self) {
        if self.sounding {
            self.since_event = 0.0;
            self.sounding = false;
        }
    }
}

struct AdditiveBell {
    chord: Chord,
    amp: f64,
    since_event: f64,
    since_onset: f64,
    sounding: bool,
}

impl Voice for AdditiveBell {
    fn sample(&mut self) -> f64 {
        let mut sample = 0.0;
        self.since_event += 1.0 / SAMPLE_HZ;
        self.since_onset += 1.0 / SAMPLE_HZ;
        let amp = if self.sounding {
            self.amp * ads(0.01, 0.01, 0.7, self.since_event)
        } else {
            self.amp * sr(0.7, 0.3, self.since_event)
        };
        
        if amp > 0.0 {
            for pitch in &self.chord.0 {
                [1.0f64, 2.23, 3.73, 4.81, 5.43, 6.24, 7.35, 8.12, 9.44, 10.21].iter().enumerate().for_each(|(i, m)| {
                    let i = i+1;
                    sample += ((pitch.0 as f64 * m * self.since_onset * 2.0 * PI)).sin() * amp * (1.0 / 2.0f64.powf(i as f64));
                })
            }
        }
        sample
    }

    fn play_chord(&mut self, chord: &Chord) {
        self.since_event = 0.0;
        self.since_onset = 0.0;
        self.sounding = true;
        self.chord = chord.clone();
    }

    fn stop(&mut self) {
        if self.sounding {
            self.since_event = 0.0;
            self.sounding = false;
        }
    }
}

struct Instrument {
    voices: Vec<(Box<dyn Voice>, f64)>,
    sequence: Vec<Note>,
    clock: f64,
}

impl Instrument {
    fn new(voice_count: usize, voice_constructor: &Fn() -> Box<dyn Voice>) -> Instrument {
        Instrument {
            voices: (0..voice_count).map(|_| (voice_constructor(), 100000.0)).collect(),
            sequence: Vec::new(),
            clock: 0.0,
        }
    }

    fn sample(&mut self) -> f64 {
        self.clock += 1.0 / SAMPLE_HZ;
        while self.sequence.len() > 0 && self.clock >= self.sequence[0].onset {
            let note = self.sequence.remove(0);
            eprintln!("Playing: {:?}", note);
            self.voices[0].0.play_chord(&note.chord);
            self.voices[0].1 = self.clock + note.duration;
            self.voices.rotate_left(1);
        }
        for (voice, end_time) in &mut self.voices {
            if *end_time < self.clock {
                voice.stop();
            }
        }
        self.voices.iter_mut().map(|v| v.0.sample()).sum()
    }

    fn schedule_note(&mut self, note: &Note) {
        if note.onset >= self.clock {
            let mut position = self.sequence.len();
            for (i, other) in self.sequence.iter().enumerate() {
                if note.onset < other.onset {
                    position = i;
                    break
                }
            }
            self.sequence.insert(position, note.clone());
        }
    }
}




fn ads(a: f64, d: f64, s: f64, t: f64) -> f64 {
    if t <= a {
        let m = 1.0 / a;
        m * t
    } else if t <= a+d {
        let m = -(1.0-s) / d;
        let t = t - a;
        1.0 + m * t
    } else {
        s
    }
}

fn sr(s:f64, r: f64, t: f64) -> f64 {
    if t <= r {
        let m = -s / r;
        s + m * t
    } else {
        0.0
    }
}

#[derive(Clone, Debug)]
struct Note {
    instrument: usize,
    chord: Chord,
    onset: f64,
    duration: f64,
    amplitude: f64,
}

fn rhythm(a: u32, b: u32, len: u32) -> Vec<u32> {
    let mut result = Vec::new();
    let mut a_m = 1;
    let mut b_m = 1;
    let mut c_m = 1;
    let mut since_last = 0;
    for i in 0..len {
        if i % a == 0 {
            a_m *= -1;
        }
        if i % b == 0 {
            b_m *= -1;
        }
        if a_m * b_m != c_m {
            result.push(since_last);
            since_last = 0;
            c_m = a_m * b_m;
        } else {
            since_last += 1;
        }
    }
    result
}

fn pitch(af: f64, bf: f64, cf: f64, pitch_series: &[Pitch], rhythm: &[u32]) -> Vec<Pitch> {
    let mut result = Vec::with_capacity(rhythm.len());
    let mut clock = 0.0;
    for duration in rhythm.iter().cloned() {
        let sample_point = clock + duration as f64 / 2.0;
        let f = 0.1 * ((sample_point * 2.0 * PI * af).sin() * 6.0 +
                (sample_point * 2.0 * PI * bf).sin() * 3.0 +
                (sample_point * 2.0 * PI * cf).sin() * 1.0);
        let mut p = f * pitch_series.len() as f64;
        let mut octave = 0;
        while p < 0.0 || p >= pitch_series.len() as f64 {
            if p < 0.0 {
                octave -= 1;
                p += pitch_series.len() as f64;
            } else {
                octave += 1;
                p -= pitch_series.len() as f64;
            }
        }
        result.push(pitch_series[p as usize].apply_interval(2.0f64.powf(octave as f64)));

        clock += duration as f64;
    }
    result
}

fn song(tonic: &Pitch) -> Vec<Note> {
    let mut rng = thread_rng();

    let semi = 16.0/15.0 - 1.0;
    let whole = semi * 2.0;
    let scale = Scale::new(&[whole, whole, semi, whole, whole, whole, semi]);

    let mut changes:HashMap<&str, usize> = HashMap::new();
    changes.insert("I", 0);
    changes.insert("IV", 3);
    changes.insert("V", 4);

    let twelve_bar = vec!["I", "I", "I", "I", "IV", "IV", "I", "I", "V", "IV", "I", "V"];
    let mut other:Vec<&str> = (0..60).map(|_| *["I", "IV", "V"].choose(&mut rng).unwrap()).collect();

    // Make sure it ends on a strong cadence
    if other[other.len()-1] != "V" {
        other.push("V");
    }
    other.push("I");

    let melody:Vec<Pitch> = other.iter().map(|c| scale.pitch(tonic, changes[c] as i32)).collect();

    let major = vec![1.0, (1.0 + semi as f64).powf(4.0), (1.0 + semi as f64).powf(6.0), (  1.0 + semi as f64).powf(9.0), (1.0 + semi as f64).powf(9.0), (1.0 + semi as f64).powf(6.0), (1.0+semi as f64).powf(4.0), 1.0];

    let mut notes = Vec::new();
    let beat = 60.0 / 70.0;
    let mut clock = 0.0;
    // Baseline
    for pitch in &melody {
        for interval in &major {
            notes.push(Note {
                instrument: 1,
                chord: Chord(vec![pitch.apply_interval(*interval*0.5)]),
                onset: clock,
                duration: (beat/2.0)*0.75,
                amplitude: 0.8,
            });
            clock += beat/4.0;
        }
    }

    let mut clock = 0.0;
    // Melody
    for pitch in &melody {
        for _ in 0..8 {
            for (i, interval) in major[1..4].iter().enumerate() {
                notes.push(Note {
                    instrument: 2+i,
                    chord: Chord(vec![pitch.apply_interval(*interval)]),
                    onset: clock,
                    duration: (beat/4.0),
                    amplitude: 1.0,
                });
            }
            clock += beat/4.0;
            notes.push(Note {
                instrument: 5,
                chord: Chord(vec![pitch.apply_interval(1.0)]),
                onset: clock,
                duration: (beat/4.0),
                amplitude: 1.0,
            });
            clock += beat/4.0;
        }
    }

    while clock >= 0.0 {
        // Kick
        notes.push(Note {
            instrument: 0,
            chord: Chord(vec![]),
            onset: clock,
            duration: 0.1,
            amplitude: 1.5,
        });
        clock -= beat;
    }
    notes.sort_by_key(|n| (n.onset * 100000.0) as i32);
    notes
}

fn main() -> Result<(), pa::Error> {
    let base_pitch = Pitch(220.0);
    let notes = song(&base_pitch);
    eprintln!("Song: {:?}", notes);

    let mut instruments = vec![
        Instrument::new(1, &|| Box::new(Kick { amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false })),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
    ];

    for note in notes {
        let instrument = &mut instruments[note.instrument];
        instrument.schedule_note(&note);
    }
    eprintln!("{:?}", instruments[0].sequence);

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            buffer[idx] = instruments.iter_mut().map(|instrument| instrument.sample() as f32).sum::<f32>() * 0.05;
            idx += 1;
        }
        pa::Continue
    };
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings::<AudioSample>(CHANNELS, SAMPLE_HZ, FRAMES)?;
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;
    stream.start()?;

    while let Ok(true) = stream.is_active() {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}
