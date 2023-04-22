mod rand;
use std::cmp::Ordering;

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

fn swap<T: Ord>(data: &mut [T], i: usize, j: usize) {
    if let Some(b) = data.get(j) {
        if data.get(i) > Some(b) {
            data.swap(i, j)
        }
    }
}

fn median_of_medians<T: Ord>(data: &mut [T]) -> usize {
    let (mut index, end) = (0, data.len().min(25));
    shuffle(data, end);
    while index + 5 < end {
        swap(data, index, index + 1);
        swap(data, index + 2, index + 3);
        swap(data, index, index + 2);
        swap(data, index + 1, index + 3);
        swap(data, index + 2, index + 4);
        swap(data, index + 1, index + 2);
        swap(data, index + 2, index + 4);
        data.swap(index / 5, index + 2);
        index += 5;
    }

    swap(data, index, index + 3);
    swap(data, index + 1, index + 4);
    swap(data, index, index + 2);
    swap(data, index + 1, index + 3);
    swap(data, index, index + 1);
    swap(data, index + 2, index + 4);
    swap(data, index + 1, index + 2);
    swap(data, index + 3, index + 4);
    swap(data, index + 2, index + 3);

    data.len().min(5) / 2
}

pub fn select_nth<T: Ord>(mut data: &mut [T], k: usize) -> &T {
    let (u, v) = select_pivots(data, k);
    let (a,b,c,d) = if k < data.len() / 2 {
        quintary_partition_a(data, u, v)
    } else {
        quintary_partition_b(data, u, v)
    };
    todo!()
}

fn select_nth_small<T: Ord>(mut data: &mut [T], mut k: usize) -> &T {
    loop {
        if data.len() > 5 {
            let m = median_of_medians(data);
            let (u, v) = ternary_partion(data, m);
            match (u, v) {
                (u, _) if k < u => data = &mut data[..u],
                (_, v) if k > v => {
                    data = &mut data[v + 1..];
                    k -= v + 1;
                }
                _ => return &data[k],
            }
        } else {
            swap(data, 0, 3);
            swap(data, 1, 4);
            swap(data, 0, 2);
            swap(data, 1, 3);
            swap(data, 0, 1);
            swap(data, 2, 4);
            swap(data, 1, 2);
            swap(data, 3, 4);
            swap(data, 2, 3);
            return &data[k];
        }
    }
}

pub fn select_pivots<T: Ord>(data: &mut [T], k: usize) -> (usize, usize) {
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
            Ordering::Equal => {
                i += 1;
                j -= 1;
                break;
            }
            _ => break,
        }
    }
    let left = &mut data[l..j + 1];
    left.rotate_left(p - l);

    let right = &mut data[i..r + 1];
    right.rotate_right(r - q);

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

    todo!()
}

pub(crate) fn quintary_partition_b<T: Ord>(
    data: &mut [T],
    mut u: usize,
    mut v: usize,
) -> (usize, usize, usize, usize) {
    todo!()
}