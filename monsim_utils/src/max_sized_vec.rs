#[cfg(test)]
mod tests;

use crate::NOTHING;
use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut, Range},
};

/// It's an array-backed vector (importantly for our use case it implements Copy) of capacity `CAP` where the elements are guaranteed to be at the beginning. _This may change in the future_ but panics if indexed outside of valid elements. It is meant for use cases with up to ~100 elements.
///
/// How this internally works: Makes an array with default members for padding, and keeps track of a cursor that indicates the number of valid elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxSizedVec<T, const CAP: usize> {
    elements: [Option<T>; CAP],
    count: usize,
}

impl<T: Display, const CAP: usize> Display for MaxSizedVec<T, CAP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![f, "["]?;
        for maybe_element in self.elements.iter() {
            if let Some(element) = maybe_element {
                write![f, " {},", element]?;
            } else {
                break;
            }
        }
        write![f, "]"]
    }
}

impl<T: Clone, const CAP: usize> MaxSizedVec<T, CAP> {
    pub fn from_slice(elements: &[T]) -> Self {
        let count = elements.len();
        assert!(
            count <= CAP,
            "Error: Attempted to create a FrontLoadedArray with a slice of length {count}, which is greater than the expected size {CAP}"
        );

        let elements = {
            let out: [Option<T>; CAP] = core::array::from_fn(|i| {
                // Fill the front of the array with the slice elements
                if i < count {
                    Some(elements[i].clone())
                // Fill the rest of the array with dummy default values.
                } else {
                    None
                }
            });
            out
        };

        Self { elements, count }
    }

    pub fn extend_clone(&mut self, new_elements: &[T]) {
        let number_of_new_elements = new_elements.len();
        assert!(
            self.count + number_of_new_elements <= CAP,
            "MaxSizedVec has {} elements and cannot be extended by {} more elements.",
            self.count,
            number_of_new_elements
        );

        for element in new_elements.into_iter() {
            self.push(element.clone());
        }
    }

    // Removes the element at `index` and shifts over the rest of the elements.
    pub fn remove(&mut self, index: usize) {
        assert!(
            index <= self.count,
            "Expected index to be less than element count, {}, found {} instead",
            self.count,
            index
        );
        for index2 in index..self.count {
            self.elements[index2] = self.elements[index2 + 1].clone();
        }
        self.elements[self.count] = None;
        self.count -= 1;
    }

    pub fn clear(&mut self) {
        *self = MaxSizedVec::empty();
    }
}

impl<T: PartialEq, const CAP: usize> MaxSizedVec<T, CAP> {
    pub fn contains(&self, item: &T) -> bool {
        self.elements
            .iter()
            .any(|element| if let Some(element) = element { *element == *item } else { false })
    }
}

impl<T, const CAP: usize> MaxSizedVec<T, CAP> {
    pub fn empty() -> Self {
        MaxSizedVec {
            elements: std::array::from_fn::<_, CAP, _>(|_| None),
            count: 0,
        }
    }

    pub fn from_vec(mut elements: Vec<T>) -> Self {
        let count = elements.len();
        assert!(
            count <= CAP,
            "Error: Attempted to create a MaxSizedVec with a slice of length {count}, which is greater than the expected size {CAP}"
        );
        elements.reverse();

        let elements = {
            let out: [Option<T>; CAP] = core::array::from_fn(|i| {
                // Fill the front of the array with the slice elements
                if i < count {
                    Some(
                        elements
                            .pop()
                            .expect("Expected an element because the loop is manually synchronised with the number of elements in `elements`"),
                    )
                // Fill the rest of the array with dummy default values.
                } else {
                    None
                }
            });
            out
        };

        Self { elements, count }
    }

    pub fn with_new_cap<const NEW_CAP: usize>(elements: Self) -> MaxSizedVec<T, NEW_CAP> {
        let count = elements.count();
        assert!(
            count < NEW_CAP,
            "Error: Attempted to create a MaxSizedVec with a slice of length {count}, which is greater than the expected size {CAP}"
        );
        let mut elements = elements.to_vec();
        elements.reverse();

        let elements = {
            let out: [Option<T>; NEW_CAP] = core::array::from_fn(|i| {
                // Fill the front of the array with the slice elements
                if i < count {
                    Some(
                        elements
                            .pop()
                            .expect("Expected an element because the loop is manually synchronised with the number of elements in `elements`"),
                    )
                // Fill the rest of the array with dummy default values.
                } else {
                    None
                }
            });
            out
        };

        MaxSizedVec { elements, count }
    }

    pub fn push(&mut self, item: T) {
        self.elements[self.count] = Some(item);
        self.count += 1;
    }

    /// Fails if the array is full.
    pub fn try_push(&mut self, item: T) -> Result<(), &'static str> {
        *self.elements.get_mut(self.count - 1).ok_or("Push failed due to array being full.")? = Some(item);
        self.count += 1;
        Ok(NOTHING)
    }

    pub fn pop(&mut self) -> T {
        let popped_element = std::mem::replace(&mut self.elements[self.count - 1], None);
        let popped_element = popped_element.expect("The index should be pointing to the last Some variant in the array");
        self.count -= 1;
        popped_element
    }

    pub fn map<U, F>(self, mut f: F) -> MaxSizedVec<U, CAP>
    where
        F: FnMut(T) -> U,
    {
        let items = self.elements.map(|element| element.map(|element| f(element)));
        MaxSizedVec {
            elements: items,
            count: self.count,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements[0..self.count].iter().flatten()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.elements[0..self.count].iter_mut().flatten()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn extend(&mut self, new_elements: Vec<T>) {
        let number_of_new_elements = new_elements.len();
        assert!(
            self.count + number_of_new_elements <= CAP,
            "MaxSizedVec has {} elements and cannot be extended by {} more elements.",
            self.count,
            number_of_new_elements
        );

        for (i, element) in new_elements.into_iter().enumerate() {
            self.elements[self.count + i] = Some(element)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn to_vec(self) -> Vec<T> {
        self.into_iter().collect()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.elements[index].as_ref()
    }
}

impl<T, const CAP: usize> Index<usize> for MaxSizedVec<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.count {
            self.elements[index].as_ref().expect("Index has been checked.")
        } else {
            panic!(
                "MaxSizedVec was indexed at {} ({}-th element) having only {} valid elements.",
                index,
                index + 1,
                self.count
            )
        }
    }
}

impl<T, const CAP: usize> IndexMut<usize> for MaxSizedVec<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.count {
            self.elements[index].as_mut().expect("Index has been checked.")
        } else {
            panic!(
                "MaxSizedVec was indexed at {} ({}-th element) having only {} valid elements.",
                index,
                index + 1,
                self.count
            )
        }
    }
}

impl<T, const CAP: usize> Index<Range<usize>> for MaxSizedVec<T, CAP> {
    type Output = [Option<T>];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.elements[index]
    }
}

impl<T, const CAP: usize> Default for MaxSizedVec<T, CAP> {
    fn default() -> Self {
        Self::empty()
    }
}

/// Iterates over the valid elements.
impl<T, const CAP: usize> IntoIterator for MaxSizedVec<T, CAP> {
    type Item = T;

    type IntoIter = std::iter::Flatten<std::iter::Take<std::array::IntoIter<Option<T>, CAP>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter().take(self.count).flatten()
    }
}
