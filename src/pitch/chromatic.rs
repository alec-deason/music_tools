use std::fmt;

use super::{PitchClassSpace, IntegerPitchClass, PitchClassOctave};

#[derive(Debug)]
pub struct ChromaticPitchClassSpace;

impl PitchClassSpace for ChromaticPitchClassSpace {
    type PitchClass = IntegerPitchClass;

    fn classes() -> Vec<Self::PitchClass> {
        (0..12).map(|i| IntegerPitchClass(i)).collect()
    }

    fn successor(p: &Self::PitchClass) -> Self::PitchClass {
        IntegerPitchClass(((p.0 as i32 + 1) % 12) as usize)
    }

    fn precursor(p: &Self::PitchClass) -> Self::PitchClass{
        let mut n = p.0 as i32 -1;
        while n < 0 {
            n += 12;
        }
        IntegerPitchClass((n % 12) as usize)
    }

    fn from_str(n: &str) -> Option<Self::PitchClass> {
        let n = if n.chars().count() == 1 { format!("{}♮", n) } else { n.to_string() };
        match &n[..] {
            "C♭" => Some(IntegerPitchClass(11)),
            "C♮" => Some(IntegerPitchClass(0)),
            "C♯" => Some(IntegerPitchClass(1)),
            "D♭" => Some(IntegerPitchClass(1)),
            "D♮" => Some(IntegerPitchClass(2)),
            "D♯" => Some(IntegerPitchClass(3)),
            "E♭" => Some(IntegerPitchClass(3)),
            "E♮" => Some(IntegerPitchClass(4)),
            "E♯" => Some(IntegerPitchClass(5)),
            "F♭" => Some(IntegerPitchClass(4)),
            "F♮" => Some(IntegerPitchClass(5)),
            "F♯" => Some(IntegerPitchClass(6)),
            "G♭" => Some(IntegerPitchClass(6)),
            "G♮" => Some(IntegerPitchClass(7)),
            "G♯" => Some(IntegerPitchClass(8)),
            "A♭" => Some(IntegerPitchClass(8)),
            "A♮" => Some(IntegerPitchClass(9)),
            "A♯" => Some(IntegerPitchClass(10)),
            "B♭" => Some(IntegerPitchClass(10)),
            "B♮" => Some(IntegerPitchClass(11)),
            "B♯" => Some(IntegerPitchClass(0)),
            _ => None,
        }
    }

    fn to_str(p: &Self::PitchClass) -> String {
        let n = p.0 % 12;
        match n {
            0 => "C",
            1 => "C♯",
            2 => "D",
            3 => "D♯",
            4 => "E",
            5 => "F",
            6 => "F♯",
            7 => "G",
            8 => "G♯",
            9 => "A",
            10 => "A♯",
            11 => "B",
            12 => "C",
            _ => panic!(),
        }.to_string()
    }
}


impl fmt::Debug for PitchClassOctave<ChromaticPitchClassSpace> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", ChromaticPitchClassSpace::to_str(&self.0), self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes() {
        assert_eq!(ChromaticPitchClassSpace::classes(), vec![
            ChromaticPitchClassSpace::from_str("C").unwrap(),
            ChromaticPitchClassSpace::from_str("C♯").unwrap(),
            ChromaticPitchClassSpace::from_str("D").unwrap(),
            ChromaticPitchClassSpace::from_str("D♯").unwrap(),
            ChromaticPitchClassSpace::from_str("E").unwrap(),
            ChromaticPitchClassSpace::from_str("F").unwrap(),
            ChromaticPitchClassSpace::from_str("F♯").unwrap(),
            ChromaticPitchClassSpace::from_str("G").unwrap(),
            ChromaticPitchClassSpace::from_str("G♯").unwrap(),
            ChromaticPitchClassSpace::from_str("A").unwrap(),
            ChromaticPitchClassSpace::from_str("A♯").unwrap(),
            ChromaticPitchClassSpace::from_str("B").unwrap(),
        ]);
    }

    #[test]
    fn from_str() {
        assert_eq!(ChromaticPitchClassSpace::from_str("C").unwrap().0, 0);
        assert_eq!(ChromaticPitchClassSpace::from_str("E").unwrap().0, 4);
        assert_eq!(ChromaticPitchClassSpace::from_str("E♯").unwrap().0, 5);
        assert_eq!(ChromaticPitchClassSpace::from_str("F").unwrap().0, 5);
    }

    #[test]
    fn to_str() {
        let p = ChromaticPitchClassSpace::from_str("C").unwrap();
        assert_eq!(ChromaticPitchClassSpace::to_str(&p), "C");
        let p = ChromaticPitchClassSpace::from_str("C♯").unwrap();
        assert_eq!(ChromaticPitchClassSpace::to_str(&p), "C♯");
        let p = ChromaticPitchClassSpace::from_str("G♭").unwrap();
        assert_eq!(ChromaticPitchClassSpace::to_str(&p), "F♯");
        let p = ChromaticPitchClassSpace::from_str("B♮").unwrap();
        assert_eq!(ChromaticPitchClassSpace::to_str(&p), "B");
    }

    #[test]
    fn successor() {
        let p = ChromaticPitchClassSpace::from_str("C").unwrap();
        let p = ChromaticPitchClassSpace::successor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("C♯").unwrap());
        let p = ChromaticPitchClassSpace::successor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("D").unwrap());
        let p = ChromaticPitchClassSpace::successor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("D♯").unwrap());
        let p = ChromaticPitchClassSpace::successor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("E").unwrap());
        let p = ChromaticPitchClassSpace::successor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("F").unwrap());
    }

    #[test]
    fn precursor() {
        let p = ChromaticPitchClassSpace::from_str("C").unwrap();
        let p = ChromaticPitchClassSpace::precursor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("B").unwrap());
        let p = ChromaticPitchClassSpace::from_str("F").unwrap();
        let p = ChromaticPitchClassSpace::precursor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("E").unwrap());
        let p = ChromaticPitchClassSpace::from_str("A").unwrap();
        let p = ChromaticPitchClassSpace::precursor(&p);
        assert_eq!(p, ChromaticPitchClassSpace::from_str("G♯").unwrap());
    }

    #[test]
    fn pitch_class_octave_ordering() {
        let a:PitchClassOctave<ChromaticPitchClassSpace> = PitchClassOctave(ChromaticPitchClassSpace::from_str("C").unwrap(), 0);
        let b = PitchClassOctave(ChromaticPitchClassSpace::from_str("D").unwrap(), 0);
        assert!(a < b);
        let a:PitchClassOctave<ChromaticPitchClassSpace> = PitchClassOctave(ChromaticPitchClassSpace::from_str("C").unwrap(), 10);
        let b = PitchClassOctave(ChromaticPitchClassSpace::from_str("D").unwrap(), 1);
        assert!(a > b);
        assert_eq!(a.min(b), b);
    }
}
