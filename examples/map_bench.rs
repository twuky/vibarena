use std::time::Duration;

fn main() {
    for len in [100_000, 1_000_000, 10_000_000] {
        println!("len={}: ", len);
        let mut map_results = Vec::new();
        let mut std_map_results = Vec::new();

        for _ in 0..10 {
            let (map_time, std_map_time) = bench(len);
            map_results.push(map_time);
            std_map_results.push(std_map_time);
        }
        let map_time = map_results.iter().sum::<Duration>() / map_results.len() as u32;
        let std_map_time = std_map_results.iter().sum::<Duration>() / std_map_results.len() as u32;

        println!("  map: {:?}, arena: {:?}", map_time, std_map_time);
    }
}

fn bench(len: i32) -> (Duration, Duration) {
    let mut map = vibarena::map::<i32>();
    let mut arena = vibarena::Arena::new();
    let mut std_map = std::collections::HashMap::new();

    for i in 0..len {
        let _key = arena.insert(i);
    }

    let t1 = std::time::Instant::now();
    for k in arena.iter_keyed() {
        map.insert(k.0, *k.1);
    }
    let t2 = std::time::Instant::now();

    let t3 = std::time::Instant::now();
    for k in arena.iter_keyed() {
        std_map.insert(k.0, *k.1);
    }
    let t4 = std::time::Instant::now();

    (t2 - t1, t4 - t3)
}
