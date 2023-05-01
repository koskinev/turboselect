#![feature(specialization)]

mod dbg;
mod pcg_rng;
use std::cmp::Ordering;

use dbg::Dbg;
use pcg_rng::PCGRng;

#[cfg(test)]
mod tests;

const ALPHA: f64 = 0.5;
const BETA: f64 = 0.25;
const CUT: usize = 1000;

fn floyd_rivest_select<T: Ord>(
    mut data: &mut [T],
    mut index: usize,
    rng: &mut PCGRng,
) -> (usize, usize) {
    let mut offset = 0;
    let (a, d) = loop {
        if index == 0 {
            break partition_at_first(data);
        } else if index == data.len() - 1 {
            break partition_at_last(data);
        } else if data.len() < CUT {
            break partition_at_index_small(data, index);
        } else {
            let (u_a, u_d, v_a, v_d) = prepare(data, index, rng);
            let (a, b, c, d) = if index <= data.len() / 2 {
                quintary_left(data, u_a, u_d, v_a, v_d)
            } else {
                quintary_right(data, u_a, u_d, v_a, v_d)
            };
            if index < a {
                data = &mut data[..a];
            } else if index < b {
                break (a, b - 1);
            } else if index <= c {
                data = &mut data[b..=c];
                offset += b;
                index -= b;
            } else if index <= d {
                break (c + 1, d);
            } else {
                data = &mut data[d + 1..];
                offset += d + 1;
                index -= d + 1;
            }
        }
    };
    (a + offset, d + offset)
}

fn median_of_5<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    swap(data, a, c);
    swap(data, b, d);
    if swap(data, c, d) {
        data.swap(a, b);
    }
    swap(data, b, e);
    if swap(data, c, e) {
        swap(data, a, c);
    } else {
        swap(data, b, c);
    }
    c
}

fn partition_at_first<T: Ord>(data: &mut [T]) -> (usize, usize) {
    let mut d = 0;
    for i in 1..data.len() {
        match data[i].cmp(&data[0]) {
            Ordering::Greater => {}
            Ordering::Less => {
                d = 0;
                data.swap(0, i);
            }
            Ordering::Equal => {
                d += 1;
                data.swap(i, d);
            }
        }
    }
    (0, d)
}

fn partition_at_last<T: Ord>(data: &mut [T]) -> (usize, usize) {
    let r = data.len() - 1;
    let mut a = r;
    for i in (0..r).rev() {
        match data[i].cmp(&data[r]) {
            Ordering::Greater => {
                a = r;
                data.swap(i, r);
            }
            Ordering::Less => {}
            Ordering::Equal => {
                a -= 1;
                data.swap(i, a);
            }
        }
    }
    (a, r)
}

/// Finds the `k`th smallest element in `data`. Returns the `(a, d)` where `a <= k <= d`.
/// After the call, `data` is partitioned into three parts:
/// - Elements in the range `0..a` are less than the `k`th smallest element
/// - Elements in the range `a..=d` are equal to the `k`th smallest element
/// - Elements in the range `d+1..` are greater than the `k`th smallest element
///
/// # Panics
///
/// Panics if `k >= data.len()`.
fn partition_at_index_small<T: Ord>(data: &mut [T], index: usize) -> (usize, usize) {
    assert!(index < data.len());
    match data.len() {
        len @ 5.. => match index {
            0 => partition_at_first(data),
            k if k == len - 1 => partition_at_last(data),
            _ => {
                let c = len / 2;
                let b = c / 2;
                let d = c + b;
                median_of_5(data, 0, b, c, d, len - 1);
                let (a, d) = ternary(data, c);
                match index {
                    k if k < a => partition_at_index_small(&mut data[..a], k),
                    k if k > d => {
                        let (u, v) = partition_at_index_small(&mut data[d + 1..], k - d - 1);
                        (d + 1 + u, d + 1 + v)
                    }
                    _ => (a, d),
                }
            }
        },
        4 => {
            sort_4(data, 0, 1, 2, 3);
            let (mut a, mut d) = (0, 3);
            while data[a] != data[index] {
                a += 1;
            }
            while data[d] != data[index] {
                d -= 1;
            }
            (a, d)
        }
        3 => {
            sort_3(data, 0, 1, 2);
            let (mut a, mut d) = (0, 2);
            while data[a] != data[index] {
                a += 1;
            }
            while data[d] != data[index] {
                d -= 1;
            }
            (a, d)
        }
        2 => {
            sort_2(data, 0, 1);
            if data[0] == data[1] {
                (0, 1)
            } else {
                (index, index)
            }
        }
        1 => (index, index),
        _ => panic!("empty slice"),
    }
}

