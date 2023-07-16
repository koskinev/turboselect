#[cfg(feature = "std")]
extern crate std;

use std::{io::Write, println, vec::Vec};

use crate::{
    choose_pivot, isqrt, partition_at, partition_equal_max, partition_equal_min, sample,
    select_nth_unstable, sort::sort_at, turboselect, wyrand::WyRng,
};

#[test]
fn bool_median() {
    #[cfg(not(miri))]
    let repeat = 1000;

    #[cfg(miri)]
    let repeat = 10;

    #[cfg(not(miri))]
    let count = 10000;
    #[cfg(miri)]
    let count = 100;

    let index = count / 2;
    let mut rng = WyRng::new(123);

    for _iter in 0..repeat {
        let mut data: Vec<_> = (0..count).map(|_| rng.bool()).collect();
        let (left, nth, right) = select_nth_unstable(data.as_mut_slice(), index);
        assert!(left.iter().all(|elem| elem <= nth));
        assert!(right.iter().all(|elem| elem >= nth));
    }
}

#[test]
fn extreme_index() {
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000_000;
    #[cfg(miri)]
    let count = 1000;

    let index = 42;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(&mut data, &mut rng);
    let (left, &mut nth, right) = select_nth_unstable(&mut data, index);

    assert!(left.iter().all(|elem| elem < &nth));
    assert!(right.iter().all(|elem| elem > &nth));
    assert_eq!(nth, data[index]);

    let index = count - 42;
    let (left, &mut nth, right) = select_nth_unstable(&mut data, index);

    assert!(left.iter().all(|elem| elem < &nth));
    assert!(right.iter().all(|elem| elem > &nth));
    assert_eq!(nth, data[index]);
}

fn iter_rng(rng: &mut WyRng, count: usize, high: usize) -> impl Iterator<Item = usize> + '_ {
    core::iter::from_fn(move || Some(rng.bounded_usize(0, high))).take(count)
}

#[test]
fn large_median() {
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000_000;
    #[cfg(miri)]
    let count = 1000;

    let mid = count / 2;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(&mut data, &mut rng);
    let (left, median, right) = select_nth_unstable(&mut data, mid);
    assert_eq!(median, &mid);
    assert!(left.iter().all(|elem| elem < &mid));
    assert!(right.iter().all(|elem| elem > &mid));
}

#[test]
fn min_10() {
    let len = 10;
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 10;

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(rng.as_mut(), len, len / 2).collect();
        let mut cloned = data.clone();
        let (_u, v) = partition_equal_min(&mut data, 0, &mut usize::lt);
        let min = &data[0];
        for (i, elem) in data.iter().enumerate() {
            if i <= v {
                assert!(elem == min);
            } else {
                assert!(elem > min);
            }
        }
        data.sort();
        cloned.sort();
        assert_eq!(data, cloned);
    }

    let mut data: Vec<_> = core::iter::repeat(1).take(10).collect();
    let (u, v) = partition_equal_min(&mut data, 0, &mut usize::lt);
    assert_eq!(u, 0);
    assert_eq!(v, 9);
}
#[test]
fn max_10() {
    let len = 10;
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let repeat = 10000;
    #[cfg(miri)]
    let repeat = 10;

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(rng.as_mut(), len, len / 2).collect();
        let mut cloned = data.clone();
        let (u, _v) = partition_equal_max(&mut data, 0, &mut usize::lt);
        let max = &data[u];
        for (i, elem) in data.iter().enumerate() {
            if i >= u {
                assert!(elem == max);
            } else {
                assert!(elem < max);
            }
        }
        data.sort();
        cloned.sort();
        assert_eq!(data, cloned);
    }

    let mut data: Vec<_> = core::iter::repeat(1).take(10).collect();
    let (u, v) = partition_equal_max(data.as_mut_slice(), 0, &mut usize::lt);
    assert_eq!(u, 0);
    assert_eq!(v, 9);
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

    let mut rng = WyRng::new(1234);

    for _iter in 0..repeat {
        let count = rng.bounded_usize(1, max);
        let high = rng.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| rng.bounded_usize(0, high)).collect();
        let index = rng.bounded_usize(0, count);
        turboselect(&mut data, index, rng.as_mut(), &mut usize::lt);
        let nth = &data[index];
        data.iter().enumerate().for_each(|(i, elem)| match i {
            i if i < index && elem > nth => panic!("{} > {} at {}", elem, nth, i),
            i if i > index && nth > elem => panic!("{} > {} at {}", nth, elem, i),
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
    let mut rng = WyRng::new(123);

    for _iter in 0..repeat {
        let count = rng.bounded_usize(1, max);
        let high = rng.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| rng.bounded_usize(0, high)).collect();
        let index = rng.bounded_usize(0, count);
        turboselect(&mut data, index, &mut rng, &mut usize::lt);
        let nth = data[index];
        assert!(data[..index].iter().all(|elem| elem <= &nth));
        assert!(data[index..].iter().all(|elem| elem >= &nth));
    }
}

