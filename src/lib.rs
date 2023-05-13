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
const BLOCK: usize = 128;
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

type Block = [MaybeUninit<u8>; BLOCK];


/// Partitions the slice into three parts, using the elements at indices `u` and `v` as pivots.
/// Returns the indices of the first and last element of the middle part as a tuple `(p, q)`.
/// 
/// The resulting partition is:
/// ´´´text
/// ┌───────┬───────────────────┬──────┐
/// │ < low │ low <= .. <= high │ high │
/// └───────┴───────────────────┴──────┘
///          p                 q
/// ´´´
/// 
/// This variant of the partitioning algorithm tests the elements first against the lower pivot
/// and conditionally against the higher pivot. If most elements are expected to be less than 
/// the lower pivot, this variant is faster than `ternary_block_partition_right`. 
/// 
/// Panics if the indices are out of bounds.
fn ternary_block_partition_left<T: Ord>(
    data: &mut [T],
    u: usize,
    v: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    sort_2(data, u, v);
    data.swap(0, u);
    data.swap(data.len() - 1, v);
    let (low, tail) = data.split_first_mut().unwrap();
    let (high, mid) = tail.split_last_mut().unwrap();
    let n = mid.len();

    let (mut i, mut j, mut k) = (n - 1, n - 1, n - 1);
    let mut num_ge_low = 0;
    let mut num_gt_high = 0;
    unsafe {
        let mut block: Block = MaybeUninit::uninit().assume_init();
        while k > 0 {
            let size = (k + 1).min(BLOCK) as u8;

            //     | block |
            // ┌───────────┬───────┬────────────────────┬────────┐
            // │  ? .. ?   │ low < │ low <= ... <= high │ > high │
            // └───────────┴───────┴────────────────────┴────────┘
            //            k k+1   i i+1                j j+1
            //
            // Scan the block of elements ending at k. Then put each element x >= low to a temporary
            // part between the first and middle parts by swapping the element with an
            // element in the range k..i. This moves the first part towards the
            // beginning of the slice.
            for offset in 0..size {
                block[num_ge_low].write(offset);
                let elem = mid.get_unchecked(k - offset as usize);
                num_ge_low += !is_less(elem, low) as usize;
            }
            for (offset_i, offset_k) in block.iter().enumerate().take(num_ge_low) {
                let offset_k = offset_k.assume_init() as usize;
                mid.swap(i - offset_i, k - offset_k);
            }

            // Scan the elements moved to k..i in the previous step. If element is x > high, swap it
            // with the element before j and decrement j. The third part grows by one
            // element.
            for offset in 0..(num_ge_low as u8) {
                block[num_gt_high].write(offset);
                let elem = mid.get_unchecked(i - offset as usize);
                num_gt_high += is_less(high, elem) as usize;
            }
            for offset_i in block.iter().take(num_gt_high) {
                let offset_i = offset_i.assume_init() as usize;
                mid.swap(j, i - offset_i);
                j = j.wrapping_sub(1);
            }
            k = k.saturating_sub(size as usize);
            i = i.wrapping_sub(num_ge_low);

            // Reset the counters
            (num_gt_high, num_ge_low) = (0, 0);

            // The first part contains elements x < low. The elements in the range ..k have not been
            // scanned yet and are unordered.
            debug_assert!(if i < n {
                mid[k + 1..=i].iter().all(|x| is_less(x, low))
            } else {
                true
            });

            // The middle part contains elements low <= x <= high.
            debug_assert!(if j < n - 1 {
                let ge_low = mid[i.wrapping_add(1)..=j].iter().all(|x| !is_less(x, low));
                let le_high = mid[i.wrapping_add(1)..=j].iter().all(|x| !is_less(high, x));
                ge_low && le_high
            } else {
                true
            });

            // The last part contains elements x > high.
            debug_assert!(if j.wrapping_add(1) < n {
                mid[j.wrapping_add(1)..].iter().all(|x| is_less(high, x))
            } else {
                true
            });
        }
    }
    let (u, v) = (i.wrapping_add(1), j.wrapping_add(2));
    data.swap(u, 0);
    data.swap(v, data.len() - 1);

    (u, v)
}