fn pivot_gap(s: usize, n: usize) -> usize {
    let n = n as f64;
    (BETA * (s as f64) * n.ln()).powf(0.5) as usize
}

fn prepare<T: Ord>(data: &mut [T], index: usize, rng: &mut PCGRng) -> (usize, usize, usize, usize) {
    let len = data.len();
    let s = sample_size(len);
    shuffle(data, s, rng);

    let g = pivot_gap(s, len);
    let u = (((index + 1) * s) / len).saturating_sub(g);
    let v = (((index + 1) * s) / len + g).min(s - 1);

    let (v_a, v_d) = floyd_rivest_select(&mut data[..s], v, rng);
    if u < v_a {
        let (u_a, u_d) = floyd_rivest_select(&mut data[..v_a], u, rng);
        let q = len - s + v_a;

        swap_parts(data, v_a, len - 1, s - v_a);
        (u_a, u_d, q, q + v_d - v_a)
    } else {
        (v_a, v_d, v_a, v_d)
    }
}

/// Partitions `data` into five parts, using the `u`th and `v`th elements as the pivots. Returns
/// `(a, b, c, d)` where `0 <= a <= b < c <= d < data.len()`.
///
/// After the partitioning, the slice is arranged as follows:
/// ```text
///  ┌───────────────────────┐
///  │ x < data[a]           │ x == data[i] where i in ..a
///  ├───────────────────────┤
///  │ x == data[a]          │ i in a..b
///  ├───────────────────────┤
///  │ data[a] < x < data[d] │ i in b..=c
///  ├───────────────────────┤
///  │ x == data[d]          │ i in c+1..=d
///  ├───────────────────────┤
///  │ x > data[d]           │ i in d+1..
///  └───────────────────────┘
/// ```
///
/// # Panics
///
/// Panics if `u` or `v` is out of bounds.
fn quintary_left<T: Ord>(
    data: &mut [T],
    u_a: usize,
    u_d: usize,
    v_a: usize,
    v_d: usize,
) -> (usize, usize, usize, usize) {
    if data[u_a] == data[v_a] {
        let (a, d) = ternary(data, u_d);
        return (a, d + 1, d, d);
    }
    // See https://github.com/rust-lang/rust/blob/master/library/core/src/slice/sort.rs#L302
    // for optimizating the partitioning.
    let s = u_a;
    let e = v_d;
    let mut l = u_d + 1;
    let mut p = l;
    let mut q = v_a - 1;
    let mut i = p - 1;
    let mut j = q + 1;
    loop {
        // Increment i until data[i] >= data[r]
        loop {
            i += 1;
            if data[i] >= data[e] {
                break;
            }
            match data[i].cmp(&data[s]) {
                Ordering::Less => continue,
                Ordering::Greater => data.swap(p, i),
                Ordering::Equal => {
                    data.swap(p, i);
                    data.swap(l, p);
                    l += 1;
                }
            }
            p += 1;
        }
        // Decrement j until data[j] < data[r]
        loop {
            j -= 1;
            match data[j].cmp(&data[e]) {
                Ordering::Greater => continue,
                Ordering::Less => break,
                Ordering::Equal => {
                    data.swap(j, q);
                    q -= 1;
                }
            }
        }
        // Exchange data[i] and data[j], then if i < j and repeat,
        // otherwise break the loop.
        if i < j {
            data.swap(i, j);
            match data[i].cmp(&data[s]) {
                Ordering::Greater => {
                    data.swap(p, i);
                    p += 1;
                }
                Ordering::Equal => {
                    data.swap(i, p);
                    data.swap(l, p);
                    l += 1;
                    p += 1;
                }
                _ => {}
            }
            if data[j] == data[e] {
                data.swap(j, q);
                q -= 1;
            }
        } else {
            break;
        }
    }

    // Cleanup. At this point, the following assertions hold:

    // let low = &data[s];
    // let high = &data[e];
    // for (k, x) in data.iter().enumerate() {
    //     if k < s {
    //         // Part 0: ..s
    //         assert!(x < low);
    //     } else if k < l {
    //         // Part 1: s..l
    //         assert!(x == low);
    //     } else if k < p {
    //         // Part 2: l..p
    //         assert!(low < x && x < high);
    //     } else if k <= j {
    //         // Part 3: p..=j
    //         assert!(x < low);
    //     } else if k <= q {
    //         // Part 4: j+1..=q
    //         assert!(x > high);
    //     } else if k <= e {
    //         // Part 5: q+1..=e
    //         assert!(x == high);
    //     } else {
    //         // Part 6: e+1..
    //         assert!(x > high);
    //     }
    // }

    let a = s + i - p;
    let b = a + l - s;
    let d = e + j - q;
    let c = d + q - e;

    // Swap parts 2 and 3.
    swap_parts(data, l, j, (j + 1 - p).min(p - l));

    // Swap parts 1 and 2.
    swap_parts(data, s, b - 1, (l - s).min(j + 1 - p));

    // Swap parts 4 and 5.
    swap_parts(data, i, e, (q + 1 - i).min(e - q));

    // The slice is now partitioned as follows:

    // let low = &data[a];
    // let high = &data[d];
    // for (k, x) in data.iter().enumerate() {
    //     if k < a {
    //         assert!(x < low);
    //     } else if k < b {
    //         assert!(x == low);
    //     } else if k <= c {
    //         assert!(low < x && x < high);
    //     } else if k <= d {
    //         assert!(x == high);
    //     } else {
    //         assert!(x > high);
    //     }
    // }

    (a, b, c, d)
}

