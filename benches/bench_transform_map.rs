use std::collections::HashMap;
use criterion::{criterion_group, criterion_main, Criterion};
use keympostor::key_action::*;
use keympostor::key_code::*;
use keympostor::key_code::{VirtualKey};
use keympostor::transform::KeyTransformMap;

pub fn benchmark_transform_map(c: &mut Criterion) {
    let mut map = KeyTransformMap::new();

    for key in &VIRTUAL_KEYS {
        let source = KeyAction {
            key: Key::from_virtual_key(key),
            ..Default::default()
        };
        map.put(source, KeyActionSequence::from(vec![]));
    }

    let actual = KeyAction {
        key: Key::from_virtual_key(&VIRTUAL_KEYS[14]),
        ..Default::default()
    };

    c.bench_function("TransformMap.get", |b| {
        b.iter(|| {
            let _ = map.get(&actual);
        })
    });
}

pub fn benchmark_map(c: &mut Criterion) {
    let mut map = HashMap::new();

    for key in VIRTUAL_KEYS {
        map.insert(key, KeyActionSequence::from(vec![]));
    }

    let actual = VIRTUAL_KEYS[42];

    c.bench_function("Map.get", |b| {
        b.iter(|| {
            let _v = map.get(&actual);
        })
    });
}

pub fn benchmark_array(c: &mut Criterion) {
    let mut array:[VirtualKey; 255] = [INVALID_VIRTUAL_KEY; 255];

    for key in VIRTUAL_KEYS {
        array[key.value as usize] = key;
    }

    let actual = VIRTUAL_KEYS[42];

    c.bench_function("Map.get", |b| {
        b.iter(|| {
            let _v = array[actual.value as usize];
        })
    });
}

criterion_group!(benches, benchmark_transform_map);
// criterion_group!(benches, benchmark_array);
criterion_main!(benches);
