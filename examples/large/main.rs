use std::{hint::black_box, time::Instant};

use select::select_nth_unstable;

mod pcg_rng;
use pcg_rng::PCGRng;

/// Generates the test data of `count` random `u32` values.
fn random_u32s(count: usize, rng: &mut PCGRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(u32::MAX as usize);
    let iter = std::iter::from_fn(move || Some(rng.u32())).take(count);
    data.extend(iter);
    data
}

/// Run the provided closure `repeat` times, timing each run and printing the
/// elapsed time for each run.
fn timeit<D, P: FnMut() -> D, F: FnMut(D)>(mut prep: P, mut test: F, repeat: usize) {
    let mut times = Vec::with_capacity(repeat);
    for _ in 0..repeat {
        let data = prep();
        let now = Instant::now();
        black_box(test(data));
        let elapsed = now.elapsed().as_secs_f32();
        times.push(elapsed);
    }
    let tot = times.iter().sum::<f32>();
    let avg = tot / times.len() as f32;
    let min = times
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max = times
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    eprintln!(
        "  {repeat} runs in {tot:.2} s: avg {avg:.6} s, min {min:.6} s, max {max:.6} s",
        repeat = repeat,
        avg = avg,
        min = min,
        max = max,
    );
}

// cargo run --example large --release
// cargo flamegraph --example large -- --release

fn main() {
    let repeat = 1000;
    let count = 10_000_000;
    let mid = count / 2;
    let mut rng = PCGRng::new(1234);

    eprintln!("Testing {count} elements");
    eprintln!("Selecting the 42nd element using the Floyd & Rivest algorithm ...");
    timeit(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), 42);
        },
        repeat,
    );

    let mut rng = PCGRng::new(1234);
    eprintln!("Selecting the 42nd element using std::slice::select_nth_unstable ...");
    timeit(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            data.select_nth_unstable(42);
        },
        repeat,
    );

    let mut rng = PCGRng::new(1234);
    eprintln!("Selecting the median element using the Floyd & Rivest algorithm ...");
    timeit(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), mid);
        },
        repeat,
    );

    let mut rng = PCGRng::new(1234);
    eprintln!("Selecting the median element using std::slice::select_nth_unstable ...");
    timeit(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            data.select_nth_unstable(mid);
        },
        repeat,
    );
}
