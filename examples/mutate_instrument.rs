#![feature(non_ascii_idents)]
extern crate crossbeam;
extern crate music_tools;
extern crate portaudio;
extern crate rand;
extern crate console;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use console::Term;
use std::thread;
use std::collections::{HashSet};

use std::f64::consts::{PI};
use std::f64::{MAX};
use std::io::{self, Read};

use crossbeam::queue::ArrayQueue;
use music_tools::synth::{Instrument, Instrumentation, Note, Voice};
use music_tools::synth::simple_instruments::{ads, sr, Function, DAGVoice};
use music_tools::{Pitch, Scale};
use portaudio as pa;
use rand::prelude::*;
use std::sync::Arc;

type AudioSample = f32;

const CHANNELS: i32 = 1;
const FRAMES: u32 = 32;
const SAMPLE_HZ: f64 = 48_000.0;


fn mutate(parent: &DAGVoice) -> DAGVoice {
    let mut rng = thread_rng();
    let mut child = parent.clone();
    for s in &mut child.initial_state {
        *s *= rng.gen_range(0.8, 1.2);
    }
    child
}


fn main() -> Result<(), pa::Error> {
    let mut voice = DAGVoice::new(1.0);
    let mut instrumentation = Instrumentation::new();
    {
        let voice = voice.clone();
        instrumentation.add_instrument(
            0,
            Instrument::new(SAMPLE_HZ, 3, & move|| Box::new(voice.clone())),
        );
    }

    let sample_buffer = Arc::new(ArrayQueue::new(SAMPLE_HZ as usize / 2));

    let callback;
    {
        let sample_buffer = sample_buffer.clone();
        callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            for i in 0..frames as usize {
                match sample_buffer.pop() {
                    Ok(sample) => buffer[i] = sample,
                    Err(_) => buffer[i] = 0.0,
                }
            }

            pa::Continue
        };
    }

    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings::<AudioSample>(CHANNELS, SAMPLE_HZ, FRAMES)?;
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    let semi = 16.0 / 15.0 - 1.0;
    let whole = semi * 2.0;
    let major_scale = Scale::new(&[whole, whole, semi, whole, whole, whole, semi]);
    let tonic = Pitch(220.0);
    let mut degree = 0;
    let input_queue = Arc::new(ArrayQueue::new(1));
    {
        let input_queue = input_queue.clone();
        thread::spawn(move || {
            let term = Term::stderr();
            loop {
                match term.read_char() {
                    Ok(n) => {
                        input_queue.push(n);
                    },
                    Err(_) => (),
                }
            }
        });
    }

    let term = Term::stdout();
    loop {
        if !input_queue.is_empty() {
            let c = input_queue.pop().unwrap();
            let mut regenerate = false;
            if c == ';' {
                regenerate = true;
            } else if c == 'q' {
                regenerate = true;
            }
            if regenerate {
                voice = mutate(&voice);
                term.write_line(&serde_json::to_string(&voice).unwrap());
                instrumentation = Instrumentation::new();
                {
                    let voice = voice.clone();
                    instrumentation.add_instrument(
                        0,
                        Instrument::new(SAMPLE_HZ, 3, & move|| Box::new(voice.clone())),
                    );
                }
            }
        }
        let len = sample_buffer.len();
        if len < SAMPLE_HZ as usize / 5 {
            if instrumentation.exhausted() {
                instrumentation.reset();
                let mut clock = 0.0;
                for _ in 0..100 {
                    let note = Note {
                        instrument: 0,
                        pitch: major_scale.pitch(&tonic, degree),
                        onset: clock,
                        duration: 0.4,
                        amplitude: 1.0,
                    };
                    clock += 0.5;

                    degree = (degree + 1) % 7;

                    instrumentation.schedule_note(&note);
                }
            }
            while !sample_buffer.is_full() {
                let sample = instrumentation.sample() as f32;
                sample_buffer.push(sample).unwrap();
            }
        } else {
            pa.sleep(500);
        }
    }
}
