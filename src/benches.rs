use crate::{miniselect, quickselect, wyrand::WyRng};

/// Returns a vector of `count` random `u32` values.
fn random_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    while data.len() < count {
        data.push(rng.u32());
    }
    data
}

/// Returns a vector of `count` random `u32` values in the range `0..sqrt(count)`.
fn random_u32s_dups(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let sqrt_count = (count as f64).sqrt() as u32;
    while data.len() < count {
        data.push(rng.bounded_u32(0, sqrt_count));
    }
    data
}

/// Returns a vector of integers in the range `0..count`, in reversed order.
fn reversed_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let max = rng.bounded_u32(0, count as u32);
    for index in 0..count {
        data.push((max * (count - index + 1) as u32) / (count as u32));
    }
    data
}

/// Returns a vector of `u32`s with a sawtooth pattern.
fn sawtooth_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let count = count as u32;
    let max_lenght = (count as f64).sqrt() as u32;
    let length = rng.bounded_u32(max_lenght / 4 + 1, max_lenght);
    for index in 0..count {
        let x = index % length;
        data.push(x);
    }
    data
}

/// Returns a random boolean vector with `count` elements.
fn random_bools(count: usize, rng: &mut WyRng) -> Vec<bool> {
    let mut data = Vec::with_capacity(count);
    while data.len() < count {
        data.push(rng.bool());
    }
    data
}

#[derive(Debug)]
struct Timings {
    runs: usize,
    nanosecs: u128,
}

impl Timings {
    fn from_times(data: &[u128]) -> Self {
        let runs = data.len();
        let nanosecs = data.iter().sum::<u128>();
        Self { runs, nanosecs }
    }
}

#[derive(Debug)]
struct Comparison {
    label: String,
    timings: Timings,
    baseline: Timings,
}

impl Comparison {
    fn new(label: impl Into<String>, timings: Timings, baseline: Timings) -> Self {
        Self {
            label: label.into(),
            timings,
            baseline,
        }
    }
}

const BILLION: u128 = 1_000_000_000;
const MIN_DURATION: u128 = 5 * BILLION;
const MIN_RUNS: usize = 250;
const MAX_RUNS: usize = 100_000;

/// Runs `func` and `baseline` repeatedly with data prepared by `prep` until `func` has run for at
/// least `duration` seconds. Prints the number of runs, the total time, and the average, minimum,
/// and maximum times for each closure. Also prints the throughput of `func` relative to `baseline`.
///
/// The `prep` closure is ignored in the timing.
fn bench<D, P: FnMut() -> D, T: FnMut(&mut D), B: FnMut(&mut D), C: FnMut(D) -> bool>(
    label: impl Into<String>,
    mut prep: P,
    mut test: T,
    mut baseline: B,
    mut check: C,
) -> Comparison {
    use std::hint::black_box;
    use std::time::Instant;

    let mut times = Vec::new();
    let mut times_baseline = Vec::new();
    let mut total = 0;
    let mut rng = WyRng::new(123456789);
    while times.len() < MAX_RUNS && (total < MIN_DURATION || times.len() < MIN_RUNS) {
        let mut data = prep();
        if rng.u64() < u64::MAX / 2 {
            let now = Instant::now();
            test(black_box(&mut data));
            let elapsed = now.elapsed().as_nanos();
            times.push(elapsed);
            total += elapsed;
            assert!(black_box(check(data)));
        } else {
            let now = Instant::now();
            baseline(black_box(&mut data));
            let elapsed = now.elapsed().as_nanos();
            times_baseline.push(elapsed);
            assert!(black_box(check(data)));
        }
    }
    let timings = Timings::from_times(&times);
    let timings_baseline = Timings::from_times(&times_baseline);
    Comparison::new(label, timings, timings_baseline)
}

