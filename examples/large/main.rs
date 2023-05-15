use std::{hint::black_box, time::Instant};

use select::select_nth_unstable;

mod pcg_rng;
use pcg_rng::PCGRng;

/// Generates the test data of `count` random `u32` values.
fn random_u32s(count: usize) -> Vec<u32> {
    let mut data = Vec::with_capacity(u32::MAX as usize);
    let mut rng = PCGRng::new(data.as_ptr() as u64);
    let iter = std::iter::from_fn(move || Some(rng.u32())).take(count);
    data.extend(iter);
    data
}

/// Run the provided closure `repeat` times, timing each run and printing the
/// elapsed time for each run.
fn timeit<F: FnMut()>(mut f: F, repeat: usize) {
    let mut times = Vec::with_capacity(repeat);
    for _ in 0..repeat {
        let now = Instant::now();
        black_box(f());
        let elapsed = now.elapsed().as_secs_f32();
        times.push(elapsed);
    }
    let avg = times.iter().sum::<f32>() / times.len() as f32;
    let min = times
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max = times
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    eprintln!(
        "  {repeat} runs: avg {avg} seconds, min {min} seconds, max {max} seconds",
        repeat = repeat,
        avg = avg,
        min = min,
        max = max,
    );
}

// cargo run --example large --release
// cargo flamegraph --example large -- --release

fn main() {
    eprint!("Generating data...");
    let count = 10_000_000;
    let repeat = 1000;
    let master = random_u32s(count);
    eprintln!("{} elements", master.len());

    eprintln!("Selecting the 42nd element using the Floyd & Rivest algorithm ...");

    timeit(
        || {
            let mut data = master.clone();
            select_nth_unstable(data.as_mut_slice(), 42);
        },
        repeat,
    );

    // eprintln!("Selecting the 42nd element using std::slice::select_nth_unstable ...");
    // timeit(
    //     || {
    //         let mut data = master.clone();
    //         data.select_nth_unstable(42);
    //     },
    //     repeat,
    // );

    // let mid = master.len() / 2;
    // eprintln!("Selecting the median element using the Floyd & Rivest algorithm ...");
    // timeit(
    //     || {
    //         let mut data = master.clone();
    //         select_nth_unstable(data.as_mut_slice(), mid);
    //     },
    //     repeat,
    // );

    // eprintln!("Selecting the median element using std::slice::select_nth_unstable ...");
    // timeit(
    //     || {
    //         let mut data = master.clone();
    //         data.select_nth_unstable(mid);
    //     },
    //     repeat,
    // );
}
