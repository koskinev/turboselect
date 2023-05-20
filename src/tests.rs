use crate::{
    floyd_rivest, median_5, partition_2_single_index, partition_3_dual_index_high,
    partition_3_dual_index_low, partition_3_dual_pivot_high, partition_3_dual_pivot_low,
    partition_3_single_index, pcg_rng::PCGRng, sample, select_min, select_nth_small,
    select_nth_unstable, sort_3, sort_4,
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

#[test]
fn hoare_2_single_index() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);

    for iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let p = iter % count;
        let pivot = data[p];

        let u = partition_2_single_index(&mut data, p, usize::lt);

        assert!(data[..u].iter().all(|elem| elem < &pivot));
        assert!(data[u..].iter().all(|elem| elem >= &pivot));
    }
}

#[test]
fn lomuto_3_single_index() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);

    for iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let p = iter % count;
        let pivot = data[p];

        let (u, v) = partition_3_single_index(&mut data, p, usize::lt);

        assert!(data[..u].iter().all(|elem| elem < &pivot));
        assert!(data[u..=v].iter().all(|elem| elem == &pivot));
        assert!(data[v + 1..].iter().all(|elem| elem > &pivot));
    }
}

#[test]
fn lomuto_3_dual_index_low() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let p = rng.bounded_usize(0, count - 1);
        let q = rng.bounded_usize(p + 1, count);

        let low = data[p].min(data[q]);
        let high = data[p].max(data[q]);

        let (u, v) = partition_3_dual_index_low(&mut data, p, q, usize::lt);
        assert!(data[..u].iter().all(|elem| elem < &low));
        assert!(data[u..=v].iter().all(|elem| elem >= &low));
        assert!(data[u..=v].iter().all(|elem| elem <= &high));
        assert!(data[v + 1..].iter().all(|elem| elem > &high));
    }
}

#[test]
fn lomuto_3_dual_index_high() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let p = rng.bounded_usize(0, count - 1);
        let q = rng.bounded_usize(p + 1, count);

        let low = data[p].min(data[q]);
        let high = data[p].max(data[q]);

        let (u, v) = partition_3_dual_index_high(&mut data, p, q, usize::lt);
        assert!(data[..u].iter().all(|elem| elem < &low));
        assert!(data[u..=v].iter().all(|elem| elem >= &low));
        assert!(data[u..=v].iter().all(|elem| elem <= &high));
        assert!(data[v + 1..].iter().all(|elem| elem > &high));
    }
}

#[test]
fn lomuto_3_dual_low() {
    let repeat = 1000;
    let count = 30;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let mut low = rng.bounded_usize(0, count);
        let mut high = rng.bounded_usize(low, count);

        let (u, v) = partition_3_dual_pivot_low(&mut data, &mut low, &mut high, usize::lt);
        assert!(data[..u].iter().all(|elem| elem < &low));
        assert!(data[u..v].iter().all(|elem| elem >= &low));
        assert!(data[u..v].iter().all(|elem| elem <= &high));
        assert!(data[v..].iter().all(|elem| elem > &high));
    }
}

#[test]
fn lomuto_3_dual_high() {
    let repeat = 1000;
    let count = 300;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();

        let mut low = rng.bounded_usize(0, count);
        let mut high = rng.bounded_usize(low, count);

        let (u, v) = partition_3_dual_pivot_high(&mut data, &mut low, &mut high, usize::lt);
        assert!(data[..u].iter().all(|elem| elem < &low));
        assert!(data[u..v].iter().all(|elem| elem >= &low));
        assert!(data[u..v].iter().all(|elem| elem <= &high));
        assert!(data[v..].iter().all(|elem| elem > &high));
    }
}

#[test]
fn floyd_rivest_300() {
    let repeat = 1000;
    let count = 300;
    let mut k = 0;
    let mut rng = PCGRng::new(0);

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(&mut rng, count, count).collect();
        let (u, v) = floyd_rivest(&mut data, k, usize::lt, &mut rng);
        assert!(u <= k && v >= k && v < count);
        let kth = data[k];
        assert_eq!(data[u], kth);
        assert_eq!(data[v], kth);
        // assert!(data[..u].iter().all(|elem| elem < &kth));
        assert!(data[..k].iter().all(|elem| elem <= &kth));
        assert!(data[k..].iter().all(|elem| elem >= &kth));
        // if v < count - 1 {
        //     assert!(data[v + 1..].iter().all(|elem| elem > &kth));
        // }
        k = (k + 1) % count;
    }
}

#[test]
fn large_median() {
    let mut pcg = PCGRng::new(123);
    let count = 10_000_000;
    let mid = count / 2;

    let mut data: Vec<usize> = (0..count).collect();
    shuffle(data.as_mut_slice(), &mut pcg);
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
fn nth_small() {
    let repeat = 1000;
    let max = 1000;
    let mut pcg = PCGRng::new(123);
    let is_less = |a: &usize, b: &usize| a < b;

    for _iter in 0..repeat {
        let count = pcg.bounded_usize(1, max);
        let high = pcg.bounded_usize(0, count);

        let mut data: Vec<_> = (0..count).map(|_| pcg.bounded_usize(0, high)).collect();
        let index = pcg.bounded_usize(0, count);
        let (u, v) = select_nth_small(&mut data, index, is_less, &mut pcg);
        let nth = data[index];
        assert_eq!(data[u], nth);
        assert_eq!(data[v], nth);
        assert!(data[..index].iter().all(|elem| elem <= &nth));
        assert!(data[index..].iter().all(|elem| elem >= &nth));
    }
}

#[test]
fn sample_10() {
    let len = 20;
    let count = 10;
    let mut rng = PCGRng::new(0);
    let mut data: Vec<_> = (0..len).collect();

    sample(data.as_mut_slice(), count, rng.as_mut());
    for i in 0..len {
        assert!(data.contains(&i));
    }
}

#[test]
fn min_10() {
    let len = 10;
    let mut rng = PCGRng::new(0);
    let repeat = 1000;

    for _iter in 0..repeat {
        let mut data: Vec<_> = iter_rng(rng.as_mut(), len, len / 2).collect();
        shuffle(data.as_mut_slice(), rng.as_mut());

        let (_u, v) = select_min(data.as_mut_slice(), usize::lt);
        let min = &data[0];
        assert!(data[..=v].iter().all(|elem| elem == min));
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
        median_5(&mut data, 0, 1, 2, 3, 4);
        assert!(data[0] <= data[2]);
        assert!(data[1] <= data[2]);
        assert!(data[2] <= data[3]);
        assert!(data[2] <= data[4]);
    }
}