/// Partitions `data` into five parts, using the `u`th and `v`th elements as the pivots. Returns
/// `(a, b, c, d)` where `0 <= a <= b < c <= d < data.len()`.
///
/// After the partitioning, the slice is arranged as follows:
/// ```text
///  ┌───────────────────────┐
///  │ x < data[a]           │ x == data[i] where i in ..a
///  ├───────────────────────┤
///  │ x == data[a]          │ i in a..b
///  ├───────────────────────┤
///  │ data[a] < x < data[d] │ i in b..=c
///  ├───────────────────────┤
///  │ x == data[d]          │ i in c+1..=d
///  ├───────────────────────┤
///  │ x > data[d]           │ i in d+1..
///  └───────────────────────┘
/// ```
///
/// # Panics
///
/// Panics if `u` or `v` is out of bounds.
fn quintary_right<T: Ord>(
    data: &mut [T],
    u_a: usize,
    u_d: usize,
    v_a: usize,
    v_d: usize,
) -> (usize, usize, usize, usize) {
    if data[u_a] == data[v_a] {
        let (a, d) = ternary(data, u_d);
        return (a, d + 1, d, d);
    }
    let s = u_a;
    let e = v_d;
    let mut p = u_d + 1;
    let mut q = v_a - 1;
    let mut h = q;
    let mut i = p - 1;
    let mut j = q + 1;
    loop {
        // Increment i until data[i] > data[s]
        loop {
            i += 1;
            match data[i].cmp(&data[s]) {
                Ordering::Less => continue,
                Ordering::Greater => break,
                Ordering::Equal => {
                    data.swap(p, i);
                    p += 1;
                }
            }
        }
        // Decrement j until data[j] <= data[s]
        loop {
            j -= 1;
            if data[j] <= data[s] {
                break;
            }
            match data[j].cmp(&data[e]) {
                Ordering::Greater => continue,
                Ordering::Less => {
                    data.swap(j, q);
                }
                Ordering::Equal => {
                    data.swap(j, q);
                    data.swap(q, h);
                    h -= 1;
                }
            }
            q -= 1;
        }
        // Exchange data[i] and data[j], then if i < j repeat,
        // otherwise break the loop
        if i < j {
            data.swap(i, j);
            if data[i] == data[s] {
                data.swap(i, p);
                p += 1;
            }
            match data[j].cmp(&data[e]) {
                Ordering::Less => {
                    data.swap(j, q);
                    q -= 1;
                }
                Ordering::Equal => {
                    data.swap(j, q);
                    data.swap(h, q);
                    h -= 1;
                    q -= 1;
                }
                _ => {}
            }
        } else {
            break;
        }
    }

    // Cleanup. At this point, the following assertions hold:

    // let low = &data[s];
    // let high = &data[e];
    // for (k, x) in data.iter().enumerate() {
    //     if k < s {
    //         // Part 0: ..s
    //         assert!(x < low);
    //     } else if k < p {
    //         // Part 1: s..p
    //         assert!(x == low);
    //     } else if k < i {
    //         // Part 2: p..i
    //         assert!(x < low);
    //     } else if k <= q {
    //         // Part 3: i..=q
    //         assert!(x > high);
    //     } else if k <= h {
    //         // Part 4: q+1..=h
    //         assert!(low < x && x < high);
    //     } else if k <= e {
    //         // Part 5: h+1..=e
    //         assert!(x == high);
    //     } else {
    //         // Part 6: e+1..
    //         assert!(x > high);
    //     }
    // }

    let a = s + i - p;
    let b = a + p - s;
    let d = e + j - q;
    let c = d + h - e;

    // Swap parts 3 and 4
    swap_parts(data, i, h, (q + 1 - i).min(h - q));

    // Swap parts 4 and 5
    swap_parts(data, c + 1, e, (e - h).min(q + 1 - i));

    // Swap parts 1 and 2
    swap_parts(data, s, b - 1, (p - s).min(i - p));

    // The slice is now partitioned as follows:

    // let low = &data[a];
    // let high = &data[d];
    // for (k, x) in data.iter().enumerate() {
    //     if k < a {
    //         assert!(x < low);
    //     } else if k < b {
    //         assert!(x == low);
    //     } else if k <= c {
    //         assert!(low < x && x < high);
    //     } else if k <= d {
    //         assert!(x == high);
    //     } else {
    //         assert!(x > high);
    //     }
    // }

    (a, b, c, d)
}

