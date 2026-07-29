

# vibarena
Simple generational arena built based on the naive slotmap from the generational arena benchmarks: https://github.com/mooman219/generational_arena_bench for use in Vibbit and related projects.

This is modified to use NonZeroU32 for generations, meaning that an Option<Key> is the same size in memory as a Key.

## Peformance
Internally this stores values in a tightly packed vec, so retrieving elements and iteration is very fast, but insertion/removal are not O(1).

You can refer to the naive-slotmap in the [generational arena benchmarks](https://github.com/mooman219/generational_arena_bench) for a general idea.

The arena also offers two aliased types: `ArenaMap` and `ArenaSet`. These are included in the `nohash` feature, which is enabled by default.

These wrap Hasmap and HashSet but implement NoHashHasher for the keys, since a key is always a unique u64 representation of the index and generation.

## Usage
refer to [examples](https://github.com/twuky/vibarena/tree/main/examples/test.rs) for usage.