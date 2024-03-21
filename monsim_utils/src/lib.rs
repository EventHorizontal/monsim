use std::{cell::Cell, collections::VecDeque, ops::{Add, Deref, DerefMut, Index, IndexMut, Mul, Not, Sub}, slice::{Iter, IterMut}};

/// Type alias for readability of parentheses
pub type Nothing = ();
/// Type alias for readability of parentheses
pub const NOTHING: () = ();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Success,
    Failure,
}

impl From<bool> for Outcome {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Success,
            false => Self::Failure,
        }
    }
}

impl From<Outcome> for bool {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Success => true,
            Outcome::Failure => false,
        }
    }
}

impl Not for Outcome {
    type Output = Outcome;

    fn not(self) -> Self::Output {
        match self {
            Outcome::Success => Outcome::Failure,
            Outcome::Failure => Outcome::Success,
        }
    }
}

impl Outcome {
    pub fn succeeded(self) -> bool {
        self.into()
    }

    pub fn failed(self) -> bool {
        not!(self.succeeded())
    }
}

/// A percentage that is unbound above
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(pub u16);
impl Percent {
    pub fn is_matchup_ineffective(&self) -> bool {
        *self == Percent(0)
    }

    pub fn is_matchup_effective(&self) -> bool {
        *self == Percent(100)
    }

    pub fn is_matchup_not_very_effective(&self) -> bool {
        *self == Percent(25) || *self == Percent(50)
    }

    pub fn is_matchup_super_effective(&self) -> bool {
        *self == Percent(200) || *self == Percent(400)
    }
}

impl Add for Percent {
    type Output = Percent;

    fn add(self, rhs: Self) -> Self::Output {
        Percent(self.0 + rhs.0)
    }
}

impl Sub for Percent {
    type Output = Percent;

    fn sub(self, rhs: Self) -> Self::Output {
        Percent(self.0.saturating_sub(rhs.0))
    }
}

impl Mul for Percent {
    type Output = Percent;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 as f64 * (rhs.0 as f64 / 100.0f64)) as u16)
    }
}

impl Mul<f64> for Percent {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        rhs * (self.0 as f64 / 100.0f64)
    }
}

impl Mul<Percent> for f64 {
    type Output = f64;

    fn mul(self, rhs: Percent) -> Self::Output {
        self * (rhs.0 as f64 / 100.0f64)
    }
}

/// A percentage that must be between 0 and 100
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClampedPercent(u16);

impl ClampedPercent {
    pub fn from(value: u16) -> Self {
        assert!(
            value <= 100,
            "ClampedPercent only takes values between 0 and 100. If you want unbound percentages, use Percent instead."
        );
        Self(value)
    }
}

impl Add for ClampedPercent {
    type Output = ClampedPercent;

    fn add(self, rhs: Self) -> Self::Output {
        ClampedPercent(self.0 + rhs.0).min(ClampedPercent(100))
    }
}

impl Sub for ClampedPercent {
    type Output = ClampedPercent;

    fn sub(self, rhs: Self) -> Self::Output {
        ClampedPercent(self.0.saturating_sub(rhs.0))
    }
}

impl Mul<f64> for ClampedPercent {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        rhs * (self.0 as f64 / 100.0f64)
    }
}

impl Mul<ClampedPercent> for f64 {
    type Output = f64;

    fn mul(self, rhs: ClampedPercent) -> Self::Output {
        self * (rhs.0 as f64 / 100.0f64)
    }
}

#[test]
fn test_percent_type() {
    let fifty_percent = ClampedPercent(50);
    assert_eq!(5.0 * fifty_percent, 2.5f64);
    assert_eq!(fifty_percent * 5.0, 2.5f64);
    assert_eq!(fifty_percent + ClampedPercent(51), ClampedPercent(100));
    assert_eq!(fifty_percent - ClampedPercent(51), ClampedPercent(0));
}

#[macro_export]
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}