fn sample_size(n: usize) -> usize {
    let n = n as f64;
    let f = n.powf(2. / 3.) * n.ln().powf(1. / 3.);
    (ALPHA * f).ceil().min(n - 1.) as usize
}

pub fn select_nth_unstable<T: Ord>(data: &mut [T], index: usize) -> &T {
    if data.len() < CUT {
        partition_at_index_small(data, index);
    } else {
        let mut rng = PCGRng::new(data.as_ptr() as u64);
        floyd_rivest_select(data, index, &mut rng);
    }
    &data[index]
}

/// Swaps elements in the range `..count`, with a random element in the range `index..count`,
/// where `index` is the index of the element.
fn shuffle<T>(data: &mut [T], count: usize, rng: &mut PCGRng) {
    let len = data.len();
    for i in 0..count {
        let j = rng.bounded_usize(i, len);
        data.swap(i, j);
    }
}

fn sort_2<T: Ord>(data: &mut [T], a: usize, b: usize) -> usize {
    swap(data, a, b);
    0
}

fn sort_3<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize) -> usize {
    swap(data, a, b);
    if swap(data, b, c) {
        swap(data, a, b);
    }
    1
}

fn sort_4<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize, d: usize) -> usize {
    swap(data, a, b);
    swap(data, c, d);
    if swap(data, b, c) {
        swap(data, a, b);
    }
    if swap(data, c, d) {
        swap(data, b, c);
    }
    1
}

