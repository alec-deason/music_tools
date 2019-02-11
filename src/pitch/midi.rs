use super::{PitchSpace, EqualTempermentSemitone, PitchConverter, PitchClassOctave};
use super::chromatic::ChromaticPitchClassSpace;

struct MIDIPitchSpace;

impl PitchSpace for MIDIPitchSpace {
    type Pos = f32;
    type Dist = f32;
    type Pitch = EqualTempermentSemitone;

    fn from_frequency(f: f32) -> Self::Pitch {
        let p = 69.0 + 12.0 * (f / 440.0).log2();
        EqualTempermentSemitone(p)
    }

    fn to_frequency(p: &Self::Pitch) -> f32 {
        440.0 * 2.0f32.powf((p.0 - 69.0) / 12.0)
    }

    fn distance(a: &Self::Pitch, b: &Self::Pitch) -> Self::Dist {
        (a.0 - b.0).abs()
    }
}

impl PitchConverter for MIDIPitchSpace {
    type PitchSpace = MIDIPitchSpace;
    type PitchClassSpace = ChromaticPitchClassSpace;

    fn to_pitch(p: &PitchClassOctave<Self::PitchClassSpace>) -> EqualTempermentSemitone {
        let pc = (p.0).0;
        // Add five because MIDI octaves have 0 at octave -5 for some reason
        let o = p.1 + 5;
        EqualTempermentSemitone((o * 12 + pc) as f32)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use crate::pitch::PitchClassSpace;

    #[test]
    fn from_frequency() {
        assert_approx_eq!(MIDIPitchSpace::from_frequency(261.63).0, 60.0, 0.001);
        assert_approx_eq!(MIDIPitchSpace::from_frequency(440.0).0, 69.0, 0.001);
    }

    #[test]
    fn distance() {
        assert_approx_eq!(MIDIPitchSpace::distance(
            &EqualTempermentSemitone(60.0),
            &EqualTempermentSemitone(69.0)),
            9.0,
            0.001
        );
    }

    #[test]
    fn to_frequency() {
        let f = MIDIPitchSpace::to_frequency(&EqualTempermentSemitone(60.0));
        assert_approx_eq!(f, 261.63, 0.01);
        let f = MIDIPitchSpace::to_frequency(&EqualTempermentSemitone(69.0));
        assert_approx_eq!(f, 440.0, 0.001);
    }

    #[test]
    fn to_pitch() {
        let pc = ChromaticPitchClassSpace::from_str("C").unwrap();
        let o = PitchClassOctave(pc, 0);
        assert_eq!(MIDIPitchSpace::to_pitch(&o), EqualTempermentSemitone(60.0));
        let o = PitchClassOctave(pc, 4);
        assert_eq!(MIDIPitchSpace::to_pitch(&o), EqualTempermentSemitone(108.0));

        let pc = ChromaticPitchClassSpace::from_str("A").unwrap();
        let o = PitchClassOctave(pc, 0);
        assert_eq!(MIDIPitchSpace::to_pitch(&o), EqualTempermentSemitone(69.0));
    }
}