/// Partitions the slice into three parts, using the elements at indices `u` and `v` as pivots.
/// Returns the indices of the first and last element of the middle part as a tuple `(p, q)`.
/// 
/// The resulting partition is:
/// ´´´text
/// ┌───────┬───────────────────┬──────┐
/// │ < low │ low <= .. <= high │ high │
/// └───────┴───────────────────┴──────┘
///          p                 q
/// ´´´
/// 
/// This variant of the partitioning algorithm tests the elements first against the higher pivot
/// and conditionally against the lower pivot. If most elements are expected to be greater than 
/// the higher pivot, this variant is faster than `ternary_block_partition_left`. 
/// 
/// Panics if the indices are out of bounds.
fn ternary_block_partition_right<T: Ord>(
    data: &mut [T],
    u: usize,
    v: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    sort_2(data, u, v);
    data.swap(0, u);
    data.swap(data.len() - 1, v);
    let (low, tail) = data.split_first_mut().unwrap();
    let (high, mid) = tail.split_last_mut().unwrap();
    let n = mid.len();

    let (mut i, mut j, mut k) = (0, 0, 0);
    let mut num_lt_low = 0;
    let mut num_le_high = 0;
    unsafe {
        let mut block: Block = MaybeUninit::uninit().assume_init();
        while k < n {
            let size = (n - k).min(BLOCK) as u8;

            //                                       | block |
            // ┌───────┬────────────────────┬────────┬─────────────┐
            // │ < low │ low <= ... <= high │ > high │   ? .. ?    │
            // └───────┴────────────────────┴────────┴─────────────┘
            //          i                    j        k
            //
            // Scan the block of elements beginning at k. Then put each element x <= high to the
            // middle part by swapping it with an element in the range j..k. Since elements in
            // j..k are x > high, this creates a temporary part between the middle an third parts,
            // where elements belong to either the first or the middle part. The third part towards
            // the end of the slice.
            for offset in 0..size {
                block[num_le_high].write(offset);
                let elem = mid.get_unchecked(k + offset as usize);
                num_le_high += !is_less(high, elem) as usize;
            }
            for (offset_j, offset_k) in block.iter().enumerate().take(num_le_high) {
                let offset_k = offset_k.assume_init() as usize;
                mid.swap(j + offset_j, k + offset_k);
            }

            // Scan the elements moved to the temporary part in the previous step. If x < low, swap
            // the element with the first element of the middle part (the element at i) and
            // increment i. Since the element at i is known to be x >= low, this moves
            // the middle part to the right by one element. The first part grows by one
            // element.
            for offset in 0..(num_le_high as u8) {
                block[num_lt_low].write(offset);
                let elem = mid.get_unchecked(j + offset as usize);
                num_lt_low += is_less(elem, low) as usize;
            }
            for offset_j in block.iter().take(num_lt_low) {
                let offset_j = offset_j.assume_init() as usize;
                mid.swap(i, j + offset_j);
                i += 1;
            }
            k += size as usize;
            j += num_le_high;

            // Reset the counters
            (num_lt_low, num_le_high) = (0, 0);

            // The first part contains elements x < low.
            debug_assert!(mid[..i].iter().all(|x| is_less(x, low)));

            // The middle part contains elements low <= x <= high.
            debug_assert!(mid[i..j].iter().all(|x| !is_less(x, low)));
            debug_assert!(mid[i..j].iter().all(|x| !is_less(high, x)));

            // The last part contains elements x > high. Elements after k have not been scanned
            // yet and are unordered.
            debug_assert!(mid[j..k].iter().all(|x| is_less(high, x)));
        }
    }
    let (u, v) = (i, j + 1);
    data.swap(u, 0);
    data.swap(v, data.len() - 1);

    (u, v)
}