use criterion::{criterion_group, criterion_main, Criterion};
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;

pub(crate) fn benchmark(c: &mut Criterion) {
    c.bench_function("GetKeyboardState", |b| {
        b.iter(|| {
            let mut keyboard_state: [u8; 256] = [0u8; 256];
            unsafe { GetKeyboardState(&mut keyboard_state) }.unwrap();
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
