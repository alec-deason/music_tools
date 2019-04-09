use std::f64::consts::PI;
use std::f64::MAX;

use super::Voice;
use crate::Pitch;

pub struct Kick {
    amp: f64,
    since_event: f64,
    since_onset: f64,
    sounding: bool,
}

impl Kick {
    pub fn new(amplitude: f64) -> Kick {
        Kick {
            amp: amplitude,
            since_event: MAX,
            since_onset: MAX,
            sounding: false,
        }
    }
}

impl Voice for Kick {
    fn sample(&mut self, delta_time: f64) -> f64 {
        self.since_event += delta_time;
        self.since_onset += delta_time;
        let amp = if self.sounding {
            self.amp * ads(0.005, 0.005, 0.75, self.since_event)
        } else {
            self.amp * sr(0.75, 0.01, self.since_event)
        };
        (90.0 * self.since_onset * 2.0 * PI).sin() * amp
    }

    fn play_pitch(&mut self, _: &Pitch) {
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

pub struct AdditiveBell {
    pitch: Pitch,
    amp: f64,
    since_event: f64,
    since_onset: f64,
    sounding: bool,
}

impl AdditiveBell {
    pub fn new(amplitude: f64) -> AdditiveBell {
        AdditiveBell {
            pitch: Pitch(440.0),
            amp: amplitude,
            since_event: MAX,
            since_onset: MAX,
            sounding: false,
        }
    }
}

impl Voice for AdditiveBell {
    fn sample(&mut self, delta_time: f64) -> f64 {
        let mut sample = 0.0;
        self.since_event += delta_time;
        self.since_onset += delta_time;
        let amp = if self.sounding {
            self.amp * ads(0.01, 0.01, 0.7, self.since_event)
        } else {
            self.amp * sr(0.7, 0.3, self.since_event)
        };

        if amp > 0.0 {
            [
                1.0f64, 2.23, 3.73, 4.81, 5.43, 6.24, 7.35, 8.12, 9.44, 10.21,
            ]
            .iter()
            .enumerate()
            .for_each(|(i, m)| {
                let i = i + 1;
                sample += (self.pitch.0 as f64 * m * self.since_onset * 2.0 * PI).sin()
                    * amp
                    * (1.0 / 2.0f64.powf(i as f64));
            })
        }
        sample
    }

    fn play_pitch(&mut self, pitch: &Pitch) {
        self.since_event = 0.0;
        self.since_onset = 0.0;
        self.sounding = true;
        self.pitch = *pitch;
    }

    fn stop(&mut self) {
        if self.sounding {
            self.since_event = 0.0;
            self.sounding = false;
        }
    }
}

pub fn ads(a: f64, d: f64, s: f64, t: f64) -> f64 {
    if t <= a {
        let m = 1.0 / a;
        m * t
    } else if t <= a + d {
        let m = -(1.0 - s) / d;
        let t = t - a;
        1.0 + m * t
    } else {
        s
    }
}

pub fn sr(s: f64, r: f64, t: f64) -> f64 {
    if t <= r {
        let m = -s / r;
        s + m * t
    } else {
        0.0
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Function {
    Sin(usize, usize, usize),
    Copy(usize, usize),
    Multiply(usize, usize),
    Scale(usize, usize),
    Add(usize, usize),
    ADSR(usize, usize, usize, usize, usize)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct DAGVoice {
      amp: f64,
      pub initial_state: Vec<f64>,
      state: Vec<f64>,
      functions: Vec<Function>,
      since_event: f64,
      since_onset: f64,
      sounding: bool,
  }

impl DAGVoice {
    pub fn new(amp: f64) -> DAGVoice {
        let mut state = vec![0.0, 0.0];
        let mut functions = vec![];
        let mut accumulators = vec![];
        let mut lfo_targets = vec![];
        for i in 1..4 {
            let input = 0;
            state.push(0.0);
            let accumulator = state.len()-1;
            state.push(i as f64);
            let relative_freq = state.len()-1;
            state.push(1.0 / 2.0f64.powf(i as f64 - 1.0));
            let relative_amp = state.len()-1;
            state.push(0.1);
            let a = state.len()-1;
            state.push(0.1);
            let d = state.len()-1;
            state.push(0.7);
            let s = state.len()-1;
            state.push(0.2);
            let r = state.len()-1;

            lfo_targets.push(relative_amp);
            /*
            lfo_targets.push(a);
            lfo_targets.push(d);
            lfo_targets.push(s);
            lfo_targets.push(r);
            */

            functions.push(Function::Copy(input, accumulator));
            functions.push(Function::Multiply(relative_freq, accumulator));
            functions.push(Function::Sin(accumulator, 1, accumulator));
            functions.push(Function::Multiply(relative_amp, accumulator));
            functions.push(Function::ADSR(a, d, s, r, accumulator));
            accumulators.push(accumulator);
        }
        for output in &lfo_targets {
            state.push(20.0);
            let freq = state.len() - 1;
            state.push(0.0);
            let phase = state.len() - 1;
            state.push(0.0);
            let accumulator = state.len() - 1;
            state.push(0.001);
            let strength = state.len() - 1;
            functions.insert(0, Function::Sin(freq, phase, accumulator));
            functions.insert(1, Function::Multiply(strength, accumulator));
            functions.insert(2, Function::Add(accumulator, *output));
        }
        state.push(0.0);
        functions.push(Function::Copy(1, state.len()-1));
        for a in &accumulators {
            functions.push(Function::Add(*a, state.len()-1));
        }
        DAGVoice {
            amp,
            initial_state: state.clone(),
            state,
            functions,
            since_event: 100.0,
            since_onset: 100.0,
            sounding: false,
        }
    }
}
impl Voice for DAGVoice {
    fn sample(&mut self, delta_time: f64) -> f64 {
        self.since_event += delta_time;
        self.since_onset += delta_time;
        for f in &self.functions {
            match f {
                Function::Sin(freq, phase, output) => {
                    let freq = self.state[*freq];
                    let phase = self.state[*phase];
                    self.state[*output] = (freq * (self.since_onset + phase) * 2.0 * PI).sin();
                },
                Function::Multiply(input, output) => {
                    self.state[*output] *= self.state[*input];
                },
                Function::Scale(input, output) => {
                    self.state[*output] *= 1.0 + self.state[*input];
                },
                Function::Add(input, output) => {
                    self.state[*output] += self.state[*input];
                },
                Function::Copy(input, output) => {
                    self.state[*output] = self.state[*input];
                },
                Function::ADSR(a, d, s, r, output) => {
                    let a = self.state[*a];
                    let d = self.state[*d];
                    let s = self.state[*s];
                    let r = self.state[*r];
                    let amp = if self.sounding {
                        ads(a, d, s, self.since_event)
                    } else {
                        sr(s, r, self.since_event)
                    };
                    self.state[*output] *= amp;
                },
            }
        }
        self.state[self.state.len() - 1] * self.amp
    }

    fn play_pitch(&mut self, pitch: &Pitch) {
        self.state = self.initial_state.clone();
        self.since_event = 0.0;
        self.since_onset = 0.0;
        self.sounding = true;
        self.state[0] = pitch.0 as f64;
    }

    fn stop(&mut self) {
        if self.sounding {
            self.since_event = 0.0;
            self.sounding = false;
        }
    }
}