/// It's an array-backed vector (importantly for our use case it implements Copy) of capacity `CAP` where the elements are guaranteed to be at the beginning. _This may change in the future_ but panics if indexed outside of valid elements. It is meant for use cases with up to ~100 elements. 
/// 
/// How this internally works: Makes an array with default members for padding, and keeps track of a cursor that indicates the number of valid elements. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxSizedVec<T, const CAP: usize> {
    elements: [T; CAP],
    count: usize,
}

impl<T: Clone, const CAP: usize> MaxSizedVec<T, CAP> {
    pub fn from_slice(elements: &[T]) -> Self {
        let count = elements.len();
        assert!(count <= CAP, "Error: Attempted to create a FrontLoadedArray with a slice of length {count}, which is greater than the expected size {CAP}");

        let placeholder = elements.first().expect("Expected a non-empty vector but vector is empty.").clone();

        let elements = {
            let out: [T; CAP] = core::array::from_fn(|i| {
                // Fill the front of the array with the slice elements
                if i < count {
                    elements[i].clone()
                // Fill the rest of the array with dummy default values.
                } else {
                    placeholder.clone()
                }
            } );
            out
        };
        
        Self {
            elements,
            count,
        }
    }

    pub fn from_vec(mut elements: Vec<T>) -> Self {
        let count = elements.len();
        assert!(count <= CAP, "Error: Attempted to create a MaxSizedVec with a slice of length {count}, which is greater than the expected size {CAP}");
        elements.reverse();

        let placeholder = elements.first().expect("Expected a non-empty vector but vector is empty.").clone();

        let elements = {
            let out: [T; CAP] = core::array::from_fn(|i| {
                // Fill the front of the array with the slice elements
                if i < count {
                    elements.pop().expect("Expected an element because the loop is manually synchronised with the number of elements in `elements`")
                // Fill the rest of the array with dummy default values.
                } else {
                    placeholder.clone()
                }
            } );
            out
        };
        
        Self {
            elements,
            count,
        }
    }

    pub fn push(&mut self, item: T) {
        self.elements[self.count - 1] = item;
        self.count += 1;
    }

    /// Fails if the array is full.
    pub fn try_push(&mut self, item: T) -> Result<(), &'static str> {
        *self.elements.get_mut(self.count - 1).ok_or("Push failed due to array being full.")? = item;
        self.count += 1;
        Ok(NOTHING)
    }

    pub fn pop(&mut self) -> T {
        let popped_element = self.elements[self.count - 1].clone();
        self.count -= 1;
        popped_element
    }
    
    pub fn map<U, F>(self, mut f: F) -> MaxSizedVec<U, CAP> 
        where F: FnMut(T) -> U + Clone
    {
        let items = self.elements.map(|item| {f(item)});
        MaxSizedVec {
            elements: items,
            count: self.count
        }
    }
    
    pub fn iter(&self) -> Iter<T> {
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.elements.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.count
    }
    
    pub fn extend(&mut self, new_elements: &[T]) {
        let number_of_new_elements = new_elements.len();
        assert!(self.count + number_of_new_elements <= CAP, "FLArray has {} elements and cannot be extended by {} more elements.", self.count, number_of_new_elements);

        for element in new_elements.into_iter() {
            self.push(element.clone());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
}

impl<T, const CAP: usize> Index<usize> for MaxSizedVec<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.count {
            &self.elements[index]
        } else {
            panic!("FLArray was indexed beyond valid elements.")
        }
    }
}

impl<T: Default, const CAP: usize> Default for MaxSizedVec<T, CAP> {
    fn default() -> Self {
        let elements = {
            let out: [T; CAP] = core::array::from_fn(|_| { T::default() });
            out
        };
        Self { elements, count: Default::default() }
    }
}

impl<T: Copy + Clone + Default, const CAP: usize> MaxSizedVec<T, CAP> {
    pub const fn placeholder(placeholder_element: T) -> Self {
        let elements = [placeholder_element; CAP];
        MaxSizedVec { elements, count: 1 }
    }
}

impl<T, const CAP: usize> IndexMut<usize> for MaxSizedVec<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.count {
            &mut self.elements[index]
        } else {
            panic!("FLArray was indexed beyond valid elements.")
        }
    }
}

