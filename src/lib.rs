#![feature(specialization)]
#![feature(strict_provenance)]
#![feature(sized_type_properties)]
#![feature(maybe_uninit_slice)]
#![feature(ptr_sub_ptr)]

mod dbg;
mod pcg_rng;
mod slice_sort;
use core::mem::MaybeUninit;
use std::{cmp::Ordering, ops::Index};

use dbg::Dbg;
use pcg_rng::PCGRng;

#[cfg(test)]
mod tests;

const ALPHA: f64 = 0.5;
const BETA: f64 = 0.25;
const BLOCK: usize = 4;
const CUT: usize = 1000;

/// Hole represents a hole in a slice i.e., an index without valid value
/// (because it was moved from or duplicated).
/// In drop, `Hole` will restore the slice by filling the hole
/// position with the value that was originally removed.
struct Hole<'a, T: 'a> {
    data: &'a mut [T],
    elt: core::mem::ManuallyDrop<T>,
    pos: usize,
}

impl<'a, T> Hole<'a, T> {
    /// Create a new `Hole` at index `pos`.
    ///
    /// Unsafe because pos must be within the data slice.
    #[inline]
    unsafe fn new(data: &'a mut [T], pos: usize) -> Self {
        debug_assert!(pos < data.len());
        // SAFE: pos should be inside the slice
        let elt = unsafe { core::ptr::read(data.get_unchecked(pos)) };
        Hole {
            data,
            elt: core::mem::ManuallyDrop::new(elt),
            pos,
        }
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }

    /// Returns a reference to the element removed.
    #[inline]
    fn element(&self) -> &T {
        &self.elt
    }

    /// Returns a reference to the element at `index`.
    ///
    /// Unsafe because index must be within the data slice and not equal to pos.
    #[inline]
    unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        unsafe { self.data.get_unchecked(index) }
    }

    /// Takes the element at `index` and moves it to the previous hole position, changing the
    /// hole to `index`.
    ///
    /// Unsafe because index must be within the data slice and not equal to pos.
    #[inline]
    unsafe fn move_to(&mut self, index: usize) {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        unsafe {
            let ptr = self.data.as_mut_ptr();
            let index_ptr: *const _ = ptr.add(index);
            let hole_ptr = ptr.add(self.pos);
            core::ptr::copy_nonoverlapping(index_ptr, hole_ptr, 1);
        }
        self.pos = index;
    }
}

impl<T> Drop for Hole<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // fill the hole again
        unsafe {
            let pos = self.pos;
            core::ptr::copy_nonoverlapping(&*self.elt, self.data.get_unchecked_mut(pos), 1);
        }
    }
}

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

        // Move sample elements greater than the higher pivot to the end of the slice
        unordered_swap(data, v_d + 1, len - 1, s - v_d - 1);

        // Move sample elements equal to the higher pivot before the elements just
        // moved to the end of the slice
        unordered_swap(data, v_a, len - s + v_d, v_d - v_a + 1);

        (u_a, u_d, len - s + v_a, len - s + v_d)
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
    unordered_swap(data, l, j, (j + 1 - p).min(p - l));

    // Swap parts 1 and 2.
    unordered_swap(data, s, b - 1, (l - s).min(j + 1 - p));

    // Swap parts 4 and 5.
    unordered_swap(data, i, e, (q + 1 - i).min(e - q));

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
    unordered_swap(data, i, h, (q + 1 - i).min(h - q));

    // Swap parts 4 and 5
    unordered_swap(data, c + 1, e, (e - h).min(q + 1 - i));

    // Swap parts 1 and 2
    unordered_swap(data, s, b - 1, (p - s).min(i - p));

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

