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
    let count = 500;
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