/// Iterates over the valid elements.
impl<T, const CAP: usize> IntoIterator for MaxSizedVec<T, CAP>{
    type Item = T;

    type IntoIter = std::iter::Take<std::array::IntoIter<T, CAP>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter().take(self.count)
    }
}

/// Makes `!` more readable
#[macro_export]
macro_rules! not {
    ($x: expr) => {
        !$x
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A type that can be deferenced to get data marked as belonging to the Ally Team
pub struct Ally<T>(pub T);

impl<T> Deref for Ally<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl<T> DerefMut for Ally<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Ally<T> {
    pub fn map<U, F>(self, f: F) -> Ally<U> where F: FnOnce(T) -> U {
        let item = f(self.0);
        Ally(item)
    }
}

impl<T> AsRef<T> for Ally<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Ally<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Into<TeamAffil<T>> for Ally<T> {
    fn into(self) -> TeamAffil<T> {
        TeamAffil::Ally(self)
    }
}

impl<T> Ally<T> {
    pub fn unwrap(self) -> T {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A type that can be deferenced to get data marked as belonging to the Opponent Team
pub struct Opponent<T>(pub T);

impl<T> Deref for Opponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl<T> DerefMut for Opponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Opponent<T> {
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<T> Opponent<T> {
    pub fn map<U, F>(self, f: F) -> Opponent<U> where F: FnOnce(T) -> U {
        let item = f(self.0);
        Opponent(item)
    }
}


impl<T> AsRef<T> for Opponent<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Opponent<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Into<TeamAffil<T>> for Opponent<T> {
    fn into(self) -> TeamAffil<T> {
        TeamAffil::Opponent(self)
    }
}

pub enum TeamAffil<T> {
    Ally(Ally<T>),
    Opponent(Opponent<T>)
}

impl<T> TeamAffil<T> {
    pub fn ally(item: Ally<T>) -> Self {
        Self::Ally(item)
    }

    pub fn opponent(item: Opponent<T>) -> Self {
        Self::Opponent(item)
    }
    
    pub fn apply<U, F>(&self, f: F) -> U where F: FnOnce(&T) -> U {
        match self {
            TeamAffil::Ally(a) => f(&**a),
            TeamAffil::Opponent(o) => f(&**o),
        }
    }

    pub fn expect_ally(self) -> Ally<T> {
        match self {
            TeamAffil::Ally(a) => a,
            TeamAffil::Opponent(_) => panic!(),
        }
    }

    pub fn expect_opponent(self) -> Opponent<T> {
        match self {
            TeamAffil::Ally(_) => panic!(),
            TeamAffil::Opponent(o) => o,
        }
    }
    
    pub fn unwrap(self) -> T {
        match self {
            TeamAffil::Ally(a) => a.0,
            TeamAffil::Opponent(o) => o.0,
        }
    }
}

impl<T> TeamAffil<T> {
    pub fn map<U, F>(self, f: F) -> TeamAffil<U> 
        where F: FnOnce(T) -> U
    {
        match self {
            TeamAffil::Ally(a) => TeamAffil::Ally(a.map(f)),
            TeamAffil::Opponent(o) => TeamAffil::Opponent(o.map(f)),
        }
    }
}

impl<T: Clone> Deref for TeamAffil<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            TeamAffil::Ally(a) => &a,
            TeamAffil::Opponent(o) => &o,
        }
    }
}

pub trait ModifyCell<T: Copy> {
    fn modify<F>(&self, f: F) where F: FnOnce(&mut T);
    fn modify_and_return<F>(&self, f: F) -> T 
        where F: FnOnce(&mut T);
}

impl<T: Copy> ModifyCell<T> for Cell<T> {
    fn modify<F>(&self, f: F) 
        where F: FnOnce(&mut T) 
    {
        let mut value = self.get();
        f(&mut value);
        self.set(value);
    }

    /// Modifies the inner value and then returns the new value
    fn modify_and_return<F>(&self, f: F) -> T 
        where F: FnOnce(&mut T) 
    {
        let mut value = self.get();
        f(&mut value);
        self.set(value);
        value
    }
}