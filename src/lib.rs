extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod synth;

use rand::prelude::*;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct Pitch(pub f32);
impl Hash for Pitch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.0 * 1000.0) as i32).hash(state);
    }
}
impl PartialEq for Pitch {
    fn eq(&self, other: &Pitch) -> bool {
        self.0 == other.0
    }
}
impl Eq for Pitch {}

impl Pitch {
    pub fn apply_interval(&self, interval: f64) -> Pitch {
        Pitch(self.0 * interval as f32)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Interval(pub f64);

#[derive(Clone, Debug)]
pub struct Chord(pub Vec<Pitch>);
impl Chord {
    pub fn invert(&self) -> Self {
        let mut chord = self.clone();
        if chord.0[0].0 < chord.0[1].0 {
            chord.0[0] = chord.0[0].apply_interval(2.0);
        } else {
            chord.0[0] = chord.0[0].apply_interval(0.5);
        }
        chord
    }

    pub fn randomize_voicing(&self) -> Self {
        let mut chord = self.clone();
        for pitch in chord.0[1..].iter_mut() {
            if thread_rng().gen::<f32>() > 0.5 {
                *pitch = pitch.apply_interval(2.0);
            } else {
                *pitch = pitch.apply_interval(0.5);
            }
        }
        chord
    }
}

pub struct Scale {
    interval_pattern: Vec<f32>,
}

impl Scale {
    pub fn new(interval_pattern: &[f32]) -> Self {
        Self {
            interval_pattern: interval_pattern.iter().cloned().collect(),
        }
    }

    pub fn pitch(&self, base_pitch: &Pitch, degree: i32) -> Pitch {
        let mut pitch = *base_pitch;
        let intervals: Vec<f32>;
        if degree < 0 {
            intervals = self
                .interval_pattern
                .iter()
                .rev()
                .map(|i| 1.0 - i)
                .collect();
        } else {
            intervals = self.interval_pattern.iter().map(|i| 1.0 + i).collect();
        }

        let mut degree = degree.abs();
        for interval in intervals.iter().cycle() {
            if degree <= 0 {
                break;
            }
            pitch.0 *= interval;
            degree -= 1;
        }
        pitch
    }
}
