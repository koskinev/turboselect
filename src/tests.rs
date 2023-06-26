use crate::{
    partition_in_blocks_dual, partition_max, partition_min, quickselect, sample,
    select_nth_unstable,
    sort::{median, sort},
    wyrand::WyRng,
};

fn iter_rng(rng: &mut WyRng, count: usize, high: usize) -> impl Iterator<Item = usize> + '_ {
    std::iter::from_fn(move || Some(rng.bounded_usize(0, high))).take(count)
}

fn shuffle<T>(data: &mut [T], rng: &mut WyRng) {
    let len = data.len();
    for i in 0..len - 1 {
        let j = rng.bounded_usize(i, len);
        data.swap(i, j);
    }
}

#[test]
fn block_dual() {
    #[cfg(not(miri))]
    let repeat = 1000;

    #[cfg(miri)]
    let repeat = 10;

    let max_count = 30;
    let mut rng = WyRng::new(123);

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
fn large_median() {
    let mut rng = WyRng::new(123);

    #[cfg(not(miri))]
    let count = 10_000_000;
    #[cfg(miri)]
    let count = 1000;

    let mid = count / 2;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(data.as_mut_slice(), &mut rng);
    let (left, median, right) = select_nth_unstable(data.as_mut_slice(), mid);
    assert_eq!(median, &mid);
    assert!(left.iter().all(|elem| elem < &mid));
    assert!(right.iter().all(|elem| elem > &mid));
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

    /// Returns a vector of `u32`s with a sawtooth pattern.
    fn sawtooth_u32s(count: usize, _rng: &mut WyRng) -> Vec<usize> {
        let mut data = Vec::with_capacity(count);
        let count = count;
        let sqrt_count = (count as f64).sqrt() as usize;
        for index in 0..count {
            let x = index % sqrt_count;
            data.push(x);
        }
        data
    }

    for iter in 0..repeat {
        let index = (iter * count) / repeat; 
        let mut data = sawtooth_u32s(count, rng.as_mut());
        let (left, nth, right) = select_nth_unstable(data.as_mut_slice(), index);
        assert!(left.iter().all(|elem| elem <= nth));
        assert!(right.iter().all(|elem| elem >= nth));
    }
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

    /// Returns a vector of integers in the range `0..count`, in reversed order.
    fn reversed_u32s(count: usize, rng: &mut WyRng) -> Vec<u32> {
        let mut data = Vec::with_capacity(count);
        let max = rng.bounded_u32(0, count as u32);
        for index in 0..count {
            data.push((max * (count - index + 1) as u32) / (count as u32));
        }
        data
    }
    
    for iter in 0..repeat {
        let index = (iter * count) / repeat; 
        let mut data = reversed_u32s(count, rng.as_mut());
        let (left, nth, right) = select_nth_unstable(data.as_mut_slice(), index);
        assert!(left.iter().all(|elem| elem <= nth));
        assert!(right.iter().all(|elem| elem >= nth));
    }
}
#[test]
fn bool_median() {
    #[cfg(not(miri))]
    let repeat = 1000;

    #[cfg(miri)]
    let repeat = 10;

    #[cfg(not(miri))]
    let count = 1000;
    #[cfg(miri)]
    let count = 100;

    let index = count / 2;
    let mut rng = WyRng::new(123);

    for _iter in 0..repeat {
        let mut data: Vec<_> = (0..count).map(|_| rng.bool()).collect();
        // let mut cloned = data.clone();
        // let (left, nth, right) = cloned.select_nth_unstable(index);
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
    shuffle(data.as_mut_slice(), &mut rng);
    let (left, &mut nth, right) = select_nth_unstable(data.as_mut_slice(), index);

    assert!(left.iter().all(|elem| elem < &nth));
    assert!(right.iter().all(|elem| elem > &nth));
    assert_eq!(nth, data[index]);

    let index = count - 42;
    let (left, &mut nth, right) = select_nth_unstable(data.as_mut_slice(), index);

    assert!(left.iter().all(|elem| elem < &nth));
    assert!(right.iter().all(|elem| elem > &nth));
    assert_eq!(nth, data[index]);
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

    let mut rng = WyRng::new(123);

    for _iter in 0..repeat {
        let count = rng.bounded_usize(1, max);
        let high = rng.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| rng.bounded_usize(0, high)).collect();
        let index = rng.bounded_usize(0, count);
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
    let mut rng = WyRng::new(123);

    for _iter in 0..repeat {
        let count = rng.bounded_usize(1, max);
        let high = rng.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| rng.bounded_usize(0, high)).collect();
        let index = rng.bounded_usize(0, count);
        quickselect(&mut data, index, &mut usize::lt, &mut rng);
        let nth = data[index];
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
        let (_u, v) = partition_min(data.as_mut_slice(), &mut usize::lt);
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
    let (u, v) = partition_min(data.as_mut_slice(), &mut usize::lt);
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
        let (u, _v) = partition_max(data.as_mut_slice(), &mut usize::lt);
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
    let (u, v) = partition_max(data.as_mut_slice(), &mut usize::lt);
    assert_eq!(u, 0);
    assert_eq!(v, 9);
}

#[test]
fn sort2() {
    let mut data = [1, 0];
    sort(data.as_mut_slice(), [0, 1], &mut i32::lt);
    assert_eq!(data, [0, 1]);
}

#[test]
fn sort3() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let count = 3;
    let mut rng = WyRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        sort(&mut data, [0, 1, 2], &mut usize::lt);
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
    let mut rng = WyRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        sort(&mut data, [0, 1, 2, 3], &mut usize::lt);
        assert!(data[0] <= data[1]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
    }
}

#[test]
fn sort9() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let mut rng = WyRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, 9, 9).collect();
        sort(
            &mut data,
            core::array::from_fn::<_, 9, _>(|i| i),
            &mut usize::lt,
        );
        for i in 1..9 {
            assert!(data[i - 1] <= data[i]);
        }
    }
}

#[test]
fn sort21() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let mut rng = WyRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, 21, 21).collect();
        sort(
            &mut data,
            core::array::from_fn::<_, 21, _>(|i| i),
            &mut usize::lt,
        );
        for i in 1..21 {
            assert!(data[i - 1] <= data[i]);
        }
    }
}

#[test]
fn median5() {
    #[cfg(not(miri))]
    let repeat = 1000;
    #[cfg(miri)]
    let repeat = 1;

    let count = 5;
    let mut rng = WyRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        median(&mut data, [0, 1, 2, 3, 4], &mut usize::lt);
        assert!(data[0] <= data[2]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
        assert!(data[2] <= data[4]);
    }
}