/// Rotates the elements so that the element at index `a_src` moves to index `a_dst`,
/// the element at index `b_src` moves to index `b_dst`.
///
/// Panics if any of the indices are out of bounds.
fn rotate_4<T>(data: &mut [T], a_src: usize, a_dst: usize, b_src: usize, b_dst: usize) {
    let a_src = &mut data[a_src] as *mut T;
    let a_dst = &mut data[a_dst] as *mut T;
    let b_src = &mut data[b_src] as *mut T;
    let b_dst = &mut data[b_dst] as *mut T;
    unsafe {
        let tmp = std::mem::ManuallyDrop::new(std::ptr::read(a_src));
        b_dst.copy_to(a_src, 1);
        b_src.copy_to(b_dst, 1);
        a_dst.copy_to(b_src, 1);
        a_dst.copy_from(&*tmp, 1);
    }
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

/// Swaps `a` and `b` if `a > b`, and returns true if the swap was performed.
fn sort_2<T: Ord>(data: &mut [T], a: usize, b: usize) -> bool {
    let swap = data[a] > data[b];
    let offset = (b as isize - a as isize) * swap as isize;
    unsafe {
        let a = &mut data[a] as *mut T;
        let x = a.offset(offset);
        core::ptr::swap(a, x);
        swap
    }
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

/// Performs an unordered swap of the first `count` elements starting from `left` with the last
/// `count` elements ending at and including`right`.
fn unordered_swap<T: Ord>(data: &mut [T], mut left: usize, mut right: usize, count: usize) {
    if count == 0 {
        return;
    }
    debug_assert!(left + count <= right);
    debug_assert!(right <= data.len());
    let inner = data[left..=right].as_mut();
    (left, right) = (0, inner.len() - 1);
    unsafe {
        let mut hole = Hole::new(inner, left);
        hole.move_to(right);
        for _ in 1..count {
            left += 1;
            hole.move_to(left);
            right -= 1;
            hole.move_to(right);
        }
    }
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
    unordered_swap(data, l, j, (p - l).min(j + 1 - p));

    // Swap parts 3 and 4
    unordered_swap(data, i, r, (r - q).min(q + 1 - i));

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

fn l_partition<T: Ord>(data: &mut [T], u: usize, v: usize) -> (usize, usize) {
    // Optimal Partitioning for Dual-Pivot Quicksort by Aumüller & Dietzfelbinger, Algorithm 3:
    //
    // procedure L-Partition(A, p, q, left, right, pos_p, pos_q)
    //     i ← left + 1; k ← right − 1; j ← i;
    //     while j ≤ k do
    //         while q < A[k] do
    //             k ← k − 1;
    //         // At this point A[k] <= q
    //         while A[j] < q do
    //             if A[j] < p then
    //                 swap A[i] and A[j];
    //                 i ← i + 1;
    //             j ← j + 1;
    //         // At this point A[j] >= q and A[j] >= p
    //         if j < k then
    //             if A[k] > p then
    //                 rotate3(A[k], A[j], A[i]);
    //                 i ← i + 1;
    //             else
    //                 swap A[j] and A[k];
    //             k ← k − 1;
    //         j ← j + 1;
    //     swap A[left] and A[i − 1];
    //     swap A[right] and A[k + 1];
    //     pos_p ← i − 1; pos_q ← k + 1;
    //
    // The resulting partition is:
    //   +-------+-------------+-------+
    //   | x < p | p <= x <= q | x > q |
    //   +-------+-------------+-------+
    //         i  j             k

    // The slice should have at least 2 * BLOCK elements.
    let len = data.len();
    assert!(len >= 2 * BLOCK);

    // Put the smaller pivot at the beginning of the slice and the larger at the end.
    sort_2(data, u, v);
    rotate_4(data, u, 0, v, len - 1);

    let (mut i, mut k, mut j) = (1, len - 1, 1);
    // unsafe {
    //     // Read the pivots onto the stack
    //     let (p, _p_guard) = read_pivot(data, 0);
    //     let (q, _q_guard) = read_pivot(data, len - 1);

    //     let origin = data.as_mut_ptr().add(1);

    //     let mut to_right = PtrCache::new(origin, |x| x > q);
    //     let mut to_mid_or_right = PtrCache::new(origin, |x| x > p);

    //     let mut to_left = PtrCache::new_back(origin, len - 1, |x| x < p);
    //     let mut to_mid_or_left = PtrCache::new_back(origin, len - 1, |x| x < q);

    //     // Swap elements between left_to_right and right_to_left
    // }

    (i - 1, k + 1)
}

/// Moves the elements at indices `p` and `q` to the beginning and end of the slice so that
/// `data[p] <= data[q]`. Then returns the pivots and the interior of the slice as a triple
/// `low, mid, high`.
fn read_pivots<T: Ord>(data: &mut [T], p: usize, q: usize) -> (Hole<T>, &mut [T], Hole<T>) {
    debug_assert!(data.len() >= 2);
    sort_2(data, p, q);
    data.swap(0, p);
    data.swap(data.len() - 1, q);
    let (head, tail) = data.split_at_mut(1);
    let (mid, tail) = tail.split_at_mut(tail.len() - 1);
    let head = unsafe { Hole::new(head, 0) };
    let tail = unsafe { Hole::new(tail, 0) };
    (head, mid, tail)
}

fn ternary_block_partition_left<T: Ord>(
    data: &mut [T],
    u: usize,
    v: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {

    sort_2(data, u, v);
    data.swap(0, u);
    data.swap(data.len() - 1, v);
    let (p, tail) = data.split_first_mut().unwrap();
    let (q, mid) = tail.split_last_mut().unwrap();
    let n = mid.len();
    
    let (mut i, mut j, mut k) = (0, 0, 0);
    let mut le_q = 0;
    let mut lt_p = 0;
    unsafe {
        let mut block: [MaybeUninit<u8>; BLOCK] = MaybeUninit::uninit().assume_init();
        while k < n {
            // data[..i] < p <= data[i..j] <= q < data[j..k] 
            
            let t = (n - k).min(BLOCK).try_into().unwrap_or(BLOCK as u8);
        
            // Scan elements after k. If elem <= q, place it between j and k. 
            for o in 0..t {
                let elem = mid.get_unchecked(k + o as usize);
                block[le_q].write(o);
                le_q += !is_less(q, elem) as usize;
            }
            for (c, u) in block.iter().enumerate().take(le_q) {
                let b = u.assume_init() as usize;
                mid.swap(j + c, k + b);
            }
            k += t as usize;

            // Scan the moved elements. If elem < p, place it before i.
            for c in 0..(le_q as u8) {
                let elem = mid.get_unchecked(j + c as usize);
                block[lt_p].write(c);
                lt_p += is_less(elem, p) as usize;
            }
            for u in block.iter().take(lt_p) {
                let b = u.assume_init() as usize;
                mid.swap(i, j + b);
                i += 1;
            }

            // Reset counters
            (lt_p, le_q) = (0, 0);
            j += le_q;
        }
    }
    let (u, v) = (i, j + 1);
    data.swap(u, 0);
    data.swap(v, data.len() - 1);

    (u, v)
}

fn ternary_block_partition_right<T: Ord>(
    data: &mut [T],
    u: usize,
    v: usize,
    is_greater: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    sort_2(data, u, v);
    data.swap(0, u);
    data.swap(data.len() - 1, v);
    let (p, tail) = data.split_first_mut().unwrap();
    let (q, mid) = tail.split_last_mut().unwrap();
    let n = mid.len();
    let last = n - 1;

    let (mut i, mut j, mut k) = (last, last, last);
    let mut ge_p = 0;
    let mut gt_q = 0;
    unsafe {
        let mut block: [MaybeUninit<u8>; BLOCK] = MaybeUninit::uninit().assume_init();
        while i > 0 {
            // data[..i] < p <= data[i..j] <= q < data[j..k] 
            let t = i.min(BLOCK).try_into().unwrap_or(BLOCK as u8);

            // Scan elements before i. If elem >= p, place it between j and k.
            for o in 0..t {
                let elem = mid.get_unchecked(i - o as usize);
                block[ge_p].write(o);
                ge_p += !is_greater(p, elem) as usize;
            }
            for (c, u) in block.iter().enumerate().take(ge_p) {
                let b = u.assume_init() as usize;
                mid.swap(j - c, i - b);
            }
            i -= t as usize;

            // Scan the moved elements. If elem > q, move it to the right of k.
            for o in 0..(ge_p as u8) {
                let elem = mid.get_unchecked(j - o as usize);
                block[gt_q].write(o);
                gt_q += is_greater(elem, q) as usize;
            }
            for u in block.iter().take(gt_q) {
                let b = u.assume_init() as usize;
                mid.swap(k, j - b);
                k -= 1;
            }
            j -= ge_p;
            (ge_p, gt_q) = (0, 0);
        }
    }
    let (u, v) = (last - i, last - j - 1);
    data.swap(u, 0);
    data.swap(v, data.len() - 1);

    (u, v)
}

/// A cache of pointers to elements that satisfy a predicate.
struct PtrCache<T, F>
where
    F: Fn(&T) -> bool,
{
    /// The pointer to the first element of the slice.
    origin: *mut T,

    /// The index of the first element in the block.
    index: usize,

    /// The offsets from the start of the block to the elements that satisfy the predicate.
    offsets: [std::mem::MaybeUninit<u8>; BLOCK],

    /// The index of the first initialized offset.
    init: u8,

    /// The number of initialized offsets.
    len: u8,

    /// The predicate to test the elements.
    test: F,
}

impl<T, F> PtrCache<T, F>
where
    F: Fn(&T) -> bool,
{
    /// Returns the number of elements that satisfy the predicate.
    fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the index of the first element of the block.
    fn start(&self) -> usize {
        self.index
    }

    /// Returns the index of the last element of the block.
    fn end(&self) -> usize {
        self.index + self.len()
    }

    /// Returns a new cache with the elements from the current cache that also satisfy the new
    /// predicate.
    fn filter<G>(&self, also: G) -> PtrCache<T, G>
    where
        G: Fn(&T) -> bool,
    {
        let mut cache = PtrCache {
            origin: self.origin,
            index: self.index,
            offsets: self.offsets,
            init: self.init,
            len: self.len,
            test: also,
        };
        let mut index = cache.init;
        let mut last = cache.init + cache.len - 1;
        unsafe {
            while index <= last {
                let offset = self.offsets[index as usize].assume_init();
                if !(cache.test)(&*cache.origin.add(cache.index + offset as usize)) {
                    cache.offsets.swap(index as usize, last as usize);
                    last -= 1;
                } else {
                    index += 1;
                }
            }
            cache.len = last - index + 1;
        }
        cache
    }

    /// Creates a new cache beginning at `origin`.
    unsafe fn new(origin: *mut T, test: F) -> Self {
        let mut this = Self {
            origin,
            index: 0,
            offsets: std::mem::MaybeUninit::uninit().assume_init(),
            init: 0,
            len: 0,
            test,
        };
        this.scan();
        this
    }

    /// Creates a new cache ending at `origin.add(len)`.
    unsafe fn new_back(origin: *mut T, len: usize, test: F) -> Self {
        let mut this = Self {
            origin,
            index: len - BLOCK,
            offsets: std::mem::MaybeUninit::uninit().assume_init(),
            init: 0,
            len: 0,
            test,
        };
        this.scan();
        this
    }

    /// Scans for elements that satisfy the predicate.
    unsafe fn scan(&mut self) {
        let mut offset = 0;
        self.len = 0;
        while offset < BLOCK {
            self.offsets[self.len as usize].write(offset as u8);
            self.len += (self.test)(unsafe { &*self.origin.add(self.index + offset) }) as u8;
            offset += 1;
        }
    }

    /// Moves the cache to the next block of elements.
    unsafe fn next(&mut self) {
        while self.len() == 0 {
            self.index += BLOCK;
            self.scan();
        }
    }

    /// Moves the cache to the previous block of elements.
    unsafe fn prev(&mut self) {
        while self.len() == 0 {
            self.index -= BLOCK;
            self.scan();
        }
    }

    /// Pops the last pointer from the cache.
    unsafe fn pop(&mut self) -> *mut T {
        self.len -= 1;
        let offset = self
            .offsets
            .get_unchecked((self.init + self.len) as usize)
            .assume_init();
        self.origin.add(self.index + offset as usize)
    }

    /// Pops the first pointer from the cache.
    unsafe fn pop_front(&mut self) -> *mut T {
        let offset = self.offsets.get_unchecked(self.init as usize).assume_init();
        self.init += 1;
        self.len -= 1;
        self.origin.add(self.index + offset as usize)
    }
}