#[test]
#[ignore]
fn quickselect_perf() {
    // cargo test -r quickselect_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- quickselect_perf --ignored

    let mut runs = Vec::new();
    let min_duration = MIN_DURATION as f64 / BILLION as f64;

    eprintln!(
        "Comparing quickselect with custom pivot selection to core::slice::select_nth_unstable."
    );
    eprintln!("Running each benchmark for at least {min_duration:.2} seconds and at least {MIN_RUNS} times. The tests and the baseline runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.\n");

    eprintln!("| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |");
    eprintln!("| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |");

    fn run<P, T>(label: &str, mut prep: P, runs: &mut Vec<Comparison>)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        let lens = [100, 1_000, 10_000, /* 1_000_000, 100_000_000 */ ];
        let p001 = |count: usize| count / 1000;
        let p01 = |count: usize| count / 100;
        let p05 = |count: usize| count / 20;
        let p25 = |count: usize| count / 4;
        let p50 = |count: usize| count / 2;
        let percentiles = [p001, p01, p05, p25, p50];

        let mut rng_a = WyRng::new(123456789);
        let mut rng_b = WyRng::new(987654321);

        let mut compare = |count, index, label| {
            bench(
                label,
                || prep(count, rng_a.as_mut()),
                |data| {
                    quickselect(data.as_mut_slice(), index, &mut T::lt, rng_b.as_mut());
                },
                |data| {
                    data.select_nth_unstable(index);
                },
                |data| {
                    let nth = &data[index];
                    data[..index].iter().all(|x| x <= nth) && data[index..].iter().all(|x| x >= nth)
                },
            )
        };

        for len in lens {
            for p in percentiles {
                let index = p(len);
                let comparison = compare(
                    len,
                    index,
                    format!("{label} (len = {len}, index = {index})",),
                );
                let throughput = comparison.timings.runs as f64
                    / (comparison.timings.nanosecs as f64 / BILLION as f64);
                let baseline = comparison.baseline.runs as f64
                    / (comparison.baseline.nanosecs as f64 / BILLION as f64);
                let ratio = throughput / baseline;
                eprintln!(
                    "| {label:<18} | {len:<12} | {index:<11} | {throughput:<20.03} | {baseline:<18.03} | {ratio:<5.03} |",
                );
                runs.push(comparison);
            }
        }
    }

    run("random (u32)", random_u32s, &mut runs);
    run("sawtooth (u32)", sawtooth_u32s, &mut runs);
    run("reversed (u32)", reversed_u32s, &mut runs);
    run("random dups (u32s)", random_u32s_dups, &mut runs);
    run("random (bool)", random_bools, &mut runs);
}

#[test]
#[ignore]
fn miniselect_perf() {
    // cargo test -r miniselect_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- miniselect_perf --ignored

    let mut runs = Vec::new();
    let min_duration = MIN_DURATION as f64 / BILLION as f64;

    eprintln!("Comparing Wirth selection with core::slice::select_nth_unstable.");
    eprintln!("Running each benchmark for at least {min_duration:.2} seconds and at least {MIN_RUNS} times. The tests and the baseline runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.\n");

    eprintln!("| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |");
    eprintln!("| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |");

    fn run<P, T>(label: &str, mut prep: P, runs: &mut Vec<Comparison>)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        let lens = [8, 12, 16, 24];
        let p05 = |count: usize| count / 20;
        let p25 = |count: usize| count / 4;
        let p50 = |count: usize| count / 2;
        let p75 = |count: usize| count * 3 / 4;
        let p95 = |count: usize| count * 19 / 20;
        let percentiles = [p05, p25, p50, p75, p95];

        let mut rng = WyRng::new(123456789);

        let mut compare = |count, index, label| {
            bench(
                label,
                || prep(count, rng.as_mut()),
                |data| {
                    miniselect(data, index, &mut T::lt);
                },
                |data| {
                    data.select_nth_unstable(index);
                },
                |data| {
                    let nth = &data[index];
                    data[..index].iter().all(|x| x <= nth) && data[index..].iter().all(|x| x >= nth)
                },
            )
        };

        for len in lens {
            for p in percentiles {
                let index = p(len);
                let comparison = compare(
                    len,
                    index,
                    format!("{label} (len = {len}, index = {index})",),
                );
                let throughput = comparison.timings.runs as f64
                    / (comparison.timings.nanosecs as f64 / BILLION as f64);
                let baseline = comparison.baseline.runs as f64
                    / (comparison.baseline.nanosecs as f64 / BILLION as f64);
                let ratio = throughput / baseline;
                eprintln!(
                    "| {label:<18} | {len:<12} | {index:<11} | {throughput:<20.03} | {baseline:<18.03} | {ratio:<5.03} |",
                );
                runs.push(comparison);
            }
        }
    }

    run("random (u32)", random_u32s, &mut runs);
    run("sawtooth (u32)", sawtooth_u32s, &mut runs);
    run("reversed (u32)", reversed_u32s, &mut runs);
    run("random dups (u32s)", random_u32s_dups, &mut runs);
    run("random (bool)", random_bools, &mut runs);
}
