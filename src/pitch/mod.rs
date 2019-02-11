use std::cmp::Ordering;

use ordered_float::OrderedFloat;

mod log_linear;
mod chromatic;

trait Pitch: Eq + Ord {}

trait PitchSpace {
    type Pos;
    type Dist;
    type Pitch: Pitch;

    fn from_frequency(f: f32) -> Self::Pitch;
    fn to_frequency(p: &Self::Pitch) -> f32;
    fn distance(a: &Self::Pitch, b: &Self::Pitch) -> Self::Dist;
}

trait PitchClass: Eq {}
trait PitchClassSpace {
    type PitchClass: PitchClass;

    fn classes() -> Vec<Self::PitchClass>;
    fn successor(p: &Self::PitchClass) -> Self::PitchClass;
    fn precursor(p: &Self::PitchClass) -> Self::PitchClass;
    fn from_str(n: &str) -> Option<Self::PitchClass>;
    fn to_str(n: &Self::PitchClass) -> String;
}

type Octave = usize;
struct PitchOctave<C: PitchClassSpace>(C::PitchClass, Octave);


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct IntegerPitchClass(usize);
impl PitchClass for IntegerPitchClass {}


struct EqualTempermentSemitone(f32);
impl Pitch for EqualTempermentSemitone {}

impl ::std::ops::Add for EqualTempermentSemitone {
    type Output = EqualTempermentSemitone;

    fn add(self, rhs: EqualTempermentSemitone) -> Self::Output {
        EqualTempermentSemitone(self.0 + rhs.0)
    }
}

impl ::std::ops::Sub for EqualTempermentSemitone {
    type Output = EqualTempermentSemitone;

    fn sub(self, rhs: EqualTempermentSemitone) -> Self::Output {
        EqualTempermentSemitone(self.0 - rhs.0)
    }
}

impl ::std::ops::Add<f32> for EqualTempermentSemitone {
    type Output = EqualTempermentSemitone;

    fn add(self, rhs: f32) -> Self::Output {
        EqualTempermentSemitone(self.0 + rhs)
    }
}

impl ::std::ops::Sub<f32> for EqualTempermentSemitone {
    type Output = EqualTempermentSemitone;

    fn sub(self, rhs: f32) -> Self::Output {
        EqualTempermentSemitone(self.0 - rhs)
    }
}

impl PartialEq for EqualTempermentSemitone {
    fn eq(&self, other: &EqualTempermentSemitone) -> bool {
        self.0 == other.0
    }
}
impl Eq for EqualTempermentSemitone {}

impl ::std::cmp::PartialOrd for EqualTempermentSemitone {
    fn partial_cmp(&self, other: &EqualTempermentSemitone) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EqualTempermentSemitone {
    fn cmp(&self, other: &EqualTempermentSemitone) -> Ordering {
        OrderedFloat(self.0).cmp(&OrderedFloat(other.0))
    }
}
