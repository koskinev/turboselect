use crate::{
    floyd_rivest_select, median_5, partition_in_blocks_dual, pcg_rng::PCGRng, quickselect, sample,
    select_min, select_nth_unstable, sort_2, sort_3, sort_4,
};

fn iter_rng(rng: &mut PCGRng, count: usize, high: usize) -> impl Iterator<Item = usize> + '_ {
    std::iter::from_fn(move || Some(rng.bounded_usize(0, high))).take(count)
}

fn shuffle<T>(data: &mut [T], rng: &mut PCGRng) {
    let len = data.len();
    for i in 0..len - 1 {
        let j = rng.bounded_usize(i, len);
        data.swap(i, j);
    }
}

/// Generates the test data of `count` random `u32` values.
fn random_u32s(count: usize, rng: &mut PCGRng) -> Vec<u32> {
    let mut data = Vec::with_capacity(u32::MAX as usize);
    while data.len() < count {
        data.push(rng.u32());
    }
    data
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
) {
    use std::hint::black_box;
    use std::time::Instant;

    eprintln!("Running the function for at least {duration:.2} seconds. The runs are randomly interleaved.");
    eprintln!("Data preparation is ignored in the timing.");

    let mut times = Vec::new();
    let mut times_baseline = Vec::new();
    let (mut total, mut total_baseline) = (0.0, 0.0);
    let mut rng = PCGRng::new(0);
    while total < duration {
        let data = prep();
        match rng.u64() % 2 {
            0 => {
                let now = Instant::now();
                func(black_box(data));
                let elapsed = now.elapsed().as_secs_f32();
                times.push(elapsed);
                total += elapsed;
            }
            1 => {
                let now = Instant::now();
                baseline(black_box(data));
                let elapsed = now.elapsed().as_secs_f32();
                times_baseline.push(elapsed);
                total_baseline += elapsed;
            }
            _ => unreachable!(),
        }
    }
    eprintln!("Function:");
    print_timings(&times);
    eprintln!("Baseline:");
    print_timings(&times_baseline);
    eprintln!(
        "Throughput is {x} of baseline",
        x = (times.len() as f32 / total) / (times_baseline.len() as f32 / total_baseline)
    );
}

/// Prints the number of runs, the total time, and the average, minimum, and maximum times.
fn print_timings(times: &[f32]) {
    let runs = times.len();
    let total: f32 = times.iter().sum();
    let avg = total / runs as f32;
    let min = times
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max = times
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    eprintln!("  {runs} runs in {total:.2} s: avg {avg:.6} s, min {min:.6} s, max {max:.6} s");
}

#[test]
fn block_dual() {
    let repeat = 1000;
    let max_count = 30;
    let mut rng = PCGRng::new(123);

    for _iter in 0..repeat {
        let count = rng.bounded_usize(1, max_count);

        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let x = rng.bounded_usize(0, count);
        let y = rng.bounded_usize(0, count);

        let (low, high) = if x < y { (&x, &y) } else { (&y, &x) };
        let (u, v) = partition_in_blocks_dual(data.as_mut_slice(), low, high, &mut usize::lt);

        assert!(data[..u].iter().all(|elem| elem < low));
        assert!(data[u..v].iter().all(|elem| low <= elem && elem <= high));
        assert!(data[v..].iter().all(|elem| elem > high));
    }
}

#[test]
fn floyd_rivest_300() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let count = 300;
    let mut k = 0;
    let mut rng = PCGRng::new(123);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        let (u, v) = floyd_rivest_select(&mut data, k, &mut usize::lt, &mut rng);
        assert!(u <= k && v >= k && v < count);
        let kth = data[k];
        assert_eq!(data[u], kth);
        assert_eq!(data[v], kth);
        assert!(data[..k].iter().all(|elem| elem <= &kth));
        assert!(data[k..].iter().all(|elem| elem >= &kth));
        k = (k + 1) % count;
    }
}

#[test]
fn large_median() {
    let mut pcg = PCGRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000_000;
    #[cfg(miri)]
    let count = 1000;

    let mid = count / 2;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(data.as_mut_slice(), &mut pcg);
    let median = select_nth_unstable(data.as_mut_slice(), mid);
    assert_eq!(median, &mid);
    assert!(data[..mid].iter().all(|elem| elem < &mid));
    assert!(data[mid + 1..].iter().all(|elem| elem > &mid));
}

#[test]
fn extreme_index() {
    let mut pcg = PCGRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000_000;
    #[cfg(miri)]
    let count = 1000;

    let index = 42;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(data.as_mut_slice(), &mut pcg);
    let nth = *select_nth_unstable(data.as_mut_slice(), index);

    assert!(data[..index].iter().all(|elem| elem < &nth));
    assert_eq!(data[index], nth);
    assert!(data[index + 1..].iter().all(|elem| elem > &nth));

    let index = count - 42;
    let nth = *select_nth_unstable(data.as_mut_slice(), index);

    assert!(data[..index].iter().all(|elem| elem < &nth));
    assert_eq!(data[index], nth);
    assert!(data[index + 1..].iter().all(|elem| elem > &nth));
}

