use std::fmt;

use crate::pitch::{PitchClassOctave, PitchClassSpace};

#[derive(Copy, Clone, Debug)]
pub enum IntervalPattern {
    Major,
    Minor,
}

impl IntervalPattern {
    fn len(&self) -> usize {
        match self {
            IntervalPattern::Major => 7,
            IntervalPattern::Minor => 7,
        }
    }

    fn interval_pattern(&self) -> Vec<usize> {
        match self {
            IntervalPattern::Major => vec![2, 2, 1, 2, 2, 2, 1],
            IntervalPattern::Minor => vec![2, 1, 2, 2, 1, 2, 2],
        }
    }
}

pub struct Scale<C: PitchClassSpace> {
    tonic: C::PitchClass,
    interval_pattern: IntervalPattern,
}

impl<C: PitchClassSpace> fmt::Debug for Scale<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?}", self.tonic, self.interval_pattern)
    }
}

impl<C: PitchClassSpace> Scale<C> {
    pub fn new(tonic: C::PitchClass, interval_pattern: IntervalPattern) -> Scale<C> {
        Scale {
            tonic,
            interval_pattern,
        }
    }

    pub fn len(&self) -> usize {
        self.interval_pattern.len()
    }

    pub fn note_from_degree(&self, degree: i32) -> PitchClassOctave<C> {
        let rising = degree > 0;
        let degree = degree.abs() as usize;
        let mut pc = self.tonic;
        let mut pattern = self.interval_pattern.interval_pattern().clone();
        if !rising {
            pattern.reverse();
        }
        let mut d_octave = 0;
        for j in pattern.iter().cycle().take(degree) {
            for _ in 0..*j {
                if rising {
                    pc = C::successor(&pc);
                    if C::degree(&pc) == 0 {
                        d_octave += 1;
                    }
                } else {
                    if C::degree(&pc) == 0 {
                        d_octave -= 1;
                    }
                    pc = C::precursor(&pc);
                }
            }
        }
        PitchClassOctave(pc, d_octave)
    }

    pub fn pitch_classes(&self) -> Vec<C::PitchClass> {
        (0..self.len())
            .map(|d| self.note_from_degree(d as i32).0)
            .collect()
    }

    pub fn degree_from_note(&self, note: &C::PitchClass) -> Option<usize> {
        // Ignores octave...
        self.pitch_classes().iter().position(|l| l == note)
    }

