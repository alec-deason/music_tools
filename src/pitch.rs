use num::{Float, NumCast};

#[derive(Copy, Clone, Debug)]
struct Pitch<N>(pub N);

trait PitchSpace<DistanceMetric, Freq> {
    fn distance(&self, a: &Pitch<Freq>, b: &Pitch<Freq>) -> DistanceMetric;
    fn position(&self, p: &Pitch<Freq>) -> DistanceMetric;
    fn from_position(&self, pos: DistanceMetric) -> Pitch<Freq>;
}

struct MIDILogLinearPitchSpace;

impl PitchSpace<f32, f32> for MIDILogLinearPitchSpace {
    fn distance(&self, a: &Pitch<f32>, b: &Pitch<f32>) -> f32 {
        let a = self.position(a);
        let b = self.position(b);
        (a-b).abs()
    }

    fn position(&self, p: &Pitch<f32>) -> f32 {
        69.0 + 12.0 * (p.0 / 440.0).log2()
    }

    fn from_position(&self, pos: f32) -> Pitch<f32> {
        let f = 440.0 * 2.0.powf((pos - 69.0) / 12.0);
        Pitch(f)
    }
}


#[cfg(test)]
mod log_linear_tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn position() {
        let space = MIDILogLinearPitchSpace;
        assert_approx_eq!(space.position(&Pitch(261.63)), 60.0, 0.001);
        assert_approx_eq!(space.position(&Pitch(440.0)), 69.0, 0.001);
    }

    #[test]
    fn distance() {
        let space = MIDILogLinearPitchSpace;
        assert_approx_eq!(space.distance(&Pitch(261.63), &Pitch(440.0)), 9.0, 0.001);
    }

    #[test]
    fn from_position() {
        let space = MIDILogLinearPitchSpace;
        let p = space.from_position(60.0);
        assert_approx_eq!(p.0, 261.63, 0.01);
        let p = space.from_position(69.0);
        assert_approx_eq!(p.0, 440.0, 0.001);
    }
}
