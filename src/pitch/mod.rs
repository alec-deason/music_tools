use std::fmt::Debug;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

use ordered_float::OrderedFloat;

pub mod midi;
pub mod chromatic;

trait Pitch: Eq + Ord {}

pub trait PitchSpace {
    type Pos;
    type Dist;
    type Pitch: Pitch;

    fn from_frequency(f: f32) -> Self::Pitch;
    fn to_frequency(p: &Self::Pitch) -> f32;
    fn distance(a: &Self::Pitch, b: &Self::Pitch) -> Self::Dist;
}

pub trait PitchConverter
   where
      Self::PitchSpace: PitchSpace,
      Self::PitchClassSpace: PitchClassSpace {
    type PitchSpace;
    type PitchClassSpace;

    fn to_pitch(p: &PitchClassOctave<Self::PitchClassSpace>) -> <<Self as PitchConverter>::PitchSpace as PitchSpace>::Pitch;
}

pub trait PitchClass: Eq + Ord + Copy + Clone + Debug + Hash {}
pub trait PitchClassSpace {
    type PitchClass: PitchClass;

    fn classes() -> Vec<Self::PitchClass>;
    fn successor(p: &Self::PitchClass) -> Self::PitchClass;
    fn precursor(p: &Self::PitchClass) -> Self::PitchClass;
    fn from_str(n: &str) -> Option<Self::PitchClass>;
    fn to_str(n: &Self::PitchClass) -> String;
}

type Octave = i32;
pub struct PitchClassOctave<C: PitchClassSpace>(pub C::PitchClass, pub Octave);
impl<C: PitchClassSpace> PitchClassOctave<C> {
    pub fn new(n: &str, o: Octave) -> Self {
        let pc = C::from_str(n).unwrap();
        PitchClassOctave(pc, o)
    }
}
impl<C: PitchClassSpace> PartialEq for PitchClassOctave<C> {
    fn eq(&self, other: &PitchClassOctave<C>) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl <C: PitchClassSpace> Eq for PitchClassOctave<C> {}

impl<C: PitchClassSpace> PartialOrd for PitchClassOctave<C> {
    fn partial_cmp(&self, other: &PitchClassOctave<C>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: PitchClassSpace> Ord for PitchClassOctave<C> {
    fn cmp(&self, other: &PitchClassOctave<C>) -> Ordering {
        (self.1, self.0).cmp(&(other.1, other.0))
    }
}

impl<C: PitchClassSpace> Hash for PitchClassOctave<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<C: PitchClassSpace> Copy for PitchClassOctave<C> {}
impl<C: PitchClassSpace> Clone for PitchClassOctave<C> {
    fn clone(&self) -> PitchClassOctave<C> {
        PitchClassOctave(self.0, self.1)
    }
}





#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct IntegerPitchClass(pub usize);
impl PitchClass for IntegerPitchClass {}


#[derive(Copy, Clone, Debug)]
pub struct Semitone(pub f32);
impl Pitch for Semitone {}

impl ::std::ops::Add for Semitone {
    type Output = Semitone;

    fn add(self, rhs: Semitone) -> Self::Output {
        Semitone(self.0 + rhs.0)
    }
}

impl ::std::ops::Sub for Semitone {
    type Output = Semitone;

    fn sub(self, rhs: Semitone) -> Self::Output {
        Semitone(self.0 - rhs.0)
    }
}

impl ::std::ops::Add<f32> for Semitone {
    type Output = Semitone;

    fn add(self, rhs: f32) -> Self::Output {
        Semitone(self.0 + rhs)
    }
}

impl ::std::ops::Sub<f32> for Semitone {
    type Output = Semitone;

    fn sub(self, rhs: f32) -> Self::Output {
        Semitone(self.0 - rhs)
    }
}

impl PartialEq for Semitone {
    fn eq(&self, other: &Semitone) -> bool {
        self.0 == other.0
    }
}
impl Eq for Semitone {}

impl ::std::cmp::PartialOrd for Semitone {
    fn partial_cmp(&self, other: &Semitone) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Semitone {
    fn cmp(&self, other: &Semitone) -> Ordering {
        OrderedFloat(self.0).cmp(&OrderedFloat(other.0))
    }
}
