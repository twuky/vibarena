use std::num::NonZeroU32;
use vibarena::{Arena, Key};

#[test]
fn can_decompose_index() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    let (k, g) = i.parts();
    let generated_i = Key::new(k, g);
    assert_eq!(arena[generated_i], 42);
}

#[test]
fn can_get_live_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    assert_eq!(arena[i], 42);
}

#[test]
fn cannot_get_free_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    assert_eq!(arena.remove(i).unwrap(), 42);
    assert!(!arena.contains_key(i));
}

#[test]
fn cannot_get_other_generation_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    assert_eq!(arena.remove(i).unwrap(), 42);
    assert!(!arena.contains_key(i));
    let j = arena.try_insert(42).unwrap();
    assert!(!arena.contains_key(i));
    assert_eq!(arena[j], 42);
    assert!(i != j);
}

#[test]
fn get_mut() {
    let mut arena = Arena::new();
    let idx = arena.insert(5);
    arena[idx] += 1;
    assert_eq!(arena[idx], 6);
}

#[test]
fn try_insert_when_full() {
    let mut arena = Arena::with_capacity(1);
    arena.try_insert(42).unwrap();
    assert_eq!(arena.try_insert(42).unwrap_err(), 42);
}

#[test]
fn try_insert_with_when_full() {
    let mut arena = Arena::with_capacity(1);
    let first_index = arena.try_insert_with_key(|_| 42).ok().unwrap();
    let returned_fn = arena.try_insert_with_key(|_| 42).unwrap_err();
    assert_eq!(returned_fn(first_index), 42);
}

#[test]
fn insert_many_and_cause_doubling() {
    let mut arena = Arena::new();
    let indices: Vec<_> = (0..1000).map(|i| arena.insert(i * i)).collect();
    for (i, idx) in indices.iter().cloned().enumerate() {
        assert_eq!(arena.remove(idx).unwrap(), i * i);
        assert!(!arena.contains_key(idx));
    }
}

#[test]
fn insert_with_indicies_match() {
    let mut arena = Arena::new();
    let a = arena.insert_with_key(|idx| (40, idx));
    let b = arena.insert_with_key(|idx| (41, idx));
    let c = arena.insert_with_key(|idx| (42, idx));
    assert_eq!(arena[a].0, 40);
    assert_eq!(arena[b].0, 41);
    assert_eq!(arena[c].0, 42);
    assert_eq!(arena[a].1, a);
    assert_eq!(arena[b].1, b);
    assert_eq!(arena[c].1, c);
}

#[test]
fn try_insert_with_indicies_match() {
    let mut arena = Arena::with_capacity(3);
    let a = arena.try_insert_with_key(|idx| (40, idx)).ok().unwrap();
    let b = arena.try_insert_with_key(|idx| (41, idx)).ok().unwrap();
    let c = arena.try_insert_with_key(|idx| (42, idx)).ok().unwrap();
    assert_eq!(arena[a].0, 40);
    assert_eq!(arena[b].0, 41);
    assert_eq!(arena[c].0, 42);
    assert_eq!(arena[a].1, a);
    assert_eq!(arena[b].1, b);
    assert_eq!(arena[c].1, c);
}

#[test]
fn into_iter() {
    let mut arena = Arena::new();
    arena.insert(0);
    arena.insert(1);
    arena.insert(2);
    let set: std::collections::BTreeSet<_> = arena.into_iter().collect();
    assert_eq!(set.len(), 3);
    assert!(set.contains(&0));
    assert!(set.contains(&1));
    assert!(set.contains(&2));
}

#[test]
fn clear_gen() {
    let mut arena = Arena::with_capacity(1);
    let idx_1 = arena.insert(1);
    arena.clear();
    let idx_2 = arena.insert(2);
    assert_ne!(idx_1, idx_2);

    // If there are no elements, do not increment generation.
    let mut arena_2 = Arena::with_capacity(1);
    arena_2.clear();
    arena_2.clear();
    arena_2.clear();
    let idx_1 = arena_2.insert(1);
    let ver = idx_1.ver();
    assert_eq!(ver, NonZeroU32::new(1).unwrap());
}