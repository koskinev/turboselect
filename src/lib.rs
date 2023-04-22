mod rand;
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

/// Shuffles `data` in place, using the Fisher-Yates algorithm.
pub(crate) fn shuffle<T>(data: &mut [T]) {
    let mut rng = usize::rng(0);
    let len = data.len();
    for i in 0..len - 1 {
        let j = rng.get_bounded(i, len);
        data.swap(i, j);
    }
}

/// Partitions `data` into three parts, using the `k`th element as the pivot. Returns `(lt, gt)`,
/// where `lt` is the index of the first element equal to the pivot, and `gt` is the index of the
/// last element equal to the pivot.
///
/// After the partitioning:
/// * The first `lt` elements are less than the pivot.
/// * The next `gt - lt + 1` elements are equal to the pivot.
/// * The last `right - gt` elements are greater than the pivot.
///
/// # Panics
///
/// Panics if `k` is out of bounds.
pub(crate) fn ternary_partion<T: Ord>(data: &mut [T], mut k: usize) -> (usize, usize) {
    data.swap(0, k);
    k = 0;
    let (mut l, mut r) = (0, data.len() - 1);
    let (mut p, mut q) = (1, r - 1);
    let (mut i, mut j) = (l, r);
    match data[k].cmp(&data[r]) {
        std::cmp::Ordering::Less => r = q,
        std::cmp::Ordering::Greater => {
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
            std::cmp::Ordering::Less => {
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
            std::cmp::Ordering::Equal => {
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

    (l + j + 1 - p, i + r - q)
}

/// Partitions `data` into five parts, using the `j`th and `k`th elements as the pivots. Returns
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
/// Panics if `j` or `k` is out of bounds.
pub(crate) fn quintary_partition<T: Ord>(
    data: &mut [T],
    mut j: usize,
    mut k: usize,
) -> (usize, usize, usize, usize) {
    let (mut l, mut r) = (0, data.len() - 1);
    if data[j] > data[k] {
        data.swap(l, k);
        data.swap(r, j);
    } else {
        data.swap(l, j);
        data.swap(r, k);
    }
    (j, k) = (l, r);
    let (mut p, mut q) = (1, r - 1);
    todo!()
}
