use crate::{quickselect, wyrand::WyRng};

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
fn reversed_u32s(count: usize, _rng: &mut WyRng) -> Vec<u32> {
    (0..count as u32).rev().collect()
}

/// Returns a vector of `u32`s with a sawtooth pattern.
fn sawtooth_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let count = count as u32;
    let max_lenght = (count as f64).sqrt() as u32;
    let length = rng.bounded_u32(max_lenght / 4, max_lenght);
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

const MIN_DURATION: f32 = 5.0;
const MIN_RUNS: usize = 200;
const MAX_RUNS: usize = 50_000;

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
    let mut total = 0.0;
    let mut rng = WyRng::new(0);
    while times.len() < MAX_RUNS && (total < MIN_DURATION || times.len() < MIN_RUNS) {
        let mut data = prep();
        if rng.u64() < u64::MAX / 2 {
            let now = Instant::now();
            test(black_box(&mut data));
            let elapsed = now.elapsed().as_secs_f32();
            times.push(elapsed);
            total += elapsed;
            assert!(black_box(check(data)));
        } else {
            let now = Instant::now();
            baseline(black_box(&mut data));
            let elapsed = now.elapsed().as_secs_f32();
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

    eprintln!(
        "Comparing quickselect with custom pivot selection to core::slice::select_nth_unstable."
    );
    eprintln!("Running each benchmark for at least {MIN_DURATION:.2} seconds and at least {MIN_RUNS} times. The tests and the baseline runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.\n");

    eprintln!("| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |");
    eprintln!("| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |");

    fn run<P, T>(label: &str, mut prep: P, runs: &mut Vec<Comparison>)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        let counts = [1_000, 10_000, /*1_000_000, 100_000_000*/];
        let p001 = |count: usize| count / 1000;
        let p01 = |count: usize| count / 100;
        let p05 = |count: usize| count / 20;
        let p25 = |count: usize| count / 4;
        let p50 = |count: usize| count / 2;
        let percentiles = [p001, p01, p05, p25, p50];

        let mut rng_a = WyRng::new(0);
        let mut rng_b = WyRng::new(0);

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

        for count in counts {
            for p in percentiles {
                let index = p(count);
                let comparison = compare(
                    count,
                    index,
                    format!("{label} (count = {count}, index = {index})",),
                );
                eprintln!(
                    "| {label:<18} | {count:<12} | {index:<11} | {tput:<20.02} | {baseline:<18.02} | {ratio:<5.03} |",
                    tput = comparison.timings.throughput,
                    baseline = comparison.baseline.throughput,
                    ratio = comparison.throughput_ratio
                );
                runs.push(comparison);
            }
        }
    }

    run("random (bool)", random_bools, &mut runs);
    run("random (u32)", random_u32s, &mut runs);
    run("sawtooth (u32)", sawtooth_u32s, &mut runs);
    run("reversed (u32)", reversed_u32s, &mut runs);
    run("random dups (u32s)", random_u32s_dups, &mut runs);
}
