#![feature(specialization)]

mod dbg;
mod rand;
use std::cmp::Ordering;

use dbg::Dbg;
pub(crate) use rand::Rng;

#[cfg(test)]
mod tests;

const ALPHA: f64 = 0.5;
const BETA: f64 = 0.25;
const CUT: usize = 600;

fn size(n: usize) -> usize {
    let n = n as f64;
    let f = n.powf(2. / 3.) * n.ln().powf(1. / 3.);
    (ALPHA * f).ceil().min(n - 1.) as usize
}

fn gap(s: usize, n: usize) -> usize {
    let n = n as f64;
    (BETA * (s as f64) * n.ln()).powf(0.5) as usize
}

fn swap<T: Ord>(data: &mut [T], a: usize, b: usize) -> bool {
    let swap = data[a] > data[b];
    if swap {
        data.swap(a, b)
    }
    swap
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
    2
}

fn guess_pivot<T: Ord>(data: &mut [T], k: usize) -> usize {
    match data.len() {
        len @ 9.. => {
            let f = len / 9;
            let a = 3 * f;
            let b = 4 * f;
            let c = 5 * f;
            let d = 6 * f;
            for index in a..d {
                sort_3(data, index - a, index, index + a);
            }
            for index in b..c {
                sort_3(data, index - f, index, index + f);
            }
            let k = (k * f) / len;
            select_nth_small(&mut data[b..c], k);
            b + k
        }
        // len @ 5.. => {
        //     let blocks = len / 5;
        //     let mut index = 0;
        //     let (start, end) = (2 * blocks, 3 * blocks);
        //     for mid in start..end {
        //         median_of_5(
        //             data,
        //             index,
        //             index + 1,
        //             mid,
        //             3 * blocks + index,
        //             3 * blocks + index + 1,
        //         );
        //         index += 2;
        //     }
        //     select_nth_small(&mut data[start..end], blocks / 2);
        //     len / 2
        // }
        5.. => {
            shuffle(data, 5);
            median_of_5(data, 0, 1, 2, 3, 4)
        }
        4 => sort_4(data, 0, 1, 2, 3),
        3 => sort_3(data, 0, 1, 2),
        2 => sort_2(data, 0, 1),
        1 => 0,
        _ => panic!("median_of_medians: empty slice"),
    }
}

/// Finds the `k`th smallest element in `data`. Returns the `(a, b)` where `a <= k <= b`.
/// After the call, `data` is partitioned into three parts:
/// - Elements in the range `0..a` are less than the `k`th smallest element
/// - Elements in the range `a..=b` are equal to the `k`th smallest element
/// - Elements in the range `b+1..` are greater than the `k`th smallest element
///
/// # Panics
///
/// Panics if `k >= data.len()`.
fn select_nth_small<T: Ord>(data: &mut [T], k: usize) -> (usize, usize) {
    assert!(k < data.len());
    // eprintln!("Start select_nth_small: k = {k}, data.len() = {}, data = {:?}", data.len(), Dbg(&data));
    match data.len() {
        5.. => {
            let k_mom = guess_pivot(data, k);
            // eprintln!("  Selected pivot at index = {k_mom}, data = {:?}", Dbg(&data));
            let (a, b) = ternary_partion(data, k_mom);
            // eprintln!("  Pivot in the range {a}..={b}, data = {:?}", Dbg(&data));
            match (a, b) {
                (a, _) if k < a => select_nth_small(&mut data[..a], k),
                (_, b) if k > b => {
                    let (u, v) = select_nth_small(&mut data[b + 1..], k - b - 1);
                    (b + 1 + u, b + 1 + v)
                }
                (u, v) => (u, v),
            }
        }
        4 => {
            sort_4(data, 0, 1, 2, 3);
            let (mut u, mut v) = (0, 3);
            while data[u] != data[k] {
                u += 1;
            }
            while data[v] != data[k] {
                v -= 1;
            }
            (u, v)
        }
        3 => {
            sort_3(data, 0, 1, 2);
            let (mut u, mut v) = (0, 2);
            while data[u] != data[k] {
                u += 1;
            }
            while data[v] != data[k] {
                v -= 1;
            }
            (u, v)
        }
        2 => {
            sort_2(data, 0, 1);
            if data[0] == data[1] {
                (0, 1)
            } else {
                (k, k)
            }
        }
        1 => (k, k),
        _ => panic!("select from empty slice"),
    }
}

pub fn select_nth<T: Ord>(mut data: &mut [T], k: usize) -> &T {
    let (u, v) = guess_pivots(data, k);
    let (a, b, c, d) = if k < data.len() / 2 {
        quintary_partition_a(data, u, v)
    } else {
        quintary_partition_b(data, u, v)
    };
    todo!()
}

pub fn guess_pivots<T: Ord>(data: &mut [T], k: usize) -> (usize, usize) {
    let len = data.len();
    let s = size(len);
    let g = gap(s, len);
    shuffle(data, s);

    let u = (((k + 1) * s) / len).saturating_sub(g);
    let v = (((k + 1) * s) / len + g).min(s - 1);

    if s < CUT {
        select_nth_small(&mut data[..s], u);
        select_nth_small(&mut data[u..s], v - u);
    } else {
        select_nth(&mut data[..s], u);
        select_nth(&mut data[u..s], v - u);
    }
    (u, v)
}

