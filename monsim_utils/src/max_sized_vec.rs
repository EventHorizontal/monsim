use std::{ops::{Index, IndexMut, Range}, slice::{Iter, IterMut}};

use crate::NOTHING;

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

    pub fn with_new_cap<const NEW_CAP: usize>(elements: Self) -> MaxSizedVec<T, NEW_CAP> {
        let count = elements.count();
        assert!(count < NEW_CAP, "Error: Attempted to create a MaxSizedVec with a slice of length {count}, which is greater than the expected size {CAP}");
        let mut elements = elements.to_vec();
        elements.reverse();

        let placeholder = elements.first().expect("Expected a non-empty vector but vector is empty.").clone();

        let elements = {
            let out: [T; NEW_CAP] = core::array::from_fn(|i| {
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
        
        MaxSizedVec {
            elements,
            count,
        }
    }

    pub fn push(&mut self, item: T) {
        self.elements[self.count] = item;
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
        self.elements[0..self.count].iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.elements[0..self.count].iter_mut()
    }

    pub fn count(&self) -> usize {
        self.count
    }
    
    pub fn extend(&mut self, new_elements: &[T]) {
        let number_of_new_elements = new_elements.len();
        assert!(self.count + number_of_new_elements <= CAP, "MaxSizedVec has {} elements and cannot be extended by {} more elements.", self.count, number_of_new_elements);

        for element in new_elements.into_iter() {
            self.push(element.clone());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    fn to_vec(self) -> Vec<T> {
        self.elements[0..self.count].to_vec()
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        self.elements.get(index)
    }      
}

impl<T, const CAP: usize> Index<usize> for MaxSizedVec<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.count {
            &self.elements[index]
        } else {
            panic!("MaxSizedVec was indexed beyond valid elements.")
        }
    }
}

impl<T, const CAP: usize> Index<Range<usize>> for MaxSizedVec<T, CAP> {
    type Output = [T];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        let last_index = index.clone().last().unwrap();
        if last_index < self.count {
            &self.elements[index]
        } else {
            panic!("MaxSizedVec was indexed beyond valid elements.")
        }
    }
}

impl<T, const CAP: usize> IndexMut<usize> for MaxSizedVec<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.count {
            &mut self.elements[index]
        } else {
            panic!("MaxSizedVec was indexed beyond valid elements.")
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

/// Iterates over the valid elements.
impl<T, const CAP: usize> IntoIterator for MaxSizedVec<T, CAP>{
    type Item = T;

    type IntoIter = std::iter::Take<std::array::IntoIter<T, CAP>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter().take(self.count)
    }
}

impl<T: Default, const CAP:usize> MaxSizedVec<T, CAP> {
    pub fn empty() -> Self {
        MaxSizedVec {
            elements: std::array::from_fn::<_, CAP, _>(|_| {
                T::default()
            }),
            count: 0,
        }
    }
}