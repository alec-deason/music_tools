use crate::pitch::{Semitone, PitchClassSpace, PitchClassOctave};
use crate::pitch::chromatic::ChromaticPitchClassSpace;

trait Interval<PC>
    where PC: PitchClassSpace {
    fn new(a: &PitchClassOctave<PC>, b: &PitchClassOctave<PC>) -> Self;
}

struct ChromaticInterval(usize, Quality);
impl Interval<ChromaticPitchClassSpace> for ChromaticInterval {
    fn new(a: &PitchClassOctave<ChromaticPitchClassSpace>, b: &PitchClassOctave<ChromaticPitchClassSpace>) -> Self {
        let a = (a.1, (a.0).0);
        let b = (b.1, (b.0).0);
        let (a, b) = (a.max(b), a.min(b));
        let d = (a.0 * 12 + a.1) - (b.0 * 12 + b.1);
        let o = d / 12;
        let (n, q) = match d % 12 {
            0 =>  (1, Quality::Perfect),
            1 =>  (2, Quality::Minor),
            2 =>  (2, Quality::Major),
            3 =>  (3, Quality::Minor),
            4 =>  (3, Quality::Major),
            5 =>  (4, Quality::Perfect),
            6 =>  (5, Quality::Diminished),
            7 =>  (5, Quality::Perfect),
            8 =>  (6, Quality::Minor),
            9 =>  (6, Quality::Major),
            10 => (7, Quality::Minor),
            11 => (7, Quality::Major),
            12 => (8, Quality::Perfect),
            _ => panic!(),
        };
        Self(n + o * 8, q)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Quality {
    Perfect,
    Major,
    Minor,
    Diminished,
    Augmented,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::PitchConverter;
    use crate::pitch::PitchClassOctave as P;

    #[test]
    fn test_interval_number() {
        for (a, ao, b, bo, n) in &[
            ("C", 0, "D", 0, 2),
            ("E", 0, "F♯", 0, 2),
            ("C", 0, "E", 0, 3),
            ("E", 0, "G♯", 0, 3),
            ("C", 0, "F", 0, 4),
            ("F", 0, "B♭", 0, 4),
            ("C", 0, "G", 0, 5),
            ("B", 0, "F♯", 1, 5),
            ("C", 0, "A", 0, 6),
            ("E♭", 0, "C", 1, 6),
            ("C", 0, "B", 0, 7),
            ("D", 0, "C♯", 1, 7),
        ] {
            let aa = P::new(a, *ao);
            let bb = P::new(b, *bo);
            let i = ChromaticInterval::new(&aa, &bb);
            assert_eq!(i.0, *n, "{:?} {:?} {}", a, b, n);
        }
    }

    #[test]
    fn test_interval_quality() {
        for (a, ao, b, bo, n) in &[
            ("C", 0, "D", 0, Quality::Major),
            ("E", 0, "F♯", 0, Quality::Major),
            ("C", 0, "E", 0, Quality::Major),
            ("E", 0, "G♯", 0, Quality::Major),
            ("C", 0, "F", 0, Quality::Perfect),
            ("F", 0, "B♭", 0, Quality::Perfect),
            ("C", 0, "G", 0, Quality::Perfect),
            ("B", 0, "F♯", 1, Quality::Perfect),
            ("C", 0, "A", 0, Quality::Major),
            ("E♭", 0, "C", 1, Quality::Major),
            ("C", 0, "B", 0, Quality::Major),
            ("D", 0, "C♯", 1, Quality::Major),
        ] {
            let aa = P::new(a, *ao);
            let bb = P::new(b, *bo);
            let i = ChromaticInterval::new(&aa, &bb);
            assert_eq!(i.1, *n, "{:?} {:?} {:?}", a, b, n);
        }
    }
}