fn swap<T: Ord>(data: &mut [T], a: usize, b: usize) -> bool {
    (data[a] > data[b]).then(|| data.swap(a, b)).is_some()
}

/// Swaps the subslice of `count` elements starting from `left` with the equal length subslice
/// ending at `right`.
///
/// Panics if `left` or `right` are out of bounds, or if the subslices overlap.
fn swap_parts<T: Ord>(data: &mut [T], left: usize, right: usize, count: usize) {
    let inner = data[left..=right].as_mut();
    let (left, tail) = inner.split_at_mut(count);
    let mid = tail.len() - count;
    let right = tail[mid..].as_mut();
    left.swap_with_slice(right);
}

/// Partitions `data` into three parts, using the element at `index` as the pivot. Returns `(a, d)`,
/// where `a` is the index of the first element equal to the pivot, and `d` is the index of the
/// last element equal to the pivot.
///
/// After the partitioning, the slice is arranged as follows:
/// ```text
///  ┌────────────┐
///  │ x < pivot  │ x == data[i] where i in ..a
///  ├────────────┤
///  │ x == pivot │ i in a..=d
///  ├────────────┤
///  │ x > pivot  │ i in d+1..
///  └────────────┘
/// ```
///
/// # Panics
///
/// Panics if `index` is out of bounds.
fn ternary<T: Ord>(data: &mut [T], index: usize) -> (usize, usize) {
    if data.len() == 1 {
        assert!(index == 0);
        return (0, 0);
    }
    data.swap(0, index);
    let mut v = 0;
    let (mut l, mut r) = (0, data.len() - 1);
    let (mut p, mut q) = (1, r - 1);
    let (mut i, mut j) = (l, r);
    match data[v].cmp(&data[r]) {
        Ordering::Less => r = q,
        Ordering::Greater => {
            data.swap(l, r);
            l = p;
            v = r;
        }
        _ => {}
    }
    loop {
        i += 1;
        j -= 1;
        // Increment i until data[i] >= data[k]
        while data[i] < data[v] {
            i += 1;
        }
        // Decrement j until data[j] <= data[k]
        while data[j] > data[v] {
            j -= 1;
        }
        // Exchange data[i] and data[j] if i < j,
        // otherwise break out of the loop.
        match i.cmp(&j) {
            Ordering::Less => {
                data.swap(i, j);
                if data[i] == data[v] {
                    data.swap(p, i);
                    p += 1;
                }
                if data[j] == data[v] {
                    data.swap(q, j);
                    q -= 1;
                }
            }
            Ordering::Greater => break,
            Ordering::Equal => {
                i += 1;
                j -= 1;
                break;
            }
        }
    }

    // Cleanup. At this point, the following assertions hold:

    // let pivot = &data[v];
    // for (k, x) in data.iter().enumerate() {
    //     if k < l {
    //         // Part 0: ..l
    //         assert!(x < pivot);
    //     } else if k < p {
    //         // Part 1: l..p
    //         assert!(x == pivot);
    //     } else if k <= j {
    //         // Part 2: p..=j
    //         assert!(x < pivot);
    //     } else if k < i {
    //         // Part 3: j+1..i
    //         assert!(x == pivot);
    //     } else if k <= q {
    //         // Part 4: i..=q
    //         assert!(x > pivot);
    //     } else if k <= r {
    //         // Part 5: q+1..=r
    //         assert!(x == pivot);
    //     } else {
    //         // Part 6: r+1..
    //         assert!(x > pivot);
    //     }
    // }

    // Swap parts 1 and 2
    swap_parts(data, l, j, (p - l).min(j + 1 - p));

    // Swap parts 3 and 4
    swap_parts(data, i, r, (r - q).min(q + 1 - i));

    let a = l + j + 1 - p;
    let d = i + r - q - 1;

    // let pivot = &data[a];
    // for (k, x) in data.iter().enumerate() {
    //     if k < a {
    //         assert!(x < pivot);
    //     } else if k <= d {
    //         assert!(x == pivot);
    //     } else {
    //         assert!(x > pivot);
    //     }
    // }
    (a, d)
}
