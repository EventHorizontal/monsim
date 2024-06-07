use std::ops::{BitAnd, Not};

use crate::{not, Nothing, NOTHING};

/// An outcome with a payload, usually nothing, _i.e._ `()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome<P> {
    Success(P),
    Failure,
}

impl From<bool> for Outcome<Nothing> {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Success(NOTHING),
            false => Self::Failure,
        }
    }
}

impl<T> From<Outcome<T>> for bool {
    fn from(value: Outcome<T>) -> Self {
        match value {
            Outcome::Success(_) => true,
            Outcome::Failure => false,
        }
    }
}

impl Not for Outcome<Nothing> {
    type Output = Outcome<Nothing>;

    fn not(self) -> Self::Output {
        match self {
            Outcome::Success(_) => Outcome::Failure,
            Outcome::Failure => Outcome::Success(NOTHING),
        }
    }
}

impl BitAnd for Outcome<Nothing> {
    type Output = Outcome<Nothing>;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self.is_success() && rhs.is_success() {
            Outcome::Success(NOTHING)
        } else {
            Outcome::Failure
        }
    }
}

impl<T> Outcome<T> {
    #[inline(always)]
    pub fn is_success(self) -> bool {
        self.into()
    }

    #[inline(always)]
    pub fn is_failure(self) -> bool {
        not!(self.is_success())
    }

    #[inline(always)]
    pub fn opposite(self) -> Outcome<Nothing> {
        match self {
            Outcome::Success(_) => Outcome::Failure,
            Outcome::Failure => Outcome::Success(NOTHING),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Outcome::Success(payload) => payload,
            Outcome::Failure => default,
        }
    }

    /// Empties the contents of the `Outcome`.
    pub fn empty(self) -> Outcome<Nothing> {
        match self {
            Outcome::Success(_) => Outcome::Success(NOTHING),
            Outcome::Failure => Outcome::Failure,
        }
    }

    pub fn map<F, U>(self, transform: F) -> Outcome<U>
    where
        F: Fn(T) -> U,
    {
        match self {
            Outcome::Success(payload) => Outcome::Success(transform(payload)),
            Outcome::Failure => Outcome::Failure,
        }
    }
}
