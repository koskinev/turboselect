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
    (data[a] > data[b]).then(|| data.swap(a, b)).is_some()
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

fn select_first<T: Ord>(data: &mut [T]) -> (usize, usize) {
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

fn select_last<T: Ord>(data: &mut [T]) -> (usize, usize) {
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
fn select_nth_small<T: Ord>(data: &mut [T], k: usize) -> (usize, usize) {
    assert!(k < data.len());
    // eprintln!(
    //     "Start select_nth_small: k = {k}, data.len() = {}",
    //     data.len()
    // );
    match data.len() {
        5.. => {
            if k == 0 {
                select_first(data)
            } else if k == data.len() - 1 {
                select_last(data)
            } else {
                let k_mom = guess_pivot(data, k);
                // eprintln!("  Selected pivot at index = {k_mom}, data = {:?}", Dbg(&data));
                let (a, d) = ternary_partion(data, k_mom);
                // eprintln!("  Pivot in the range {a}..={b}, data = {:?}", Dbg(&data));
                match (a, d) {
                    (a, _) if k < a => select_nth_small(&mut data[..a], k),
                    (_, b) if k > b => {
                        let (u, v) = select_nth_small(&mut data[b + 1..], k - b - 1);
                        (b + 1 + u, b + 1 + v)
                    }
                    (u, v) => (u, v),
                }
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

pub fn select_nth<T: Ord>(data: &mut [T], k: usize) -> (usize, usize) {
    if k == 0 {
        select_first(data)
    } else if k == data.len() - 1 {
        select_last(data)
    } else if data.len() < CUT {
        let (a, d) = select_nth_small(data, k);
        (a, d)
    } else {
        eprintln!("select_nth: k = {k}, data.len() = {}", data.len());
        let (u_a, u_d, v_a, v_d) = prepare_partition(data, k);
        let (a, b, c, d) = if k < data.len() / 2 {
            quintary_partition_left(data, u_a, u_d, v_a, v_d)
        } else {
            quintary_partition_right(data, u_a, u_d, v_a, v_d)
        };
        match k {
            k if b <= k && k <= c => select_nth(&mut data[b..=c], k - b),
            k if k < a => select_nth(&mut data[..a], k),
            k if a <= k && k < b => (a, b - 1),
            k if c < k && k <= d => (c + 1, d),
            k => select_nth(&mut data[d + 1..], k - d - 1),
        }
    }
}

pub fn prepare_partition<T: Ord>(data: &mut [T], k: usize) -> (usize, usize, usize, usize) {
    let len = data.len();
    let s = size(len);
    shuffle(data, s);

    let g = gap(s, len);
    let u = (((k + 1) * s) / len).saturating_sub(g);
    let v = (((k + 1) * s) / len + g).min(s - 1);

    let (v_a, v_d) = select_nth(&mut data[..s], v);
    let (u_a, u_d) = select_nth(&mut data[..v_a], u);

    let q = len - s + v_a;
    for k in 0..s - v_a {
        data.swap(v_a + k, q + k);
    }
    (u_a, u_d, q, q + v_d - v_a)
}

/// Swaps elements in the range `..count`, with a random element in the range `index..count`,
/// where `index` is the index of the element.
pub fn shuffle<T>(data: &mut [T], count: usize) {
    let mut rng = usize::rng(data.as_ptr() as u64);
    let len = data.len();
    for i in 0..count {
        let j = rng.get_bounded(i, len);
        data.swap(i, j);
    }
}

/// Partitions `data` into three parts, using the `k`th element as the pivot. Returns `(a, d)`,
/// where `a` is the index of the first element equal to the pivot, and `d` is the index of the
/// last element equal to the pivot.
///
/// After the partitioning, the slice is arranged as follows:
/// ```text
///  ┌────────────────┐
///  │ x < data[a]    │ x == data[i] where i in ..a
///  ├────────────────┤
///  │ x == data[a]   │ i in a..=d
///  ├────────────────┤
///  │ x > data[d]    │ i in d+1..
///  └────────────────┘
/// ```
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
            if !swap(data, l, j) {
                break;
            } else {
                j -= 1;
                l += 1;
            }
        }
    }
    if q >= i {
        while q < r {
            if !swap(data, i, r) {
                break;
            } else {
                i += 1;
                r -= 1;
            }
        }
    }

    (l + j + 1 - p, i + r - q - 1)
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
pub(crate) fn quintary_partition_left<T: Ord>(
    mut data: &mut [T],
    u_a: usize,
    u_d: usize,
    v_a: usize,
    v_d: usize,
) -> (usize, usize, usize, usize) {
    assert!(u_d <= v_a && data[u_d] <= data[v_a]);
    data = &mut data[u_a..=v_d];
    let r = data.len() - 1;
    let mut l = 1 + u_d - u_a;
    let mut p = l;
    let mut q = r + v_a - v_d - 1;
    let mut i = p - 1;
    let mut j = q + 1;
    loop {
        // B2: Increment i until data[i] >= data[r]
        loop {
            i += 1;
            if data[i] >= data[r] {
                break;
            }
            match data[i].cmp(&data[0]) {
                Ordering::Greater => data.swap(p, i),
                Ordering::Less => continue,
                Ordering::Equal => {
                    data.swap(p, i);
                    data.swap(l, p);
                    l += 1;
                }
            }
            p += 1;
        }
        // B3: Decrement j until data[j] < data[r]
        loop {
            j -= 1;
            match data[j].cmp(&data[r]) {
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
            match data[i].cmp(&data[0]) {
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
            if data[j] == data[r] {
                data.swap(j, q);
                q -= 1;
            }
        } else {
            break;
        }
    }

    // B5: Cleanup. At this point, the pivots are at the ends of the slice and the slice is
    // partitioned into five parts:
    //  +-----------------------+
    //  | x == data[0]          | x == data[k] where k in ..l
    //  +-----------------------+
    //  | data[0] < x < data[r] | k in l..p
    //  +-----------------------+
    //  | x < data[0]           | k in p..=j
    //  +-----------------------+
    //  | x > data[r]           | k in i..=q
    //  +-----------------------+
    //  | x == data[r]          | k in q+1..
    //  +-----------------------+
    let a = i - p;
    let b = a + l;
    let d = r + j - q;
    let c = d + q - r;

    // Swap the second and the middle parts.
    for k in 0..(j + 1 - p).min(p - l) {
        data.swap(l + k, j - k)
    }
    // Swap the first and second parts.
    for k in 0..l.min(j + 1 - p) {
        data.swap(k, b - k - 1)
    }
    // Swap the fourth and fifth parts.
    for k in 0..(q + 1 - i).min(r - q) {
        data.swap(i + k, r - k)
    }
    (a + u_a, b + u_a, c + u_a, d + u_a)
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
pub(crate) fn quintary_partition_right<T: Ord>(
    mut data: &mut [T],
    u_a: usize,
    u_d: usize,
    v_a: usize,
    v_d: usize,
) -> (usize, usize, usize, usize) {
    assert!(u_d <= v_a && data[u_d] <= data[v_a]);
    data = &mut data[u_a..=v_d];
    let r = data.len() - 1;
    let mut p = 1 + u_d - u_a;
    let mut q = r + v_a - v_d - 1;
    let mut h = q;
    let mut i = p - 1;
    let mut j = q + 1;
    loop {
        // C2: Increment i until data[i] > data[0]
        loop {
            i += 1;
            match data[i].cmp(&data[0]) {
                Ordering::Greater => break,
                Ordering::Less => continue,
                Ordering::Equal => {
                    data.swap(p, i);
                    p += 1;
                }
            }
        }
        // C3: Decrement j until data[j] <= data[0]
        loop {
            j -= 1;
            if data[j] <= data[0] {
                break;
            }
            match data[j].cmp(&data[r]) {
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
        // C4: Exchange data[i] and data[j] if i < j and repeat B2 and B3,
        // otherwise stop
        if i < j {
            data.swap(i, j);
            if data[i] == data[0] {
                data.swap(i, p);
                p += 1;
            }
            match data[j].cmp(&data[r]) {
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

    // B5: Cleanup. At this point, the pivots are at the ends of the slice and the slice is
    // partitioned into five parts:
    //  +-----------------------+
    //  | x == data[0]          | x == data[k] where k in ..p
    //  +-----------------------+
    //  | x < data[0]           | k in p..i
    //  +-----------------------+
    //  | x > data[r]           | k in i..=q
    //  +-----------------------+
    //  | data[0] < x < data[r] | k in q+1..=h
    //  +-----------------------+
    //  | x == data[r]          | k in h+1..
    //  +-----------------------+

    let a = i - p;
    let b = a + p;
    let d = r + j - q;
    let c = d + h - r;

    // Swap the middle and fourth parts.
    for k in 0..(q + 1 - i).min(h - q) {
        data.swap(i + k, h - k);
    }
    // Swap the fourth and last parts.
    for k in 0..(r - h).min(q + 1 - i) {
        data.swap(c + k + 1, r - k);
    }
    // Swap the first and second parts.
    for k in 0..p.min(i - p) {
        data.swap(k, b - k - 1);
    }
    (a + u_a, b + u_a, c + u_a, d + u_a)
}