/// Swaps elements in the range `..count`, with a random element in the range `index..count`,
/// where `index` is the index of the element.
pub fn shuffle<T>(data: &mut [T], count: usize) {
    let mut rng = usize::rng(0);
    let len = data.len();
    for i in 0..count {
        let j = rng.get_bounded(i, len);
        data.swap(i, j);
    }
}

/// Partitions `data` into three parts, using the `k`th element as the pivot. Returns `(a, b)`,
/// where `a` is the index of the first element equal to the pivot, and `b` is the index of the
/// last element equal to the pivot.
///
/// After the partitioning:
/// * The first `a` elements are less than the pivot.
/// * The next `b - a + 1` elements are equal to the pivot.
/// * The last `data.len() - b + 1` elements are greater than the pivot.
///
/// # Panics
///
/// Panics if `k` is out of bounds.
pub fn ternary_partion<T: Ord>(data: &mut [T], mut k: usize) -> (usize, usize) {
    if data.len() == 1 {
        assert!(k == 0);
        return (0, 0);
    }
    data.swap(0, k);
    k = 0;
    let (mut l, mut r) = (0, data.len() - 1);
    let (mut p, mut q) = (1, r - 1);
    let (mut i, mut j) = (l, r);
    match data[k].cmp(&data[r]) {
        Ordering::Less => r = q,
        Ordering::Greater => {
            data.swap(l, r);
            l = p;
            k = r;
        }
        _ => {}
    }
    loop {
        loop {
            i += 1;
            if data[i] >= data[k] {
                break;
            }
        }
        loop {
            j -= 1;
            if data[j] <= data[k] {
                break;
            }
        }
        match i.cmp(&j) {
            Ordering::Less => {
                data.swap(i, j);
                if data[i] == data[k] {
                    data.swap(p, i);
                    p += 1;
                }
                if data[j] == data[k] {
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
    if p <= j {
        while p > l {
            data.swap(l, j);
            j -= 1;
            l += 1;
        }
    }
    if q >= i {
        while q < r {
            data.swap(i, r);
            r -= 1;
            i += 1;
        }
    }

    (l + j + 1 - p, i + r - q - 1)
}

/// Partitions `data` into five parts, using the `u`th and `v`th elements as the pivots. Returns
/// `(a, b, c, d)` where `0 <= a <= b < c <= d < data.len()`.
///
/// After the partitioning:
/// * The first `a` elements are less than the first pivot.
/// * The next `b - a + 1` elements are equal to the first pivot.
/// * The next `c - b - 1` elements are between the two pivots.
/// * The next `d - c + 1` elements are equal to the second pivot.
/// * The last `data.len() - d - 1` elements are greater than the second pivot.
///
/// # Panics
///
/// Panics if `u` or `v` is out of bounds.
pub(crate) fn quintary_partition_a<T: Ord>(
    data: &mut [T],
    mut u: usize,
    mut v: usize,
) -> (usize, usize, usize, usize) {
    let (mut l, mut r) = (0, data.len() - 1);
    if data[u] > data[v] {
        data.swap(l, v);
        data.swap(r, u);
    } else {
        data.swap(l, u);
        data.swap(r, v);
    }
    (u, v) = (l, r);
    let (mut pl, mut ph, mut q) = (1, 1, r - 1);
    let (mut i, mut j) = (l, r);
    loop {
        // B2: Increment i until data[i] >= data[u]
        loop {
            i += 1;
            if data[i] >= data[v] {
                break;
            }
            match data[i].cmp(&data[u]) {
                Ordering::Greater => data.swap(ph, i),
                Ordering::Less => continue,
                Ordering::Equal => {
                    data.swap(pl, i);
                    data.swap(pl, ph);
                    pl += 1;
                }
            }
            ph += 1;
        }
        // B3: Decrement j until data[j] < data[v]
        loop {
            j -= 1;
            match data[j].cmp(&data[v]) {
                Ordering::Greater => continue,
                Ordering::Less => break,
                Ordering::Equal => {
                    data.swap(j, q);
                    q -= 1;
                }
            }
        }
        // B4: Exchange data[i] and data[j] if i < j and repeat B2 and B3,
        // otherwise stop
        if i < j {
            data.swap(i, j);
            match data[i].cmp(&data[u]) {
                Ordering::Greater => {
                    data.swap(ph, i);
                    ph += 1;
                }
                Ordering::Equal => {
                    data.swap(i, ph);
                    data.swap(pl, ph);
                    pl += 1;
                    ph += 1;
                }
                _ => {}
            }
            if data[j] == data[v] {
                data.swap(j, q);
                q -= 1;
            }
        } else {
            break;
        }
    }
    // B5: Cleanup
    let a = l + i - ph;
    let b = a + pl - l;
    let d = r + j - q;
    let c = d + q - r;
    data[pl..j + 1].rotate_left(ph - pl);
    data[i..r + 1].rotate_right(r - q);
    (a, b, c, d)
}

pub(crate) fn quintary_partition_b<T: Ord>(
    data: &mut [T],
    mut u: usize,
    mut v: usize,
) -> (usize, usize, usize, usize) {
    todo!()
}
