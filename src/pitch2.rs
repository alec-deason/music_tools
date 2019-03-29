use regex::Regex;


//TODO Maybe the octive marker should be nullable where a default is clear?
#[derive(Copy, Clone, Debug)]
struct Notation(Letter, Accidental, i32);
#[derive(Copy, Clone, Debug)]
enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}
#[derive(Copy, Clone, Debug)]
enum Accidental {
    // TODO: Double/triples?
    Flat,
    Natural,
    Sharp,
}
impl Notation {
    fn new(spelling: &str) -> Option<Notation> {
        let pattern = Regex::new(r"(?P<letter>[ABCDEFG])(?P<extra_bit>[♭♮♯])?(?P<octave>\d)?$").unwrap();
        let captures = pattern.captures(spelling);

        if let Some(captures) = captures {
            let letter = match &captures["letter"] {
                "C" =>  Letter::C,
                "D" => Letter::D,
                "E" => Letter::E,
                "F" => Letter::F,
                "G" => Letter::G,
                "A" => Letter::A,
                "B" => Letter::B,
                _ => panic!(),
            };
            let accidental = match captures.get(2).map(|m| m.as_str()).unwrap_or("♮") {
                "♭" => Accidental::Flat,
                "♮" => Accidental::Natural,
                "♯" => Accidental::Sharp,
                _ => panic!(),
            };
            let octave = captures.get(3).map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
            Some(Notation(letter, accidental, octave))
        } else {
            None
        }
    }
}


// TODO: When const generics become
// available then that will be a better
// solution and T can always be zero-sized
// but for now it carries a struct which
// may or may not be zero sized.
struct Pitch<T: Tuning>(i32, T);
struct PitchClass<T: Tuning>(usize, T);

trait Tuning: Sized + Clone {
    fn to_frequency(&self, p: &Pitch<Self>) -> f32;
    fn successor(&self, p: &Pitch<Self>) -> Option<Pitch<Self>>;
    fn predecessor(&self, p: &Pitch<Self>) -> Option<Pitch<Self>>;
    fn from_spelling(&self, n: &str) -> Option<Pitch<Self>>;
    fn pitch_class(&self, p: &Pitch<Self>) -> PitchClass<Self>;
}

#[derive(Copy, Clone)]
struct TwelveEDO;
impl Tuning for TwelveEDO {
    fn to_frequency(&self, p: &Pitch<Self>) -> f32 {
        440.0 * 2.0f32.powf((p.0 - 57) as f32 / 12.0)
    }

    fn successor(&self, p: &Pitch<Self>) -> Option<Pitch<Self>> {
        Some(Pitch(p.0 + 1, Self))
    }

    fn predecessor(&self, p: &Pitch<Self>) -> Option<Pitch<Self>> {
        Some(Pitch(p.0 - 1, Self))
    }

    fn from_spelling(&self, spelling: &str) -> Option<Pitch<Self>> {
        if let Some(notation) = Notation::new(spelling) {
            let mut offset = match (notation.0, notation.1) {
                (Letter::C, Accidental::Flat) => 11,
                (Letter::C, Accidental::Natural) => 0,
                (Letter::C, Accidental::Sharp) => 1,
                (Letter::D, Accidental::Flat) => 1,
                (Letter::D, Accidental::Natural) => 2,
                (Letter::D, Accidental::Sharp) => 3,
                (Letter::E, Accidental::Flat) => 3,
                (Letter::E, Accidental::Natural) => 4,
                (Letter::E, Accidental::Sharp) => 5,
                (Letter::F, Accidental::Flat) => 4,
                (Letter::F, Accidental::Natural) => 5,
                (Letter::F, Accidental::Sharp) => 6,
                (Letter::G, Accidental::Flat) => 6,
                (Letter::G, Accidental::Natural) => 7,
                (Letter::G, Accidental::Sharp) => 8,
                (Letter::A, Accidental::Flat) => 8,
                (Letter::A, Accidental::Natural) => 9,
                (Letter::A, Accidental::Sharp) => 10,
                (Letter::B, Accidental::Flat) => 10,
                (Letter::B, Accidental::Natural) => 11,
                (Letter::B, Accidental::Sharp) => 0,
            };
            offset += 12 * notation.2;
            Some(Pitch(offset, *self))
        } else {
            None
        }
    }

