use super::{ternary_partion, Rng};

#[test]
fn ternary() {
    let repeat = 1000;
    let count = 100;
    let k = count / 2;
    let mut rng = usize::rng(0).in_range(0, count);

    for _iter in 0..repeat {
        let mut data: Vec<_> = rng.by_ref().take(count).collect();

        let pivot = data[k];
        let (lt, gt) = ternary_partion(&mut data, k);

        let lt_max = data[..lt].iter().max();
        let mid_min = data[lt..gt].iter().min();
        let mid_max = data[lt..gt].iter().max();
        let gt_min = data[gt..].iter().min();

        assert!(lt_max.map_or(true, |x| x < &pivot));
        assert!(mid_min.map_or(true, |x| x == &pivot));
        assert!(mid_max.map_or(true, |x| x == &pivot));
        assert!(gt_min.map_or(true, |x| x > &pivot));
    }
}
