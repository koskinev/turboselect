use crate::{
    ternary_block_partition_left, floyd_rivest_select, median_of_5, pcg_rng::PCGRng, prepare,
    quintary_left, quintary_right, read_pivots, select_nth_unstable, shuffle, sort_3, sort_4, ternary_block_partition_right,
};

use super::{partition_at_index_small, ternary};

fn iter_rng(rng: &mut PCGRng, count: usize, high: usize) -> impl Iterator<Item = usize> + '_ {
    std::iter::from_fn(move || Some(rng.bounded_usize(0, high))).take(count)
}

#[test]
fn partition_3() {
    let repeat = 1000;
    let count = 30;
    let k = count / 2;
    let mut rng = PCGRng::new(0);
    // let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let pivot = data[k];
        let (low, high) = ternary(&mut data, k);

        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < low => assert!(elem < &pivot),
                i if i > high => assert!(elem > &pivot),
                _ => assert!(elem == &pivot),
            }
        }
    }
}

#[test]
fn lomuto_2_left() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);
    // let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let (first, last) = (data[0], data[count - 1]);
        let (low, high) = (first.min(last), last.max(first));
        let (p, q) = ternary_block_partition_left(&mut data, 0, count - 1, |a, b| a < b);
        assert!(data[p] == low && data[q] == high);

        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < p => assert!(elem < &low),
                i if i > q => assert!(elem > &high),
                _ => assert!(&low <= elem && elem <= &high),
            }
        }
    }
}

#[test]
fn lomuto_2_right() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);
    // let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let (first, last) = (data[0], data[count - 1]);
        let (low, high) = (first.min(last), last.max(first));
        let (p, q) = ternary_block_partition_right(&mut data, 0, count - 1, |a, b| a < b);
        assert!(data[p] == low && data[q] == high);

        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < p => assert!(elem < &low),
                i if i > q => assert!(elem > &high),
                _ => assert!(&low <= elem && elem <= &high),
            }
        }
    }
}

#[test]
fn partition_5_left() {
    let repeat = 1000;
    let count = 30;
    let mut rng = PCGRng::new(123);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count / 3).collect();

        let (u_a, u_d, v_a, v_d) = prepare(&mut data, count / 3, &mut rng);
        let pivot_u = data[u_a];
        let pivot_v = data[v_a];

        eprintln!("Pivots are {pivot_u} and {pivot_v}");
        let (a, b, c, d) = quintary_left(&mut data, u_a, u_d, v_a, v_d);

        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < a => assert!(elem < &pivot_u),
                i if a <= i && i < b => assert!(elem == &pivot_u),
                i if b <= i && i <= c => assert!(elem > &pivot_u && elem < &pivot_v),
                i if c < i && i <= d => assert!(elem == &pivot_v),
                _ => assert!(elem > &pivot_v),
            }
        }
    }
}

#[test]
fn partition_5_right() {
    let repeat = 1000;
    let count = 100;
    let mut rng = PCGRng::new(123);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let (u_a, u_d, v_a, v_d) = prepare(&mut data, 2 * count / 3, &mut rng);
        let pivot_u = data[u_a];
        let pivot_v = data[v_a];

        let (a, b, c, d) = quintary_right(&mut data, u_a, u_d, v_a, v_d);

        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < a => assert!(elem < &pivot_u),
                i if a <= i && i < b => assert!(elem == &pivot_u),
                i if b <= i && i <= c => assert!(elem > &pivot_u && elem < &pivot_v),
                i if c < i && i <= d => assert!(elem == &pivot_v),
                _ => assert!(elem > &pivot_v),
            }
        }
    }
}

#[test]
fn partition_small() {
    let repeat = 100;
    let count = 20;
    let mut rng = PCGRng::new(123);

    for _iter in 0..repeat {
        let high = rng.bounded_usize(0, count);
        let mut data: Vec<_> = iter_rng(&mut rng, count, high).collect();
        let k = rng.bounded_usize(0, count);
        let (u, v) = partition_at_index_small(&mut data, k);
        assert!(u <= v && u <= k && v >= k && v < count);
        let uth = data[u];
        let vth = data[v];
        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < u => assert!(elem < &uth),
                i if i > v => assert!(elem > &vth),
                _ => (),
            }
        }
    }
}

#[test]
fn partition() {
    let repeat = 1000;
    let count = 10000;

    let mut rng = PCGRng::new(0);
    let mut k = 0;

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        floyd_rivest_select(&mut data, k, &mut rng);
        let kth = data[k];
        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < k => assert!(elem <= &kth),
                i if i > k => assert!(elem >= &kth),
                _ => (),
            }
        }
        k += count / repeat;
    }
}

#[test]
fn large_median() {
    let mut pcg = PCGRng::new(0);
    let count = 10_000_000;
    let mid = count / 2;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(data.as_mut_slice(), count, &mut pcg);
    let median = select_nth_unstable(data.as_mut_slice(), mid);
    assert_eq!(median, &mid);
}

#[test]
fn nth() {
    let repeat = 1000;
    let max = 10000;
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
fn sort3() {
    let repeat = 10000;
    let count = 3;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        sort_3(&mut data, 0, 1, 2);
        assert!(data[0] <= data[1]);
        assert!(data[1] <= data[2]);
    }
}

#[test]
fn sort4() {
    let repeat = 10000;
    let count = 4;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        sort_4(&mut data, 0, 1, 2, 3);
        assert!(data[0] <= data[1]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
    }
}

#[test]
fn median5() {
    let repeat = 10000;
    let count = 5;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        median_of_5(&mut data, 0, 1, 2, 3, 4);
        assert!(data[0] <= data[2]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
        assert!(data[2] <= data[4]);
    }
}

#[test]
fn pivots() {
    let repeat = 10000;
    let count = 10;
    let mut rng = PCGRng::new(123);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        let (low, inner, high) = read_pivots(&mut data, 1, count - 2);
        assert!(low.element() <= high.element());
        assert!(inner.len() == count - 2);
    }
}