#[test]
fn nth() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    #[cfg(not(miri))]
    let max = 10000;
    #[cfg(miri)]
    let max = 1000;

    let mut pcg = PCGRng::new(0);

    for _iter in 0..repeat {
        let count = pcg.bounded_usize(1, max);
        let high = pcg.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| pcg.bounded_usize(0, high)).collect();
        let index = pcg.bounded_usize(0, count);
        select_nth_unstable(&mut data, index);
        let nth = data[index];
        data.iter().enumerate().for_each(|(i, elem)| match i {
            i if i < index => assert!(elem <= &nth),
            i if i > index => assert!(elem >= &nth),
            _ => (),
        });
    }
}

#[test]
fn nth_small() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let max = 1000;
    let mut pcg = PCGRng::new(123);

    for _iter in 0..repeat {
        let count = pcg.bounded_usize(1, max);
        let high = pcg.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| pcg.bounded_usize(0, high)).collect();
        let index = pcg.bounded_usize(0, count);
        let (u, v) = quickselect(&mut data, index, &mut usize::lt, &mut pcg);
        let nth = data[index];
        assert_eq!(data[u], nth);
        assert_eq!(data[v], nth);
        assert!(data[..index].iter().all(|elem| elem <= &nth));
        assert!(data[index..].iter().all(|elem| elem >= &nth));
    }
}

#[test]
fn sample_n() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let len = 20;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let count = rng.bounded_usize(1, len + 1);
        let mut data: Vec<_> = (0..len).collect();
        sample(data.as_mut_slice(), count, rng.as_mut());
        for i in 0..len {
            assert!(data.contains(&i));
        }
    }
}

#[test]
fn min_10() {
    let len = 10;
    let mut rng = PCGRng::new(0);

    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 10;

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(rng.as_mut(), len, len / 2).collect();
        shuffle(data.as_mut_slice(), rng.as_mut());

        let (_u, v) = select_min(data.as_mut_slice(), &mut usize::lt);
        let min = &data[0];
        assert!(data[..=v].iter().all(|elem| elem == min));
    }
}

#[test]
fn sort2() {
    let mut data = [1, 0];
    let swapped = sort_2(data.as_mut_slice(), 0, 1, &mut i32::lt);
    assert!(swapped);
    assert_eq!(data, [0, 1]);
}

#[test]
fn sort3() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let count = 3;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        sort_3(&mut data, 0, 1, 2, &mut usize::lt);
        assert!(data[0] <= data[1]);
        assert!(data[1] <= data[2]);
    }
}

#[test]
fn sort4() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let count = 4;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        sort_4(&mut data, 0, 1, 2, 3, &mut usize::lt);
        assert!(data[0] <= data[1]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
    }
}

#[test]
fn median5() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let count = 5;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        median_5(&mut data, 0, 1, 2, 3, 4, &mut usize::lt);
        assert!(data[0] <= data[2]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
        assert!(data[2] <= data[4]);
    }
}

#[test]
#[ignore]
fn small_index_perf() {
    // cargo test -r small_index_perf -- --show-output --ignored
    // cargo flamegraph --unit-test -- small_index_perf
    let duration = 3.0;
    let count = 10_000_000;
    let mut rng = PCGRng::new(1234);

    eprintln!("Selecting the 42nd element from {count} elements.");
    eprintln!("Baseline is core::slice::select_nth_unstable.");
    bench(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), 42);
        },
        |mut data| {
            data.select_nth_unstable(42);
        },
        duration,
    );
}

#[test]
#[ignore]
fn large_median_perf() {
    // cargo test -r large_median_perf -- --nocapture
    // cargo flamegraph --unit-test -- large_median_perf
    let duration = 5.0;
    let count = 10_000_000;
    let mid = count / 2;

    let mut rng = PCGRng::new(1234);
    eprintln!("Selecting the median from {count} elements.");
    eprintln!("Baseline is core::slice::select_nth_unstable.");
    bench(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), mid);
        },
        |mut data| {
            data.select_nth_unstable(mid);
        },
        duration,
    );
}

#[test]
#[ignore]
fn small_42nd_perf() {
    // cargo test -r our_small_index_perf -- --nocapture
    // cargo flamegraph --unit-test -- our_small_index_perf

    let duration = 5.0;
    let count = 1000;
    let mut rng = PCGRng::new(1234);

    eprintln!("Selecting the 42nd element from {count} elements.");
    eprintln!("Baseline is core::slice::select_nth_unstable.");
    bench(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), 42);
        },
        |mut data| {
            data.select_nth_unstable(42);
        },
        duration,
    );
}

#[test]
#[ignore]
fn small_median_perf() {
    // cargo test -r small_median_perf -- --nocapture --ignored
    // cargo flamegraph --unit-test -- small_median_perf --ignored

    let duration = 5.0;
    let count = 1000;
    let mid = count / 2;

    let mut rng = PCGRng::new(1234);
    eprintln!("Selecting the median from {count} elements.");
    eprintln!("Baseline is core::slice::select_nth_unstable.");
    bench(
        || random_u32s(count, rng.as_mut()),
        |mut data| {
            select_nth_unstable(data.as_mut_slice(), mid);
        },
        |mut data| {
            data.select_nth_unstable(mid);
        },
        duration,
    );
}