#[test]
#[ignore]
fn pivots() {
    let mut rng = WyRng::new(123);
    let max = 10_000;
    let repeat = 10_000;

    let mut output = std::fs::File::create("misc/pivots.csv").unwrap();
    let mut results = Vec::new();
    let mut total_cost = 0.;
    writeln!(results, "index,partition_at,len,i,p,cost").unwrap();

    for _iter in 0..repeat {
        let count = rng.bounded_usize(100, max);
        // let mut data: Vec<_> = (0..count).collect();
        // shuffle(&mut data, rng.as_mut());

        let mut data = reversed_usizes(count, &mut rng);

        let index = rng.bounded_usize(0, count);

        let (p, _) = choose_pivot(&mut data, index, rng.as_mut(), &mut usize::lt);
        let (u, v) = partition_at(&mut data, p, &mut usize::lt);
        let cost = if index < u {
            u
        } else if index <= v {
            0
        } else {
            count - v
        } as f64
            / count as f64;
        total_cost += cost;
        let partition_at = (u + v) / 2;
        writeln!(
            results,
            "{index},{partition_at},{count},{i},{p},{cost}",
            i = (index as f64) / (count as f64),
            p = (partition_at as f64) / (count as f64)
        )
        .unwrap();
    }
    let ratio = total_cost / repeat as f64;
    println!("average cost: {ratio:.3}",);
    output.write_all(&results).unwrap(); // 0.390
}

#[test]
fn reversed() {
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000;
    #[cfg(miri)]
    let count = 1000;

    #[cfg(not(miri))]
    let repeat = 1000;

    #[cfg(miri)]
    let repeat = 10;

    for iter in 0..repeat {
        let index = (iter * count) / repeat;
        let mut data = reversed_usizes(count, rng.as_mut());
        let (left, nth, right) = select_nth_unstable(data.as_mut_slice(), index);
        left.iter().enumerate().for_each(|(i, elem)| match i {
            i if elem > nth => panic!("iter {iter}: left[{i}] = {elem} > nth = {nth}"),
            _ => (),
        });
        right.iter().enumerate().for_each(|(i, elem)| match i {
            i if elem < nth => panic!("iter {iter}: left[{i}] = {elem} < nth = {nth}"),
            _ => (),
        });
    }
}

#[test]
fn sample_n() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let len = 20;
    let mut rng = WyRng::new(0);

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
fn sawtooth() {
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000;
    #[cfg(miri)]
    let count = 1000;

    #[cfg(not(miri))]
    let repeat = 1000;

    #[cfg(miri)]
    let repeat = 10;

    for iter in 0..repeat {
        let index = (iter * count) / repeat;
        let mut data = sawtooth_usizes(count, rng.as_mut());
        let (left, nth, right) = select_nth_unstable(data.as_mut_slice(), index);
        assert!(left.iter().all(|elem| elem <= nth));
        assert!(right.iter().all(|elem| elem >= nth));
    }
}

fn shuffle<T>(data: &mut [T], rng: &mut WyRng) {
    let len = data.len();
    for i in 0..len - 1 {
        let j = rng.bounded_usize(i, len);
        data.swap(i, j);
    }
}

#[test]
fn sorts() {
    fn sort_indexed<const N: usize>() {
        #[cfg(not(miri))]
        let repeat = 1000;
        #[cfg(miri)]
        let repeat = 1;

        let mut rng = WyRng::new(0);
        let pos: [usize; N] = core::array::from_fn(|i| i);
        for _iter in 0..repeat {
            let mut data: Vec<_> = iter_rng(&mut rng, N, N).collect();
            sort_at(&mut data, pos, &mut usize::lt);
            for i in 1..N {
                assert!(data[i - 1] <= data[i]);
            }
        }
    }
    sort_indexed::<3>();
    sort_indexed::<5>();
    sort_indexed::<7>();
}

#[test]
#[cfg(feature = "std")]
fn sqrts() {
    for x in 0..1000 {
        assert_eq!(isqrt(x), (x as f64).sqrt().floor() as usize);
    }

    let mut rng = WyRng::new(123);
    for _ in 0..10000 {
        let x = rng.usize();
        assert_eq!(isqrt(x), (x as f64).sqrt().floor() as usize);
    }
}

/// Returns a vector of integers in the range `0..count`, in reversed order.
fn reversed_usizes(count: usize, rng: &mut WyRng) -> Vec<usize> {
    let mut data = Vec::with_capacity(count);
    let max = rng.bounded_usize(0, count);
    for index in 0..count {
        data.push((max * (count - index + 1)) / count);
    }
    data
}

/// Returns a vector of `u32`s with a sawtooth pattern.
fn sawtooth_usizes(count: usize, rng: &mut WyRng) -> Vec<usize> {
    let mut data = Vec::with_capacity(count);
    let max_lenght = (count as f64).sqrt() as usize;
    let length = rng.bounded_usize(max_lenght / 4 + 1, max_lenght);
    for index in 0..count {
        let x = index % length;
        data.push(x);
    }
    data
}
