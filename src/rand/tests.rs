#[test]
fn get_rng() {
    use super::Rng;

    let mut rng = u32::rng(0);
    let (a, b) = (rng.next(), rng.next());
    assert!(a != b);
    type Arry = [f64; 8];
    let mut rng = Arry::rng(0);
    let arr = rng.next().unwrap();
    for elem in arr.into_iter() {
        assert!(elem >= 0.0);
        assert!(elem < 1.0);
    }
}

#[test]
fn bounded_u64() {
    use super::Rng;
    use std::collections::BTreeSet;

    let bound = 256;
    let mut rng = u64::rng(0).in_range(0, bound);
    let mut seen = BTreeSet::new();

    while seen.len() < 256 {
        let x = rng.get();
        assert!(x < bound);
        seen.insert(x);
    }
}
