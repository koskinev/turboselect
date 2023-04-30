use crate::{
    median_of_5, partition_at_index, prepare, quintary_left,
    quintary_right,
    rand::{PCGRng, Rng},
    select_nth, sort_3, sort_4,
};

use super::{partition_at_index_small, ternary};

#[test]
fn partition_3() {
    let repeat = 1000;
    let count = 100;
    let k = count / 2;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();

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
fn partition_5_left() {
    let repeat = 1000;
    let count = 400;
    let mut pcg = PCGRng::new(123);
    let mut rng = usize::rng(456).in_range(0, count / 3);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();

        let (u_a, u_d, v_a, v_d) = prepare(&mut data, count / 3, &mut pcg);
        let pivot_u = data[u_a];
        let pivot_v = data[v_a];

        // eprintln!("Pivots are {pivot_u} and {pivot_v}");
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
    let mut pcg = PCGRng::new(123);
    let mut rng = usize::rng(123).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();

        let (u_a, u_d, v_a, v_d) = prepare(&mut data, 2 * count / 3, &mut pcg);
        let pivot_u = data[u_a];
        let pivot_v = data[v_a];

        // eprintln!("Pivots are {pivot_u} and {pivot_v}");
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
    let mut range_rng = usize::rng(0).in_range(2, count);

    for _iter in 0..repeat {
        let rng = usize::rng(0).in_range(0, range_rng.get());
        let mut data: Vec<_> = rng.take(count).collect();
        let k = range_rng.get();
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

    let mut pcg = PCGRng::new(0);
    let mut rng = usize::rng(0).in_range(0, count);
    let mut k = 0;

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();
        partition_at_index(&mut data, k, &mut pcg);
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
fn nth() {
    let repeat = 1000;
    let max = 10000;
    let mut pcg = PCGRng::new(0);

    for _iter in 0..repeat {
        let count = pcg.bounded_usize(1, max);
        let high = pcg.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| pcg.bounded_usize(0, high)).collect();
        let index = pcg.bounded_usize(0, count);
        select_nth(&mut data, index);
        let nth = data[index];
        data.iter().enumerate().for_each(|(i, elem)| {
            match i {
                i if i < index => assert!(elem <= &nth),
                i if i > index => assert!(elem >= &nth),
                _ => (),
            }
        });
    }
}

#[test]
fn sort3() {
    let repeat = 10000;
    let count = 3;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();
        sort_3(&mut data, 0, 1, 2);
        assert!(data[0] <= data[1]);
        assert!(data[1] <= data[2]);
    }
}

#[test]
fn sort4() {
    let repeat = 10000;
    let count = 4;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();
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
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();
        median_of_5(&mut data, 0, 1, 2, 3, 4);
        assert!(data[0] <= data[2]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
        assert!(data[2] <= data[4]);
    }
}
