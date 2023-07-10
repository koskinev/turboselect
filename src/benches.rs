#[cfg(feature = "std")]
extern crate std;

use std::{eprint, eprintln, format, vec::Vec};

use crate::{miniselect, partition_in_blocks, quickselect, wyrand::WyRng};

const MILLION: u128 = 1_000_000;
const BILLION: u128 = 1_000_000_000;

/// Returns a random boolean vector with `count` elements.
fn random_bools(count: usize, rng: &mut WyRng) -> Vec<bool> {
    let mut data = Vec::with_capacity(count);
    while data.len() < count {
        data.push(rng.bool());
    }
    data
}

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

#[derive(Default)]
struct Durations {
    ours: Vec<u128>,
    baseline: Vec<u128>,
}

/// Runs `func` and `baseline` repeatedly with data prepared by `prep` until `func` has run for at
/// least `runs` times. Prints the number of runs, the total time, and the average, minimum,
/// and maximum times for each closure. Also prints the throughput of `func` relative to `baseline`.
///
/// The `prep` closure is ignored in the timing.
fn bench<D, P: FnMut() -> D, T: FnMut(&mut D), B: FnMut(&mut D), C: FnMut(D) -> bool>(
    mut prep: P,
    mut ours: T,
    mut baseline: B,
    mut check: C,
    runs: usize,
) -> Durations {
    use std::hint::black_box;
    use std::time::Instant;

    let mut durations = Durations::default();
    let mut rng = WyRng::new(123456789);
    while durations.ours.len() < runs {
        let mut data = prep();
        if rng.u64() < u64::MAX / 2 {
            let now = Instant::now();
            ours(black_box(&mut data));
            let elapsed = now.elapsed().as_nanos();
            durations.ours.push(elapsed);
            assert!(black_box(check(data)));
        } else {
            let now = Instant::now();
            baseline(black_box(&mut data));
            let elapsed = now.elapsed().as_nanos();
            durations.baseline.push(elapsed);
            assert!(black_box(check(data)));
        }
    }
    durations
}

#[test]
#[ignore]
fn quickselect_perf() {
    // cargo test -r quickselect_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- quickselect_perf --ignored

    eprintln!(
        "Comparing quickselect with custom pivot selection to core::slice::select_nth_unstable."
    );
    eprintln!("The tests and the baseline runs are randomly interleaved. Data preparation is ignored in the timing.\n");

    eprintln!("| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |");
    eprintln!("| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |");

    fn run<P, T>(label: &str, mut prep: P)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        let lens = [1_000, 10_000 /* 1_000_000, 100_000_000 */];
        let percentiles = [0.001, 0.01, 0.05, 0.25, 0.5];
        let percentile = |count: usize, p: f64| (count as f64 * p) as usize;
        let runs = |len: usize| 1_000_000 / ((len as f32).sqrt() as usize);

        let mut rng_a = WyRng::new(123456789);
        let mut rng_b = WyRng::new(987654321);

        let mut compare = |len, index| {
            bench(
                || prep(len, rng_a.as_mut()),
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
                runs(len),
            )
        };

        const MILLION: u128 = 1_000_000;
        for len in lens {
            for p in percentiles {
                let index = percentile(len, p);
                let durations = compare(len, index);
                let our_runs = durations.ours.len();
                let baseline_runs = durations.baseline.len();
                let our_time = durations.ours.iter().sum::<u128>();
                let baseline_time = durations.baseline.iter().sum::<u128>();
                let throughput =
                    (len * our_runs) as f64 / ((MILLION * our_time) as f64 / BILLION as f64);
                let baseline = (len * baseline_runs) as f64
                    / ((MILLION * baseline_time) as f64 / BILLION as f64);
                let ratio = throughput / baseline;
                eprintln!(
                    "| {label:<18} | {len:<12} | {index:<11} | {throughput:<20.03} | {baseline:<18.03} | {ratio:<5.03} |",
                );
            }
        }
    }

    run("random (u32)", random_u32s);
    run("sawtooth (u32)", sawtooth_u32s);
    run("reversed (u32)", reversed_u32s);
    run("random dups (u32s)", random_u32s_dups);
    run("random (bool)", random_bools);
}

#[test]
#[ignore]
fn quickselect_rec_perf() {
    // cargo test -r quickselect_rec_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- quickselect_rec_perf --ignored

    fn run<P, T>(label: &str, mut prep: P)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        use std::io::Write;

        eprint!("Benchmarking {label} ..");

        let lens = [1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];
        let percentiles = [0.001, 0.01, 0.05, 0.25, 0.5];
        let percentile = |count: usize, p: f64| (count as f64 * p) as usize;
        let runs = |len: usize| 1_000_000 / ((len as f32).sqrt() as usize);

        let mut rng_a = WyRng::new(123456789);
        let mut rng_b = WyRng::new(987654321);

        let mut compare = |len, index| {
            bench(
                || prep(len, rng_a.as_mut()),
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
                runs(len),
            )
        };

        let mut results = Vec::new();
        for len in lens {
            for p in percentiles {
                let index = percentile(len, p);
                let durations = compare(len, index);
                writeln!(results, "target,len,index,nanosecs").unwrap();
                for duration in &durations.ours {
                    writeln!(results, "ours,{len},{index},{duration}").unwrap();
                }
                for duration in &durations.baseline {
                    writeln!(results, "baseline,{len},{index},{duration}").unwrap();
                }
            }
            eprint!(".")
        }

        let mut output = std::fs::File::create(format!("{label}.csv")).unwrap();
        output.write_all(&results).unwrap();

        eprintln!("wrote {label}.csv");
    }

    run("random_u32", random_u32s);
    run("sawtooth_u32", sawtooth_u32s);
    run("reversed_u32", reversed_u32s);
    run("randomdups_u32", random_u32s_dups);
    run("random_bool", random_bools);
}

#[test]
#[ignore]
fn miniselect_perf() {
    // cargo test -r miniselect_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- miniselect_perf --ignored

    eprintln!("Comparing Wirth selection with core::slice::select_nth_unstable.");
    eprintln!("The tests and the baseline runs are randomly interleaved. Data preparation is ignored in the timing.\n");

    eprintln!("| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |");
    eprintln!("| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |");

    fn run<P, T>(label: &str, mut prep: P)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        let lens = [8, 12, 16, 24];
        let percentiles = [0.05, 0.25, 0.5, 0.27, 0.95];
        let percentile = |count: usize, p: f64| (count as f64 * p) as usize;
        let mut rng = WyRng::new(123456789);

        let mut compare = |count, index| {
            bench(
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
                100_000,
            )
        };

        for len in lens {
            for p in percentiles {
                let index = percentile(len, p);
                let durations = compare(len, index);
                let our_runs = durations.ours.len();
                let baseline_runs = durations.baseline.len();
                let our_time = durations.ours.iter().sum::<u128>();
                let baseline_time = durations.baseline.iter().sum::<u128>();
                let throughput =
                    (len * our_runs) as f64 / ((MILLION * our_time) as f64 / BILLION as f64);
                let baseline = (len * baseline_runs) as f64
                    / ((MILLION * baseline_time) as f64 / BILLION as f64);
                let ratio = throughput / baseline;
                eprintln!(
                    "| {label:<18} | {len:<12} | {index:<11} | {throughput:<20.03} | {baseline:<18.03} | {ratio:<5.03} |",
                );
            }
        }
    }

    run("random (u32)", random_u32s);
    run("sawtooth (u32)", sawtooth_u32s);
    run("reversed (u32)", reversed_u32s);
    run("random dups (u32s)", random_u32s_dups);
    run("random (bool)", random_bools);
}
