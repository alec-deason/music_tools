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

fn ads(a: f64, d: f64, s: f64, t: f64) -> f64 {
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

fn sr(s: f64, r: f64, t: f64) -> f64 {
    if t <= r {
        let m = -s / r;
        s + m * t
    } else {
        0.0
    }
}
