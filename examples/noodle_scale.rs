#![feature(non_ascii_idents)]
extern crate music_tools;
extern crate portaudio;
extern crate rand;
extern crate num;

use std::thread::sleep;
use std::time::Duration;
use rand::prelude::*;
use portaudio as pa;
use music_tools::{Pitch, Chord};
use std::f64::consts::{PI};

type AudioSample = f32;

const CHANNELS: i32 = 1;
const FRAMES: u32 = 8;
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

    fn exhausted(&self) -> bool {
        self.sequence.len() == 0
    }

    fn reset(&mut self) {
        //self.clock = 0.0;
        self.sequence.clear();
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
        }
        since_last += 1;
    }
    result
}

fn pitch(af: f64, bf: f64, cf: f64, pitch_series: &[Pitch], rhythm: &[f64]) -> Vec<Pitch> {
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

fn song(tonic: &Pitch, target_length: u32) -> Vec<Note> {
    let mut rng = thread_rng();

    let semi = 16.0/15.0 - 1.0;
    let mut intervals = vec![0, 1, 1, 1, 1];
    for _ in 0..9 {
        let idx = rng.gen_range(0, intervals.len());
        intervals[idx] += 1;
    }
    let mut pitch_series = vec![intervals[0]];
    for interval in &intervals[1..] {
        let last = pitch_series[pitch_series.len()-1];
        pitch_series.push(last + interval)
    }
    let pitch_series:Vec<Pitch> = pitch_series.iter().map(|p| tonic.apply_interval((1.0f64+semi).powf(*p as f64))).collect();
    let beat = 60.0 / 90.0;

    let rhythm_pattern:Vec<f64> = rhythm(rng.gen_range(3, 10), rng.gen_range(2, 8), target_length).iter().map(|r| *r as f64 * beat * 0.25).collect();
    let a = rng.gen_range(7.0, 13.0);
    let b = rng.gen_range(3.0, 7.0);
    let c = rng.gen_range(0.0, 1.0);
    let melody_a = pitch(a, b, c, &pitch_series, &rhythm_pattern);

    let mut notes:Vec<Note> = melody_a.iter().zip(&rhythm_pattern).scan(0.0, |clock, (m, r)| {
        let note = Note {
            instrument: 1,
            chord: Chord(vec![*m]),
            onset: *clock,
            duration: 0.1,
            amplitude: 1.0,
        };
        *clock += *r as f64 * beat;
        Some(note)
    }).collect();

    let rhythm_pattern:Vec<f64> = rhythm(rng.gen_range(3, 10), rng.gen_range(2, 8), target_length).iter().map(|r| *r as f64 * beat * 0.25).collect();
    let a = rng.gen_range(7.0, 13.0);
    let b = rng.gen_range(3.0, 7.0);
    let c = rng.gen_range(0.0, 1.0);
    let melody_b = pitch(a, b, c, &pitch_series, &rhythm_pattern);
    notes.extend(melody_b.iter().zip(&rhythm_pattern).scan(0.0, |clock, (m, r)| {
          let note = Note {
              instrument: 2,
              chord: Chord(vec![m.apply_interval(0.5)]),
              onset: *clock,
              duration: 0.1,
              amplitude: 1.0,
          };
          *clock += *r as f64 * beat;
          Some(note)
      }));

    let rhythm_pattern:Vec<f64> = rhythm(rng.gen_range(3, 10), rng.gen_range(2, 8), target_length).iter().map(|r| *r as f64 * beat * 0.25).collect();
    notes.extend(rhythm_pattern.iter().scan(0.0, |clock, r| {
          let note = Note {
              instrument: 0,
              chord: Chord(vec![]),
              onset: *clock,
              duration: 0.1,
              amplitude: 1.0,
          };
          *clock += *r as f64 * beat;
          Some(note)
    }));

    notes
}

fn main() -> Result<(), pa::Error> {
    let base_pitch = Pitch(220.0);

    let mut instruments = vec![
        Instrument::new(1, &|| Box::new(Kick { amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false })),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
        Instrument::new(3, &|| Box::new(AdditiveBell { chord: Chord(vec![base_pitch, base_pitch.apply_interval((16.0/15.0 - 1.0)*4.0),   base_pitch.apply_interval((16.0/15.0 -1.0)*7.0)]), amp: 1.0, since_event: 1000.0, since_onset: 10000.0, sounding: false }) ),
    ];


    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings::<AudioSample>(CHANNELS, SAMPLE_HZ, FRAMES)?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;

    loop {
        let frames = match stream.write_available() {
            Ok(available) => match available {
                pa::StreamAvailable::Frames(frames) => frames,
                pa::StreamAvailable::InputOverflowed => {
                    eprintln!("underflow");
                    0
                },
                pa::StreamAvailable::OutputUnderflowed => {
                    eprintln!("overflow");
                    0
                },
            },
            Err(_) => panic!(),
        };
        if frames > 0 {
            stream.write(frames as u32, |output| {
                for i in 0..frames as usize {
                    let sample = instruments.iter_mut().map(|instrument| instrument.sample() as f32).sum::<f32>() * 0.1;
                    output[i] = sample;
                }
            })?;
            if instruments.iter().all(|i| i.exhausted()) {
                instruments.iter_mut().for_each(|i| i.reset());
                let notes = song(&base_pitch, 1000);
                for note in notes {
                    let instrument = &mut instruments[note.instrument];
                    instrument.schedule_note(&note);
                }
            }
        } else {
            sleep(Duration::from_millis(1000/SAMPLE_HZ as u64));
        }
    }
}
