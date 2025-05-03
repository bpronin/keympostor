use criterion::{criterion_group, criterion_main, Criterion};
use keympostor::key_action::*;
use keympostor::key_code::*;
use keympostor::transform::TransformMap;

#[inline]
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn benchmark_transform_map(c: &mut Criterion) {
    let mut map = TransformMap::new();

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
            map.get(&actual);
        })
    });
}

criterion_group!(benches, benchmark_transform_map);
criterion_main!(benches);
