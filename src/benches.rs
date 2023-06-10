use crate::{pcg_rng::PCGRng, select_nth_unstable};

fn shuffle<T>(data: &mut [T], rng: &mut PCGRng) {
    let len = data.len();
    let ptr = data.as_mut_ptr();
    for i in 0..len - 1 {
        let j = rng.bounded_usize(i, len);
        unsafe {
            let a = ptr.add(i);
            let b = ptr.add(j);
            std::ptr::swap(a, b);
        }
    }
}

/// Returns a vector of `count` random `u32` values.
fn random_u32s(count: usize, rng: &mut PCGRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    while data.len() < count {
        data.push(rng.u32());
    }
    data
}

/// Returns a vector of `count` random `u32` values in the range `0..sqrt(count)`.
fn random_u32s_dups(count: usize, rng: &mut PCGRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let sqrt_count = (count as f64).sqrt() as u32;
    while data.len() < count {
        data.push(rng.bounded_u32(0, sqrt_count));
    }
    data
}

/// Returns a vector of integers in the range `0..count`, in reversed order.
fn reversed_u32s(count: usize, _rng: &mut PCGRng) -> Vec<u32> {
    (0..count as u32).rev().collect()
}

/// Returns a vector of `u32`s in the range `0..count`, in random order.
fn shuffled_u32s(count: usize, rng: &mut PCGRng) -> Vec<u32> {
    let mut data: Vec<_> = (0..count as u32).collect();
    shuffle(&mut data, rng);
    data
}

/// Returns a vector of `u32`s with a sawtooth pattern.
fn sawtooth_u32s(count: usize, _rng: &mut PCGRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let count = count as u32;
    let sqrt_count = (count as f64).sqrt() as u32;
    for index in 0..count {
        let x = index % sqrt_count;
        data.push(x);
    }
    data
}

#[derive(Debug)]
struct Timings {
    runs: usize,
    total: f32,
    throughput: f32,
    avg: f32,
    min: f32,
    max: f32,
    median: f32,
}

impl Timings {
    fn from_times(data: &[f32]) -> Self {
        let runs = data.len();
        let total: f32 = data.iter().sum();
        let avg = total / runs as f32;
        let min = *data
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max = *data
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let mut data = data.to_vec();
        let (_, &mut median, _) =
            data.select_nth_unstable_by(runs / 2, |a, b| a.partial_cmp(b).unwrap());
        Self {
            runs,
            total,
            throughput: runs as f32 / total,
            avg,
            min,
            max,
            median,
        }
    }
}

#[derive(Debug)]
struct Comparison {
    label: String,
    throughput_ratio: f32,
    timings: Timings,
    baseline: Timings,
}

impl Comparison {
    fn new(label: impl Into<String>, timings: Timings, baseline: Timings) -> Self {
        Self {
            label: label.into(),
            throughput_ratio: timings.throughput / baseline.throughput,
            timings,
            baseline,
        }
    }
}

/// Runs `func` and `baseline` repeatedly with data prepared by `prep` until `func` has run for at
/// least `duration` seconds. Prints the number of runs, the total time, and the average, minimum,
/// and maximum times for each closure. Also prints the throughput of `func` relative to `baseline`.
///
/// The `prep` closure is ignored in the timing.
fn bench<D, P: FnMut() -> D, A: FnMut(D), B: FnMut(D)>(
    mut prep: P,
    mut func: A,
    mut baseline: B,
    duration: f32,
) -> (Timings, Timings) {
    use std::hint::black_box;
    use std::time::Instant;

    eprintln!("Running the function for at least {duration:.2} seconds. The runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.");

    let mut times = Vec::new();
    let mut times_baseline = Vec::new();
    let mut total = 0.0;
    let mut rng = PCGRng::new(0);
    while total < duration {
        let data = prep();
        if rng.u64() < u64::MAX / 2 {
            let now = Instant::now();
            func(black_box(data));
            let elapsed = now.elapsed().as_secs_f32();
            times.push(elapsed);
            total += elapsed;
        } else {
            let now = Instant::now();
            baseline(black_box(data));
            let elapsed = now.elapsed().as_secs_f32();
            times_baseline.push(elapsed);
        }
    }
    let timings = Timings::from_times(&times);
    let timings_baseline = Timings::from_times(&times_baseline);
    (timings, timings_baseline)
}

fn compare<P, T>(
    label: impl Into<String>,
    mut prep: P,
    count: usize,
    index: usize,
    duration: f32,
) -> Comparison
where
    P: FnMut(usize, &mut PCGRng) -> Vec<T>,
    T: Ord,
{
    let mut rng = PCGRng::new(1234);
    let label = label.into();
    eprintln!("Testing {} ...", label);
    let (timings, baseline) = bench(
        || prep(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), index);
        },
        |mut data| {
            data.select_nth_unstable(index);
        },
        duration,
    );
    let comparison = Comparison::new(label, timings, baseline);
    eprintln!("{:#?}", comparison);
    comparison
}

#[test]
#[ignore]
fn perf_tests() {
    // cargo test -r perf_tests -- --nocapture --ignored

    fn run<P>(label: &str, prep: P)
    where
        P: FnMut(usize, &mut PCGRng) -> Vec<u32> + Copy,
    {
        let duration = 5.0;
        let counts = [100_000_000, 1_000_000, 10_000];
        let p50 = |count: usize| count / 2;
        let p25 = |count: usize| count / 4;
        let p01 = |count: usize| count / 100;

        for count in counts {
            let index = p01(count);
            let label = format!("{label} (count = {count}, index = {index})",);
            compare(&label, prep, count, index, duration);

            let index = p25(count);
            let label = format!("{label} (count = {count}, index = {index})",);
            compare(&label, prep, count, index, duration);
            
            let index = p50(count);
            let label = format!("{label} (count = {count}, index = {index})",);
            compare(&label, prep, count, index, duration);
        }
    }

    run("random_u32s", random_u32s);
    run("shuffled_u32s", shuffled_u32s);
    run("sawtooth_u32s", sawtooth_u32s);
    run("reversed_u32s", reversed_u32s);
    run("random_u32s_dups", random_u32s_dups);
}
