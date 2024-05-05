use crate::MaxSizedVec;

#[should_panic]
#[test]
fn test_if_full_max_sized_vec_allows_more_elements() {
    let mut m: MaxSizedVec<i32, 3> = MaxSizedVec::from_vec(vec![1,2,3]);
    m.push(4);
}

#[should_panic]
#[test]
fn test_if_partially_filled_max_size_vec_will_allow_indexing_into_none_elements() {
    let mut m: MaxSizedVec<i32, 16> = MaxSizedVec::from_vec(vec![1,2,3]);
    m[3];
}

#[test]
fn test_if_partially_filled_max_size_vec_will_allow_indexing_into_last_some_element() {
    let mut m: MaxSizedVec<i32, 16> = MaxSizedVec::from_vec(vec![1,2,3]);
    m[2];
}

#[test]
fn test_instantiating_new_max_sized_vec() {
    let mut m: MaxSizedVec<i32, 5> = MaxSizedVec::from_vec(vec![1,2,3]);
    assert_eq!(m.get(0), Some(1).as_ref());
    assert_eq!(m.get(1), Some(2).as_ref());
    assert_eq!(m.get(2), Some(3).as_ref());
    assert_eq!(m.get(3), None);
    assert_eq!(m.get(4), None);
    assert_eq!(m.count(), 3)
}