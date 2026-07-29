use vibarena::Arena;

pub fn main() {
    let mut arena = Arena::new();

    // Inserting a value creates a unique key for the value.
    let index = arena.insert(("John", 38));

    assert!(arena.len() == 1);

    // Retrieving a value by key returns a reference to the value.
    // This is optional, since the key could be invalid if the object was removed.
    let (name, age) = arena.get(index).unwrap();
    assert!(*name == "John");
    assert!(*age == 38);

    arena.get_mut(index).unwrap().1 += 1;

    assert!(arena.get(index).unwrap().1 == 39);

    // The arena iterates only the values by default.
    for (_name, _age) in &arena {
        // println!("{}: {}", _name, _age);
    }

    // Iterating with keys:
    for (_key, _value) in arena.iter_keyed() {
        // println!("{}: {}", _key.idx(), _value.1);
    }

    arena.clear();
    let index_2 = arena.insert(("Steve", 40));
    // Reinserting values reuses the index in the internal vector,
    // but the generation counter is incremented.
    assert!(index_2.idx() == index.idx() && index_2.ver() != index.ver());
    // The old key is now invalid.
    assert!(arena.get(index).is_none());
}
