use std::collections::HashMap;

use super::Pitch;

pub mod simple_instruments;

pub trait Voice {
    fn sample(&mut self, delta_time: f64) -> f64;
    fn play_pitch(&mut self, pitch: &Pitch);
    fn stop(&mut self);
}

pub struct Instrument {
    voices: Vec<(Box<dyn Voice>, f64)>,
    sequence: Vec<Note>,
    clock: f64,
    pub amp: f64,
    sample_rate: f64,
}

impl Instrument {
    pub fn new(
        sample_rate: f64,
        voice_count: usize,
        voice_constructor: &Fn() -> Box<dyn Voice>,
    ) -> Instrument {
        Instrument {
            voices: (0..voice_count)
                .map(|_| (voice_constructor(), 100000.0))
                .collect(),
            sequence: Vec::new(),
            clock: 0.0,
            amp: 1.0,
            sample_rate,
        }
    }

    pub fn sample(&mut self) -> f64 {
        let delta_time = 1.0 / self.sample_rate;
        self.clock += delta_time;
        while self.sequence.len() > 0 && self.clock >= self.sequence[0].onset {
            let note = self.sequence.remove(0);
            self.voices[0].0.play_pitch(&note.pitch);
            self.voices[0].1 = self.clock + note.duration;
            self.voices.rotate_left(1);
        }
        for (voice, end_time) in &mut self.voices {
            if *end_time < self.clock {
                voice.stop();
            }
        }
        self.voices.iter_mut().map(|v| v.0.sample(delta_time)).sum::<f64>() * self.amp
    }

    pub fn schedule_note(&mut self, note: &Note) {
        if note.onset >= self.clock {
            let mut position = self.sequence.len();
            for (i, other) in self.sequence.iter().enumerate() {
                if note.onset < other.onset {
                    position = i;
                    break;
                }
            }
            self.sequence.insert(position, note.clone());
        }
    }

    pub fn exhausted(&self) -> bool {
        self.sequence.len() == 0
    }

    pub fn reset(&mut self) {
        self.clock = 0.0;
        self.sequence.clear();
    }
}

pub struct Instrumentation {
    pub instruments: HashMap<usize, Instrument>,
}

impl Instrumentation {
    pub fn new() -> Instrumentation {
        Instrumentation {
            instruments: HashMap::new(),
        }
    }

    pub fn add_instrument(&mut self, instrument_idx: usize, instrument: Instrument) {
        self.instruments.insert(instrument_idx, instrument);
    }

    pub fn sample(&mut self) -> f64 {
        self.instruments
            .values_mut()
            .map(|instrument| instrument.sample())
            .sum::<f64>()
            * 0.1
    }

    pub fn schedule_note(&mut self, note: &Note) {
        self.instruments
            .get_mut(&note.instrument)
            .unwrap()
            .schedule_note(note);
    }

    pub fn exhausted(&self) -> bool {
        self.instruments
            .values()
            .all(|instrument| instrument.exhausted())
    }

    pub fn reset(&mut self) {
        self.instruments
            .values_mut()
            .for_each(|instrument| instrument.reset())
    }
}

#[derive(Clone, Debug)]
pub struct Note {
    pub instrument: usize,
    pub pitch: Pitch,
    pub onset: f64,
    pub duration: f64,
    pub amplitude: f64,
}