    pub fn is_step(&self, a: &PitchClassOctave<C>, b: &PitchClassOctave<C>) -> bool {
        let (a, b) = (a.min(b), a.max(b));
        let a_d = self.degree_from_note(&a.0);
        if let Some(a_d) = a_d {
            let a_1 = self.note_from_degree(a_d as i32 + 1);
            if a_1.0 == b.0 {
                if C::degree(&a_1.0) < C::degree(&a.0) {
                    // we've gone through an octive
                    a.1 + 1 == b.1
                } else {
                    a.1 == b.1
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::chromatic::ChromaticPitchClassSpace as PC;

    #[test]
    fn test_note_from_degree_positive_degrees() {
        let scale: Scale<PC> = Scale::new(PC::from_str("E").unwrap(), IntervalPattern::Major);
        for (degree, expected) in &[
            (0, PitchClassOctave(PC::from_str("E").unwrap(), 0)),
            (1, PitchClassOctave(PC::from_str("F♯").unwrap(), 0)),
            (2, PitchClassOctave(PC::from_str("G♯").unwrap(), 0)),
            (3, PitchClassOctave(PC::from_str("A").unwrap(), 0)),
            (4, PitchClassOctave(PC::from_str("B").unwrap(), 0)),
            (5, PitchClassOctave(PC::from_str("C♯").unwrap(), 1)),
            (6, PitchClassOctave(PC::from_str("D♯").unwrap(), 1)),
            (7, PitchClassOctave(PC::from_str("E").unwrap(), 1)),
            (8, PitchClassOctave(PC::from_str("F♯").unwrap(), 1)),
            (9, PitchClassOctave(PC::from_str("G♯").unwrap(), 1)),
        ] {
            assert_eq!(scale.note_from_degree(*degree), *expected, "{}", degree);
        }
        let scale: Scale<PC> = Scale::new(PC::from_str("E").unwrap(), IntervalPattern::Major);
        assert_eq!(
            scale.note_from_degree(5),
            PitchClassOctave(PC::from_str("C♯").unwrap(), 1)
        );
        assert_eq!(
            scale.note_from_degree(4),
            PitchClassOctave(PC::from_str("B").unwrap(), 0)
        );
    }

    #[test]
    fn test_note_from_degree_negative_degrees() {
        let scale: Scale<PC> = Scale::new(PC::from_str("A").unwrap(), IntervalPattern::Minor);
        for (degree, expected) in &[
            (0, PitchClassOctave(PC::from_str("A").unwrap(), 0)),
            (-1, PitchClassOctave(PC::from_str("G").unwrap(), 0)),
            (-2, PitchClassOctave(PC::from_str("F").unwrap(), 0)),
            (-3, PitchClassOctave(PC::from_str("E").unwrap(), 0)),
            (-4, PitchClassOctave(PC::from_str("D").unwrap(), 0)),
            (-5, PitchClassOctave(PC::from_str("C").unwrap(), 0)),
            (-6, PitchClassOctave(PC::from_str("B").unwrap(), -1)),
            (-7, PitchClassOctave(PC::from_str("A").unwrap(), -1)),
            (-8, PitchClassOctave(PC::from_str("G").unwrap(), -1)),
            (-9, PitchClassOctave(PC::from_str("F").unwrap(), -1)),
        ] {
            assert_eq!(scale.note_from_degree(*degree), *expected, "{}", degree);
        }
    }

    #[test]
    fn test_pitch_classes() {
        let scale: Scale<PC> = Scale::new(PC::from_str("A").unwrap(), IntervalPattern::Minor);
        assert_eq!(
            scale.pitch_classes(),
            vec![
                PC::from_str("A").unwrap(),
                PC::from_str("B").unwrap(),
                PC::from_str("C").unwrap(),
                PC::from_str("D").unwrap(),
                PC::from_str("E").unwrap(),
                PC::from_str("F").unwrap(),
                PC::from_str("G").unwrap(),
            ]
        );
        let scale: Scale<PC> = Scale::new(PC::from_str("D").unwrap(), IntervalPattern::Major);
        assert_eq!(
            scale.pitch_classes(),
            vec![
                PC::from_str("D").unwrap(),
                PC::from_str("E").unwrap(),
                PC::from_str("F♯").unwrap(),
                PC::from_str("G").unwrap(),
                PC::from_str("A").unwrap(),
                PC::from_str("B").unwrap(),
                PC::from_str("C♯").unwrap(),
            ]
        )
    }

    #[test]
    fn test_degree_from_note() {
        let scale: Scale<PC> = Scale::new(PC::from_str("G").unwrap(), IntervalPattern::Major);
        assert_eq!(scale.degree_from_note(&PC::from_str("G").unwrap()), Some(0));
        assert_eq!(scale.degree_from_note(&PC::from_str("A").unwrap()), Some(1));
        assert_eq!(scale.degree_from_note(&PC::from_str("B").unwrap()), Some(2));
        assert_eq!(
            scale.degree_from_note(&PC::from_str("F♯").unwrap()),
            Some(6)
        );
        assert_eq!(scale.degree_from_note(&PC::from_str("C♯").unwrap()), None);
        let scale: Scale<PC> = Scale::new(PC::from_str("E").unwrap(), IntervalPattern::Major);
        assert_eq!(
            scale.degree_from_note(&PC::from_str("C♯").unwrap()),
            Some(5)
        );
        assert_eq!(scale.degree_from_note(&PC::from_str("B").unwrap()), Some(4));
    }

    #[test]
    fn is_step() {
        let scale: Scale<PC> = Scale::new(PC::from_str("G").unwrap(), IntervalPattern::Major);
        let a = PitchClassOctave(PC::from_str("G").unwrap(), 0);
        let b = PitchClassOctave(PC::from_str("F").unwrap(), 0);
        assert!(!scale.is_step(&b, &a));

        for tonic in &[
            PC::from_str("E").unwrap(),
            PC::from_str("C").unwrap(),
            PC::from_str("A").unwrap(),
        ] {
            for pattern in &[IntervalPattern::Major, IntervalPattern::Minor] {
                let scale: Scale<PC> = Scale::new(*tonic, *pattern);
                for i in 0..scale.len() * 2 {
                    eprintln!("thing");
                    let a = scale.note_from_degree(i as i32);
                    let b = scale.note_from_degree(i as i32 + 1);
                    eprintln!("{} {:?} {:?} {:?}", i, a, b, scale);
                    assert!(scale.is_step(&b, &a));
                }
            }
        }
    }

    #[test]
    fn is_step_across_octaves() {
        let scale: Scale<PC> = Scale::new(PC::from_str("E").unwrap(), IntervalPattern::Major);
        let a = PitchClassOctave(PC::from_str("C").unwrap(), 0);
        let b = PitchClassOctave(PC::from_str("D").unwrap(), 1);
        assert!(!scale.is_step(&b, &a));
        let a = PitchClassOctave(PC::from_str("C").unwrap(), 0);
        let b = PitchClassOctave(PC::from_str("D").unwrap(), -1);
        assert!(!scale.is_step(&b, &a));
    }

}