    fn pitch_class(&self, p: &Pitch<Self>) -> PitchClass<Self> {
        match p.0 / 12 {
            0 =>  PitchClass(0, *self),
            1 =>  PitchClass(1, *self),
            2 =>  PitchClass(2, *self),
            3 =>  PitchClass(3, *self),
            4 =>  PitchClass(4, *self),
            5 =>  PitchClass(0, *self),
            6 =>  PitchClass(0, *self),
            7 =>  PitchClass(0, *self),
            8 =>  PitchClass(0, *self),
            9 =>  PitchClass(0, *self),
            10 => PitchClass(0, *self),
            11 => PitchClass(0, *self),
        }
    }
}

#[derive(Copy, Clone)]
struct Pythagorean(f32);
impl Tuning for Pythagorean {
    fn to_frequency(&self, p: &Pitch<Self>) -> f32 {
        let class = p.0 % 12;
        let octave = p.0 / 12;
        let ratio = match class {
            0 => 1.0,
            1 => 256.0/243.0,
            2 => 9.0/8.0,
            3 => 32.0/27.0,
            4 => 81.0/64.0,
            5 =>  4.0/3.0,
            6 =>  1024.0/729.0,
            7 =>  3.0/2.0,
            8 =>  128.0/81.0,
            9 =>  27.0/16.0,
            10 =>  16.0/9.0,
            11 =>  243.0/128.0,
            _ => panic!(),
        };
        ratio*2.0f32.powf(octave as f32)*self.0
    }

    fn successor(&self, p: &Pitch<Self>) -> Option<Pitch<Self>> {
       Some(Pitch(p.0 + 1, p.1))
    }

    fn predecessor(&self, p: &Pitch<Self>) -> Option<Pitch<Self>> {
       Some(Pitch(p.0 - 1, p.1))
    }

    fn from_spelling(&self, spelling: &str) -> Option<Pitch<Self>> {
        if let Some(notation) = Notation::new(spelling) {
            let offset = notation.2 * 12;
            match (notation.0, notation.1) {
                (Letter::C, Accidental::Natural) => Some(Pitch(0 + offset, *self)),
                (Letter::C, Accidental::Sharp) =>   Some(Pitch(1 + offset, *self)),
                (Letter::D, Accidental::Natural) => Some(Pitch(2 + offset, *self)),
                (Letter::E, Accidental::Flat) =>    Some(Pitch(3 + offset, *self)),
                (Letter::E, Accidental::Natural) => Some(Pitch(4 + offset, *self)),
                (Letter::F, Accidental::Natural) => Some(Pitch(5 + offset, *self)),
                (Letter::F, Accidental::Sharp) =>   Some(Pitch(6 + offset, *self)),
                (Letter::G, Accidental::Natural) => Some(Pitch(7 + offset, *self)),
                (Letter::G, Accidental::Sharp) =>   Some(Pitch(8 + offset, *self)),
                (Letter::A, Accidental::Natural) => Some(Pitch(9 + offset, *self)),
                (Letter::B, Accidental::Flat) =>    Some(Pitch(10 + offset, *self)),
                (Letter::B, Accidental::Natural) => Some(Pitch(11 + offset, *self)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn pitch_class(&self, p: &Pitch<Self>) -> PitchClass<Self> {
        PitchClass(0, *self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn twelve_edo() {
        let tuning = TwelveEDO;
        assert_approx_eq!(tuning.to_frequency(&tuning.from_spelling("A4").unwrap()), 440.0, 0.001);
        assert_approx_eq!(tuning.to_frequency(&tuning.from_spelling("C♯4").unwrap()), 277.18, 0.005);
        assert_approx_eq!(tuning.to_frequency(&tuning.from_spelling("C1").unwrap()), 32.7, 0.005);
    }

    #[test]
    fn pythagorean() {
        let tuning = Pythagorean(440.0);
        assert_approx_eq!(tuning.to_frequency(&tuning.from_spelling("C").unwrap()), 440.0, 0.001);
        assert_approx_eq!(tuning.to_frequency(&tuning.from_spelling("F").unwrap()), 440.0*(4.0/3.0), 0.001);
    }
}

trait Scale {
}
