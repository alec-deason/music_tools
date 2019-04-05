#![feature(non_ascii_idents)]
extern crate music_tools;
extern crate portaudio;
extern crate rand;
extern crate crossbeam;

use std::sync::Arc;
use crossbeam::queue::{ArrayQueue};
use rand::prelude::*;
use portaudio as pa;
use music_tools::{Pitch, Scale};
use music_tools::synth::{Instrument, Note, Instrumentation};
use music_tools::synth::simple_instruments::AdditiveBell;

type AudioSample = f32;

const CHANNELS: i32 = 1;
const FRAMES: u32 = 32;
const SAMPLE_HZ: f64 = 48_000.0;


fn main() -> Result<(), pa::Error> {
    let base_pitch = Pitch(220.0);

    let mut instrumentation = Instrumentation::new();
    instrumentation.add_instrument(0, Instrument::new(SAMPLE_HZ, 2, &|| Box::new(AdditiveBell::new(1.0))));

    let sample_buffer = Arc::new(ArrayQueue::new(SAMPLE_HZ as usize * 3));
    
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

    let semi = 16.0/15.0 - 1.0;
    let whole = semi * 2.0;
    let major_scale = Scale::new(&[whole, whole, semi, whole, whole, whole, semi]);
    let tonic = Pitch(220.0);
    let mut degree = 0;


    loop {
        let len = sample_buffer.len();
        if len < SAMPLE_HZ as usize {
            if instrumentation.exhausted() {
                instrumentation.reset();
                let mut clock = 0.0;
                for _ in 0..100 {
                    let note = Note {
                        instrument: 0,
                        pitch: major_scale.pitch(&tonic, degree),
                        onset: clock,
                        duration: 0.2,
                        amplitude: 1.0,
                    };
                    clock += 0.3;

                    degree = (degree + thread_rng().gen_range(-1,2)).max(-7).min(14);

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
