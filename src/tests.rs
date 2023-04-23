use crate::{sort_3, sort_4, median_of_5, select_nth};

use super::{select_nth_small, ternary_partion, Rng};

#[test]
fn ternary() {
    let repeat = 1000;
    let count = 10;
    let k = count / 2;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();

        let pivot = data[k];
        let (low, high) = ternary_partion(&mut data, k);

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
fn nth_small() {
    let repeat = 1000;
    let count = 600;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();
        let k = rng.get();
        let kth = *select_nth_small(&mut data, k);
        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < k => assert!(elem <= &kth),
                i if i > k => assert!(elem >= &kth),
                _ => (),
            }
        }
    }
}

#[test]
fn nth() {
    let repeat = 1000;
    let count = 1000;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();
        let k = 42; // rng.get();
        let kth = *select_nth(&mut data, k);
        for (index, elem) in data.iter().enumerate() {
            match index {
                i if i < k => assert!(elem <= &kth),
                i if i > k => assert!(elem >= &kth),
                _ => (),
            }
        }
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
        assert!(data[0]<=data[1]);
        assert!(data[1]<=data[2]);
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
        assert!(data[0]<=data[1]);
        assert!(data[1]<=data[2]);
        assert!(data[2]<=data[3]);
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
        assert!(data[0]<=data[2]);
        assert!(data[1]<=data[2]);
        assert!(data[2]<=data[3]);
        assert!(data[2]<=data[4]);
    }
}