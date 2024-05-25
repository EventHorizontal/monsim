use std::ops::{BitAnd, Deref, DerefMut, Not};

mod max_sized_vec;
pub use max_sized_vec::MaxSizedVec;
mod percentage;
pub use percentage::{ClampedPercent, Percent};

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

impl BitAnd for Outcome {
    type Output = Outcome;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self.succeeded() && rhs.succeeded() {
            Outcome::Success
        } else {
            Outcome::Failure
        }
    }
}

impl Outcome {
    #[inline(always)]
    pub fn succeeded(self) -> bool {
        self.into()
    }

    #[inline(always)]
    pub fn failed(self) -> bool {
        not!(self.succeeded())
    }

    #[inline(always)]
    pub fn opposite(self) -> Outcome {
        match self {
            Outcome::Success => Outcome::Failure,
            Outcome::Failure => Outcome::Success,
        }
    }
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

/// Makes `!` more readable
#[macro_export]
macro_rules! not {
    ($x: expr) => {
        !$x
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A type that can be deferenced to get data marked as belonging to the Ally Team
pub struct Ally<T>(T);

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

impl<T> Into<TeamAffl<T>> for Ally<T> {
    fn into(self) -> TeamAffl<T> {
        TeamAffl::Ally(self)
    }
}

impl<T> Ally<T> {
    pub const fn new(item: T) -> Self {
        Self(item)
    }

    pub fn unwrap(self) -> T {
        self.0
    }

    pub fn map_consume<U, F>(self, f: F) -> Ally<U> where F: FnOnce(T) -> U {
        let item = f(self.0);
        Ally(item)
    }
}

pub trait IntoAlly<T> {
    fn mark_as_ally(self) -> Ally<T>;
}

impl<T> IntoAlly<T> for T {
    fn mark_as_ally(self) -> Ally<T> {
        Ally::new(self)
    }
}

impl<T: Clone> Ally<T> {
    pub fn map_clone<U, F>(&self, f: F) -> Ally<U> where F: FnOnce(T) -> U {
        let item = f(self.0.clone());
        Ally(item)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A type that can be deferenced to get data marked as belonging to the Opponent Team
pub struct Opponent<T>(T);

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

impl<T> Into<TeamAffl<T>> for Opponent<T> {
    fn into(self) -> TeamAffl<T> {
        TeamAffl::Opponent(self)
    }
}

impl<T> Opponent<T> {
    pub const fn new(item: T) -> Self {
        Self(item)
    }

    pub fn unwrap(self) -> T {
        self.0
    }

    pub fn map_consume<U, F>(self, f: F) -> Opponent<U> 
        where F: FnOnce(T) -> U 
    {
        let item = f(self.0);
        Opponent(item)
    }
}

impl<T: Clone> Opponent<T> {
    pub fn map_clone<U, F>(&self, f: F) -> Opponent<U> where F: FnOnce(T) -> U {
        let item = f(self.0.clone());
        Opponent(item)
    }
}

pub trait IntoOpponent<T> {
    fn mark_as_opponent(self) -> Opponent<T>;
}

impl<T> IntoOpponent<T> for T {
    fn mark_as_opponent(self) -> Opponent<T> {
        Opponent::new(self)
    }
}

pub enum TeamAffl<T> {
    Ally(Ally<T>),
    Opponent(Opponent<T>)
}

impl<T> TeamAffl<T> {
    pub fn ally(item: Ally<T>) -> Self {
        Self::Ally(item)
    }

    pub fn opponent(item: Opponent<T>) -> Self {
        Self::Opponent(item)
    }
    
    pub fn apply<U, F>(&self, f: F) -> U where F: FnOnce(&T) -> U {
        match self {
            TeamAffl::Ally(a) => f(&**a),
            TeamAffl::Opponent(o) => f(&**o),
        }
    }

    pub fn expect_ally(self) -> Ally<T> {
        match self {
            TeamAffl::Ally(a) => a,
            TeamAffl::Opponent(_) => panic!(),
        }
    }

    pub fn expect_opponent(self) -> Opponent<T> {
        match self {
            TeamAffl::Ally(_) => panic!(),
            TeamAffl::Opponent(o) => o,
        }
    }
    
    pub fn unwrap(self) -> T {
        match self {
            TeamAffl::Ally(a) => a.0,
            TeamAffl::Opponent(o) => o.0,
        }
    }
}

impl<T> TeamAffl<T> {
    pub fn map<U, F>(self, f: F) -> TeamAffl<U> 
        where F: FnOnce(T) -> U
    {
        match self {
            TeamAffl::Ally(a) => TeamAffl::Ally(a.map_consume(f)),
            TeamAffl::Opponent(o) => TeamAffl::Opponent(o.map_consume(f)),
        }
    }
}

impl<T: Clone> Deref for TeamAffl<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            TeamAffl::Ally(a) => &a,
            TeamAffl::Opponent(o) => &o,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Count {
    Fixed(u8),
    RandomInRange{ min: u8, max: u8 }
}

