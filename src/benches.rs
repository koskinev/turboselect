#[cfg(feature = "std")]
extern crate std;

use std::vec::Vec;

use crate::{select_nth_unstable, sort::tinysort, wyrand::WyRng};

/// Returns a vector of integers where approximately 90% of the elements are in sorted order. The
/// maximum is randomized and in the range `0..count`.
fn mostly_sorted_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let max = rng.bounded_u32(0, count as u32);
    for index in 0..count {
        data.push((max * index as u32) / (count as u32));
    }
    for _ in 0..count / 10 {
        let index = rng.bounded_usize(0, count);
        let value = rng.bounded_u32(0, count as u32);
        data[index] = value;
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

/// Returns a vector of `count` random `u32` values.
fn random_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    while data.len() < count {
        data.push(rng.u32());
    }
    data
}

/// Returns a vector of `count` random `u32` values in the range `0..sqrt(count)`.
fn random_dups_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let sqrt_count = (count as f64).sqrt() as u32;
    while data.len() < count {
        data.push(rng.bounded_u32(0, sqrt_count));
    }
    data
}

/// Returns a vector of integers in reversed order. The maximum is randomized and in the range
/// `0..count`.
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

/// Returns a vector of integers in sorted order. The maximum is randomized and in the range
/// `0..count`.
fn sorted_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(count);
    let max = rng.bounded_u32(0, count as u32);
    for index in 0..count {
        data.push((max * index as u32) / (count as u32));
    }
    data
}

#[derive(Default)]
struct Durations {
    ours: Vec<u128>,
    baseline: Vec<u128>,
}

impl Durations {
    const BILLION: u128 = 1_000_000_000;
    const MILLION: u128 = 1_000_000;

    fn throughputs(&self, len: usize) -> (f64, f64) {
        let our_througput = ((len * self.ours.len()) as f64)
            / ((Self::MILLION * self.ours.iter().sum::<u128>()) as f64 / Self::BILLION as f64);
        let baseline_througput = ((len * self.baseline.len()) as f64)
            / ((Self::MILLION * self.baseline.iter().sum::<u128>()) as f64 / Self::BILLION as f64);
        (our_througput, baseline_througput)
    }
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
fn turboselect_perf() {
    // cargo test -r turboselect_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- turboselect_perf --ignored
    use std::{eprintln, format};

    fn run<P, T>(label: &str, mut prep: P)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        use colored::*;
        use std::io::Write;

        let lens = [
            1_000, 10_000, 100_000, // 1_000_000, 10_000_000, 100_000_000
        ];
        let percentiles = [0.001, 0.01, 0.05, 0.25, 0.5];
        let percentile = |count: usize, p: f64| (count as f64 * p) as usize;
        let runs = |len: usize| 1_000_000 / ((len as f32).sqrt() as usize);
        let mut rng = WyRng::new(123456789);

        let mut compare = |len, index| {
            bench(
                || prep(len, rng.as_mut()),
                |data| {
                    select_nth_unstable(data, index);
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

        // std::fs::create_dir_all("bench_results").unwrap();
        // let mut output = std::fs::File::create(format!("bench_results/{label}.csv")).unwrap();
        let mut results = Vec::new();
        writeln!(results, "target,len,percentile,nanosecs").unwrap();
        for len in lens {
            for p in percentiles {
                let index = percentile(len, p);
                let durations = compare(len, index);
                let (our_tput, baseline_tput) = durations.throughputs(len);
                let ratio = our_tput / baseline_tput;
                let ratio = if ratio > 1.0 {
                    format!("{:5.03}", ratio).green()
                } else {
                    format!("{:5.03}", ratio).red()
                };
                eprintln!(
                    "| {label:<18} | {len:<12} | {index:<11} | {our_tput:<20.03} | {baseline_tput:<18.03} | {ratio} |",
                );

                for duration in &durations.ours {
                    writeln!(
                        results,
                        "turboselect,{len},p{ptile},{duration}",
                        ptile = 100. * p,
                    )
                    .unwrap();
                }
                for duration in &durations.baseline {
                    writeln!(
                        results,
                        "core::slice,{len},p{ptile},{duration}",
                        ptile = 100. * p,
                    )
                    .unwrap();
                }
            }
        }
        // output.write_all(&results).unwrap();
    }

    eprintln!("Benchmarking turboselect against core::slice::select_nth_unstable. The runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.\n");

    eprintln!("| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |");
    eprintln!("| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |");

    run("random_u32", random_u32s);
    run("sorted_u32", sorted_u32s);
    run("mostlysort_u32", mostly_sorted_u32s);
    run("sawtooth_u32", sawtooth_u32s);
    run("reversed_u32", reversed_u32s);
    run("randomdup_u32", random_dups_u32s);
    run("random_bool", random_bools);
}

#[test]
#[ignore]
fn tinysort_perf() {
    // cargo test -r tinysort_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- tinysort_perf --ignored
    use colored::*;
    use std::{eprintln, format};

    fn run<P, T>(label: &str, mut prep: P)
    where
        P: FnMut(usize, &mut WyRng) -> Vec<T> + Copy,
        T: Ord,
    {
        let lens = [4, 8, 16, 24, 32, 64];
        let runs = |len: usize| 2_000_000 / ((len as f32).sqrt() as usize);
        let mut rng = WyRng::new(123456789);

        let mut compare = |len| {
            bench(
                || prep(len, rng.as_mut()),
                |data| {
                    tinysort(data, &mut T::lt);
                },
                |data| data.sort_unstable(),
                |data| data.windows(2).all(|w| w[0] <= w[1]),
                runs(len),
            )
        };

        for len in lens {
            let durations = compare(len);
            let (our_tput, baseline_tput) = durations.throughputs(len);
            let ratio = our_tput / baseline_tput;
            let ratio = if ratio > 1.0 {
                format!("{:5.03}", ratio).green()
            } else {
                format!("{:5.03}", ratio).red()
            };
            eprintln!(
                    "| {label:<18} | {len:<12} | {our_tput:<20.03} | {baseline_tput:<18.03} | {ratio:<5.03} |",
                );
        }
    }

    eprintln!("Benchmarking tinysort against core::slice::sort_unstable. The runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.\n");

    eprintln!(
        "| data type          | slice length | throughput, M el/s   | baseline, M el /s  | ratio |"
    );
    eprintln!(
        "| ------------------ | ------------ | -------------------- | ------------------ | ----- |"
    );

    run("random_u32", random_u32s);
    run("sawtooth_u32", sawtooth_u32s);
    run("reversed_u32", reversed_u32s);
    run("randomdups_u32", random_dups_u32s);
    run("random_bool", random_bools);
}
