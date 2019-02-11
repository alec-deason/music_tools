use super::{PitchSpace, EqualTempermentSemitone};

struct MIDILogLinearPitchSpace;

impl PitchSpace for MIDILogLinearPitchSpace {
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


#[cfg(test)]
mod log_linear_tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn from_frequency() {
        assert_approx_eq!(MIDILogLinearPitchSpace::from_frequency(261.63).0, 60.0, 0.001);
        assert_approx_eq!(MIDILogLinearPitchSpace::from_frequency(440.0).0, 69.0, 0.001);
    }

    #[test]
    fn distance() {
        assert_approx_eq!(MIDILogLinearPitchSpace::distance(
            &EqualTempermentSemitone(60.0),
            &EqualTempermentSemitone(69.0)),
            9.0,
            0.001
        );
    }

    #[test]
    fn to_frequency() {
        let f = MIDILogLinearPitchSpace::to_frequency(&EqualTempermentSemitone(60.0));
        assert_approx_eq!(f, 261.63, 0.01);
        let f = MIDILogLinearPitchSpace::to_frequency(&EqualTempermentSemitone(69.0));
        assert_approx_eq!(f, 440.0, 0.001);
    }
}
