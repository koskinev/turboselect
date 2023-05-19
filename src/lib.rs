#![feature(specialization)]
#![feature(strict_provenance)]
#![feature(sized_type_properties)]
#![feature(maybe_uninit_slice)]
#![feature(ptr_sub_ptr)]

mod dbg;
mod pcg_rng;
mod slice_sort;
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr, slice,
};
use pcg_rng::PCGRng;

#[cfg(test)]
mod tests;

const ALPHA: f64 = 0.5;
const BETA: f64 = 0.25;
const BLOCK: usize = 128;
const CUT: usize = 1000;

struct Block {
    offsets: [MaybeUninit<u8>; BLOCK],
}

impl Block {
    fn new() -> Self {
        Self {
            offsets: [MaybeUninit::uninit(); BLOCK],
        }
    }

    #[inline]
    unsafe fn write(&mut self, index: u8, value: u8) {
        self.offsets.get_unchecked_mut(index as usize).write(value);
    }

    #[inline]
    unsafe fn get(&self, index: u8) -> usize {
        self.offsets.get_unchecked(index as usize).assume_init() as usize
    }
}

struct Elem<'a, T> {
    /// The slice of elements.
    data: &'a mut [T],

    /// A pointer to the position of the current element.
    ptr: Option<*mut T>,

    /// A temporary storage for the value of the current element.
    tmp: MaybeUninit<T>,
}

impl<'a, T> Elem<'a, T> {
    /// Creates a new `Elem` from a slice of elements without selecting an element.
    fn new(data: &'a mut [T]) -> Self {
        Self {
            data,
            ptr: None,
            tmp: MaybeUninit::uninit(),
        }
    }

    /// Creates a new `Elem` from a single element.
    fn from_mut(elem: &'a mut T) -> Self {
        let data = slice::from_mut(elem);
        unsafe {
            let ptr = data.as_mut_ptr();
            let val = ptr.read();
            let tmp = MaybeUninit::new(val);
            Self {
                data,
                ptr: Some(ptr),
                tmp,
            }
        }
    }

    /// Returns a reference to the current element. Unsafe because the element may not be selected.
    unsafe fn element(&self) -> &T {
        debug_assert!(self.ptr.is_some());

        self.tmp.assume_init_ref()
    }

    /// Returns a reference to the element at `index`. Unsafe because index must be in bounds.
    unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index < self.data.len());

        self.data.get_unchecked(index)
    }

    /// Moves the position of the current element to `index`. This also moves the position of the
    /// element at `index` to the previous position of the current element.
    ///
    /// Unsafe because index must be in bounds and the current element must be selected.
    unsafe fn move_to(&mut self, index: usize) {
        debug_assert!(index < self.data.len());
        debug_assert!(self.ptr.is_some());

        let src = self.data.as_mut_ptr().add(index);
        debug_assert!(Some(src) != self.ptr);

        self.ptr.unwrap_unchecked().write(src.read());
        self.ptr = Some(src);
    }

    /// Selects the element at `index` as the current element. Unsafe because index must be in
    /// bounds.
    unsafe fn select(&mut self, index: usize) {
        debug_assert!(index < self.data.len());

        let src = self.data.as_mut_ptr().add(index);
        self.ptr = Some(src);
        self.tmp.write(ptr::read(src));
    }

    /// Swaps the current element with the element at `index`. Unsafe because index must be in
    /// bounds and the current element must be selected.
    unsafe fn swap(&mut self, index: usize) {
        debug_assert!(index < self.data.len());
        debug_assert!(self.ptr.is_some());

        let dst = self.data.as_mut_ptr().add(index);
        self.ptr.unwrap_unchecked().write(dst.read());
        dst.write(self.tmp.assume_init_read());
        self.ptr = None;
    }
}

impl<T> Drop for Elem<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Write the temporary value to the current element.
        if let Some(ptr) = self.ptr {
            unsafe {
                ptr::copy_nonoverlapping(self.tmp.assume_init_ref(), ptr, 1);
            }
        }
    }
}

fn median_5<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    sort_2(data, a, b);
    sort_2(data, c, d);
    sort_2(data, a, c);
    sort_2(data, b, d);
    sort_2(data, c, e);
    sort_2(data, b, c);
    sort_2(data, c, e);
    c
}

fn partition_at_first<T: Ord>(data: &mut [T], is_less: impl Fn(&T, &T) -> bool) -> (usize, usize) {
    // The index of the last element that is equal to the first element.
    let mut v = 0;
    for i in 1..data.len() {
        // If the element is less than the first element of the array, swap the elements
        // and set v = 0.
        if is_less(&data[i], &data[0]) {
            v = 0;
            data.swap(0, i);
        }
        // Otherwise, if the first element is not less than the element, it must be equal to
        // the element. Increment v and swap the element with the element at v.
        else if !is_less(&data[0], &data[i]) {
            v += 1;
            data.swap(i, v);
        }
    }
    (0, v)
}

fn partition_at_last<T: Ord>(data: &mut [T], is_less: impl Fn(&T, &T) -> bool) -> (usize, usize) {
    let v = data.len() - 1;
    let mut u = v;
    for i in (0..v).rev() {
        // If the element is greater than the last element of the array, swap the elements
        // and set u = v.
        if is_less(&data[v], &data[i]) {
            u = v;
            data.swap(i, v);
        }
        // Otherwise, if the last element is not greater than the element, it must be equal to
        // the element. Decrement u and swap the element with the element at u.
        else if !is_less(&data[i], &data[v]) {
            u -= 1;
            data.swap(i, u);
        }
    }
    (u, v)
}

/// For the given index and slice length, returns `(size, p, q)`, where `size` is the sample size
/// and `p` and `q` are the pivot positions
fn sample_parameters(index: usize, n: usize) -> (usize, usize, usize) {
    let index = index as f64;
    let n = n as f64;
    let f = n.powf(2. / 3.) * n.ln().powf(1. / 3.);
    let size = (ALPHA * f).ceil().min(n - 1.);
    let gap = (BETA * size * n.ln()).powf(0.5);
    let p = (index * size / n - gap).ceil().max(0.) as usize;
    let q = (index * size / n + gap).ceil().min(size - 1.) as usize;
    (size as usize, p, q)
}

fn prepare_partition_2<T: Ord, F>(
    data: &mut [T],
    index: usize,
    is_less: F,
    rng: &mut PCGRng,
) -> usize
where
    F: Fn(&T, &T) -> bool + Copy,
{
    // Take a random sample from the data for pivot selection
    let (len, k, q) = sample_parameters(index, data.len());
    let sample = sample(data, len, rng);

    // Select the index for the pivot
    let index = if (len - k) < (q + 1).min(len / 2) {
        k
    } else if (q + 1) < (len / 2) {
        q
    } else {
        len / 2
    };

    // Find the pivot
    let (low, _high) = select_floyd_rivest(sample, index, is_less, rng);
    low
}

fn prepare_partition_3<T: Ord, F>(
    data: &mut [T],
    index: usize,
    is_less: F,
    rng: &mut PCGRng,
) -> (usize, usize)
where
    F: Fn(&T, &T) -> bool + Copy,
{
    // Take a random sample from the data for pivot selection
    let len = data.len();
    let (count, p, q) = sample_parameters(index, len);
    let sample = sample(data, count, rng);

    // Find the pivots
    let (q_low, q_high) = select_floyd_rivest(sample, q, is_less, rng);

    let (p_high, q_low) = if p < q_low {
        // The low pivot must be less than the high pivot
        let (_, p_high) = select_floyd_rivest(&mut sample[..q_low], p, is_less, rng);
        (p_high, q_low)
    } else {
        // The low pivot is equal to the high pivot
        (q_low, q_low + 1)
    };

    // Move sample elements >= high pivot to the end of the slice
    unordered_swap(data, q_high + 1, len - 1, count - q_high - 1);

    // Move sample elements == high pivot before the elements just moved to the end of the slice
    unordered_swap(data, q_low, len - count + q_high, q_high - q_low + 1);

    // Return the position of the last element equal to the low pivot and the position of the
    // first element equal to the high pivot
    (p_high, len - count + q_low)
}

/// Rotates the elements at `a`, `b`, and `c` in the slice `data` such that the element at `a` is
/// moved to `b`, the element at `b` is moved to `c`, and the element at `c` is moved to `a`.
unsafe fn rotate_3<T>(data: &mut [T], a: usize, b: usize, c: usize) {
    let a = data.get_unchecked_mut(a) as *mut T;
    let b = data.get_unchecked_mut(b) as *mut T;
    let c = data.get_unchecked_mut(c) as *mut T;

    let tmp = ManuallyDrop::new(ptr::read(b));
    b.write(ptr::read(a));
    a.write(ptr::read(c));
    c.write(ManuallyDrop::into_inner(tmp));
}

/// Takes a `count` element random sample from the slice, placing it into the beginning of the
/// slice. Returns the sample as a slice.
fn sample<'a, T>(data: &'a mut [T], count: usize, rng: &mut PCGRng) -> &'a mut [T] {
    let len = data.len();
    assert!(count <= len);
    unsafe {
        let mut elem = Elem::new(data);
        for i in 0..count {
            let j = rng.bounded_usize(i, len);
            elem.select(i);
            elem.swap(j);
        }
    }
    &mut data[..count]
}

pub fn select_nth_unstable<T: Ord>(data: &mut [T], index: usize) -> &T {
    let mut rng = PCGRng::new(0);
    if data.len() < CUT {
        select_nth_small(data, index, T::lt, &mut rng);
    } else {
        select_floyd_rivest(data, index, T::lt, &mut rng);
    }
    &data[index]
}

fn sort_2<T: Ord>(data: &mut [T], a: usize, b: usize) -> bool {
    debug_assert!(a != b);
    debug_assert!(a < data.len());
    debug_assert!(b < data.len());

    unsafe {
        let a = data.get_unchecked_mut(a) as *mut T;
        let b = data.get_unchecked_mut(b) as *mut T;
        let min = (&*a).min(&*b) as *const T;
        let swap = min == b;
        let tmp = ManuallyDrop::new(ptr::read(min));
        *b = ptr::read((&*a).max(&*b) as *const T);
        *a = ManuallyDrop::into_inner(tmp);
        swap
    }
}

fn sort_3<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize) -> usize {
    sort_2(data, a, b);
    if sort_2(data, b, c) {
        sort_2(data, a, b);
    }
    1
}

fn sort_4<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize, d: usize) -> usize {
    sort_2(data, a, b);
    sort_2(data, c, d);
    if sort_2(data, b, c) {
        sort_2(data, a, b);
    }
    if sort_2(data, c, d) {
        sort_2(data, b, c);
    }
    1
}

fn sort_5<T: Ord>(data: &mut [T], a: usize, b: usize, c: usize, d: usize, e: usize) {
    sort_2(data, a, d);
    sort_2(data, b, e);
    sort_2(data, a, c);
    sort_2(data, b, d);
    sort_2(data, a, b);
    sort_2(data, c, e);
    sort_2(data, b, c);
    sort_2(data, d, e);
    sort_2(data, c, d);
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
    let mut elem = Elem::new(inner);
    unsafe {
        elem.select(left);
        elem.move_to(right);
        for _ in 1..count {
            left += 1;
            elem.move_to(left);
            right -= 1;
            elem.move_to(right);
        }
    }
}

/// Moves the elements at indices `p` and `q` to the beginning and end of the slice so that
/// `data[p] <= data[q]`. Then returns the pivots and the interior of the slice as a triple
/// `low, mid, high`.
fn read_pivots<T: Ord>(data: &mut [T], p: usize, q: usize) -> (&mut T, &mut [T], &mut T) {
    debug_assert!(data.len() > 1);
    debug_assert!(p != q);

    sort_2(data, p, q);
    data.swap(0, p);
    data.swap(data.len() - 1, q);
    let (head, tail) = data.split_at_mut(1);
    let (mid, tail) = tail.split_at_mut(tail.len() - 1);
    (&mut head[0], mid, &mut tail[0])
}

// Reorders the slice so that the element at `index` is at its sorted position. Returns the
// indices of the first and last elements equal to the element at `index`.
fn select_floyd_rivest<T: Ord, F>(
    data: &mut [T],
    index: usize,
    is_less: F,
    rng: &mut PCGRng,
) -> (usize, usize)
where
    F: Fn(&T, &T) -> bool + Copy,
{
    let (mut inner, mut inner_index) = (data, index);
    let (mut offset, mut delta) = (0, usize::MAX);
    let (u, v) = loop {
        let len = inner.len();
        if inner_index == 0 {
            break partition_at_first(inner, is_less);
        } else if inner_index == inner.len() - 1 {
            break partition_at_last(inner, is_less);
        } else if inner.len() < CUT {
            break select_nth_small(inner, inner_index, is_less, rng);
        } else if len / 3 < inner_index && inner_index < 2 * len / 3 {
            let p = prepare_partition_2(inner, inner_index, is_less, rng);
            let sub = &mut inner[p..];
            let u = partition_2_single_index(sub, 0, is_less);
            match p + u {
                u if inner_index < u => {
                    inner = &mut inner[..u];
                }
                u => {
                    inner = &mut inner[u..];
                    inner_index -= u;
                    offset += u;
                }
            }
        } else {
            let (p, q) = prepare_partition_3(inner, inner_index, is_less, rng);
            let sub = &mut inner[p..=q];
            let q = q - p;
            let (u, v) = if delta == 0 {
                partition_3_single_index(sub, 0, is_less)
            } else if inner_index < len / 2 {
                partition_3_dual_index_high(sub, 0, q, is_less)
            } else {
                partition_3_dual_index_low(sub, 0, q, is_less)
            };
            match (p + u, p + v) {
                (u, _v) if inner_index < u => {
                    inner = &mut inner[..u];
                }
                (u, v) if inner_index <= v => {
                    if inner[u] == inner[v] {
                        break (u, v);
                    } else if inner_index == u {
                        break (u, u);
                    } else if inner_index == v {
                        break (v, v);
                    } else {
                        inner = &mut inner[u..=v];
                        inner_index -= u;
                        offset += u;
                    }
                }
                (_u, v) => {
                    inner = &mut inner[v + 1..];
                    offset += v + 1;
                    inner_index -= v + 1;
                }
            }
        }
        delta = len - inner.len();
    };
    (u + offset, v + offset)
}

fn select_nth_small<T, F>(
    data: &mut [T],
    index: usize,
    is_less: F,
    rng: &mut PCGRng,
) -> (usize, usize)
where
    T: Ord,
    F: Fn(&T, &T) -> bool + Copy,
{
    let (mut d, mut i) = (data, index);
    let (mut offset, mut delta) = (0, usize::MAX);
    assert!(i < d.len());
    let (u, v) = loop {
        match (i, d.len()) {
            (0, _) => break partition_at_first(d, is_less),
            (i, len) if i == len - 1 => break partition_at_last(d, is_less),
            (_, 25..) => {
                let len = d.len();
                let sample = sample(d, 25, rng);
                for j in 0..5 {
                    median_5(sample, j, j + 5, j + 10, j + 15, j + 20);
                }
                sort_5(sample, 10, 11, 12, 13, 14);
                if delta == 0 {
                    match partition_3_single_index(d, 12, is_less) {
                        (u, _v) if i < u => {
                            d = &mut d[..u];
                        }
                        (u, v) if i <= v => break (u, v),
                        (_u, v) => {
                            d = &mut d[v..];
                            i -= v;
                            offset += v;
                        }
                    }
                } else {
                    match partition_3_dual_index_low(d, 11, 13, is_less) {
                        (u, _v) if i < u => {
                            d = &mut d[..u];
                        }
                        (u, v) if i <= v => {
                            if d[u] == d[v] {
                                break (u, v);
                            } else if i == u {
                                break (u, u);
                            } else if i == v {
                                break (v, v);
                            } else {
                                d = &mut d[u..=v];
                                i -= u;
                                offset += u;
                            }
                        }
                        (_u, v) => {
                            d = &mut d[v + 1..];
                            i -= v + 1;
                            offset += v + 1;
                        }
                    }
                }
                delta = len - d.len();
            }
            (_, 6..) => {
                median_5(d, 0, 1, 2, 3, 4);
                match partition_3_single_index(d, 2, is_less) {
                    (u, _v) if i < u => {
                        d = &mut d[..u];
                    }
                    (u, v) if i <= v => break (u, v),
                    (_u, v) => {
                        d = &mut d[v..];
                        i -= v;
                        offset += v;
                    }
                }
            }
            (_, 5) => {
                sort_5(d, 0, 1, 2, 3, 4);
                break (i, i);
            }
            (_, 4) => {
                sort_4(d, 0, 1, 2, 3);
                break (i, i);
            }
            (_, 3) => {
                sort_3(d, 0, 1, 2);
                break (i, i);
            }
            (_, 2) => {
                sort_2(d, 0, 1);
                break (i, i);
            }
            _ => break (i, i),
        }
    };
    (u + offset, v + offset)
}

fn partition_2_single_index<T: Ord>(
    data: &mut [T],
    p: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> usize {
    debug_assert!(!data.is_empty());
    data.swap(0, p);
    let (head, tail) = data.split_at_mut(1);
    let pivot = &mut head[0];
    let u = partition_2_single_pivot(tail, pivot, is_less);
    data.swap(0, u);
    u
}

/// Partitions the slice into three parts using the element at index `p` as the pivot value. Returns
/// the indices of the first and last elements of the middle part, i.e. the elements equal to the
/// pivot.
///
/// The resulting partition is:
/// ```text
/// ┌─────────┬──────────┬─────────┐
/// │ < pivot │ == pivot │ > pivot │
/// └─────────┴──────────┴─────────┘
///            u        v
/// ```
///
/// Panics if the slice is empty or if `p` is out of bounds.
fn partition_3_single_index<T: Ord>(
    data: &mut [T],
    p: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    debug_assert!(!data.is_empty());
    data.swap(0, p);
    let (head, tail) = data.split_at_mut(1);
    let pivot = &mut head[0];
    let (u, v) = partition_3_single_pivot(tail, pivot, is_less);
    data.swap(0, u);
    (u, v)
}

/// Partitions the slice into three parts using the elements at indices `p` and `q` as the pivot
/// values. Returns the indices of the first and last elements of the middle part, i.e. the elements
/// that satisfy `low <= x <= high´, where `low` and `high` are the pivots.
///
/// The resulting partition is:
/// ```text
/// ┌─────────┬───────────────────┬─────────┐
/// │ < low   │ low <= .. <= high │ > high  │
/// └─────────┴───────────────────┴─────────┘
///            u                 v
/// ```
///
/// This variant of the algorithm tests the elements first against the lower pivot and conditionally
/// against the higher pivot. If most elements are expected to be less than the lower pivot, this
/// is faster than the high variant.
///
/// Panics if the slice has less than two elements, if `p` or `q` are out of bounds or if `p == q`.
fn partition_3_dual_index_low<T: Ord>(
    data: &mut [T],
    p: usize,
    q: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    let (low, mid, high) = read_pivots(data, p, q);
    let (u, v) = partition_3_dual_pivot_low(mid, low, high, is_less);
    data.swap(0, u);
    data.swap(v + 1, data.len() - 1);
    (u, v + 1)
}

/// Partitions the slice into three parts using the elements at indices `p` and `q` as the pivot
/// values. Returns the indices of the first and last elements of the middle part, i.e. the elements
/// that satisfy `low <= x <= high´, where `low` and `high` are the pivots.
///
/// The resulting partition is:
/// ```text
/// ┌─────────┬───────────────────┬─────────┐
/// │ < low   │ low <= .. <= high │ > high  │
/// └─────────┴───────────────────┴─────────┘
///            u                 v
/// ```
/// This variant of the algorithm tests the elements first against the higher pivot and
/// conditionally against the lower pivot. If most elements are expected to be greater than
/// the higher pivot, this is faster than the low variant.
///
/// Panics if the slice has less than two elements, if `p` or `q` are out of bounds or if `p == q`.
fn partition_3_dual_index_high<T: Ord>(
    data: &mut [T],
    p: usize,
    q: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    let (low, mid, high) = read_pivots(data, p, q);
    let (u, v) = partition_3_dual_pivot_high(mid, low, high, is_less);
    data.swap(0, u);
    data.swap(v + 1, data.len() - 1);
    (u, v + 1)
}

fn partition_2_single_pivot<T: Ord>(
    data: &mut [T],
    pivot: &mut T,
    is_less: impl Fn(&T, &T) -> bool,
) -> usize {
    unsafe {
        let (mut l, mut r) = (0, data.len() - 1);
        while l < r && is_less(pivot, data.get_unchecked(r)) {
            r -= 1;
        }
        while l < r && is_less(data.get_unchecked(l), pivot) {
            l += 1;
        }

        let data = &mut data[l..=r];
        let n = data.len();
        let (mut i, mut k) = (0, 0);

        let pivot = Elem::from_mut(pivot);
        let mut tmp = Elem::new(data);
        let mut offsets = Block::new();
        let mut num_lt: u8 = 0;
        let mut h: u8 = 0;


        while k < n {
            let size = (n - k).min(BLOCK) as u8;

            //                      | block |
            // ┌─────────┬──────────┬─────────────┐
            // │ < pivot │ >= pivot │   ? .. ?    │
            // └─────────┴──────────┴─────────────┘
            //            i          k

            // Scan the block beginning at k and store the offsets to elements <= pivot.
            while h < size {
                offsets.write(num_lt, h);
                let elem = tmp.get(k + h as usize);
                num_lt += is_less(elem, pivot.element()) as u8;
                h += 1;
            }
            h = 0;

            // Swap each element < pivot with the first element >= pivot.
            while h < num_lt {
                let m = k + offsets.get(h);
                tmp.select(m);
                tmp.swap(i + h as usize);
                h += 1;
            }
            h = 0;

            // Increment the indices and reset the counter
            k += size as usize;
            i += num_lt as usize;
            num_lt = 0;
        }
        l + i
    }
}

fn partition_3_single_pivot<T: Ord>(
    data: &mut [T],
    pivot: &mut T,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    unsafe {
        let (mut l, mut r) = (0, data.len() - 1);
        while l < r && is_less(pivot, data.get_unchecked(r)) {
            r -= 1;
        }
        while l < r && is_less(data.get_unchecked(l), pivot) {
            l += 1;
        }

        let data = &mut data[l..=r];
        let n = data.len();
        let (mut i, mut j, mut k) = (0, 0, 0);

        let pivot = Elem::from_mut(pivot);
        let mut tmp = Elem::new(data);
        let mut offsets = Block::new();
        let mut num_lt: u8 = 0;
        let mut num_le: u8 = 0;
        let mut h: u8 = 0;


        while k < n {
            let size = (n - k).min(BLOCK) as u8;

            //                                | block |
            // ┌─────────┬──────────┬─────────┬─────────────┐
            // │ < pivot │ == pivot │ > pivot │   ? .. ?    │
            // └─────────┴──────────┴─────────┴─────────────┘
            //            i          j         k

            // Scan the block beginning at k and store the offsets to elements <= pivot.
            while h < size {
                let elem = tmp.get(k + h as usize);
                offsets.write(num_le, h);
                num_le += !is_less(pivot.element(), elem) as u8;
                h += 1;
            }
            h = 0;

            // Swap each element <= pivot with the first element > pivot and store the offsets to
            // elements < pivot.
            while h < num_le {
                let m = k + offsets.get(h);
                tmp.select(m);
                offsets.write(num_lt, h);
                num_lt += is_less(tmp.element(), pivot.element()) as u8;
                tmp.swap(j + h as usize);
                h += 1;
            }
            h = 0;

            // Swap each element < pivot with the first element >= pivot.
            while h < num_lt {
                let m = j + offsets.get(h);
                tmp.select(m);
                tmp.swap(i + h as usize);
                h += 1;
            }
            h = 0;

            // Increment the indices
            k += size as usize;
            j += num_le as usize;
            i += num_lt as usize;

            // Reset the counters
            (num_le, num_lt) = (0, 0);

            // The first part contains elements x < pivot.
            // debug_assert!(data[..i].iter().all(|x| is_less(x, pivot.element())));

            // The middle part contains elements x == pivot.
            // debug_assert!(data[i..j].iter().all(|x| !is_less(x, pivot.element())));
            // debug_assert!(data[i..j].iter().all(|x| !is_less(pivot.element(), x)));

            // The last part contains elements x > pivot. Elements after k have not been scanned
            // yet and are unordered.
            // debug_assert!(data[j..k].iter().all(|x| is_less(pivot, x)));
        }
        (l + i, l + j)
    }
}

/// Partitions the slice into three parts, using `low` and `high` as pivots. Returns the indices
/// of the first elements of the second and third parts of the partition a tuple `(u, v)`.
///
/// The resulting partition is:
/// ```text
/// ┌───────┬───────────────────┬────────┐
/// │ < low │ low <= .. <= high │ > high │
/// └───────┴───────────────────┴────────┘
///          u                   v
/// ```
///
/// This variant of the algorithm tests the elements first against the lower pivot and conditionally
/// against the higher pivot. If most elements are expected to be less than the lower pivot, this
/// is faster than the high variant.
///
/// Panics if the indices are out of bounds or if `low > high`.
fn partition_3_dual_pivot_low<T: Ord>(
    data: &mut [T],
    low: &mut T,
    high: &mut T,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    assert!(low <= high);
    unsafe {
        let (mut l, mut r) = (0, data.len() - 1);
        while l < r && is_less(high, data.get_unchecked(r)) {
            r -= 1;
        }
        while l < r && is_less(data.get_unchecked(l), low) {
            l += 1;
        }

        let data = &mut data[l..=r];
        let n = data.len();
        let (mut i, mut j, mut k) = (n, n, n);

        let low = Elem::from_mut(low);
        let high = Elem::from_mut(high);
        let mut tmp = Elem::new(data);
        let mut offsets = Block::new();
        let mut num_gt_high: u8 = 0;
        let mut num_ge_low: u8 = 0;
        let mut h: u8 = 1;


        while k > 0 {
            let size = k.min(BLOCK) as u8;
            //     | block |
            // ┌───────────┬───────┬────────────────────┬────────┐
            // │  ? .. ?   │ low < │ low <= ... <= high │ > high │
            // └───────────┴───────┴────────────────────┴────────┘
            //            k k+1     i                    j

            // Scan the block ending at k and store the offsets to elements >= low.
            while h <= size {
                offsets.write(num_ge_low, h);
                let elem = tmp.get(k - h as usize);
                num_ge_low += !is_less(elem, low.element()) as u8;
                h += 1;
            }
            h = 0;

            // Swap each element >= low with the last element < low and store the offsets to
            // elements > high.
            while h < num_ge_low {
                let m = k - offsets.get(h);
                tmp.select(m);
                offsets.write(num_gt_high, h);
                num_gt_high += is_less(high.element(), tmp.element()) as u8;
                tmp.swap(i - 1 - h as usize);
                h += 1;
            }
            h = 0;

            // Swap each element > high with the last element <= high.
            while h < num_gt_high {
                let m = i - 1 - offsets.get(h);
                tmp.select(m);
                tmp.swap(j - 1 - h as usize);
                h += 1;
            }
            h = 1;

            // Increment the indices
            k -= size as usize;
            i -= num_ge_low as usize;
            j -= num_gt_high as usize;

            // Reset the counters
            (num_gt_high, num_ge_low) = (0, 0);

            // The first part contains elements x < low. The elements before k + 1 are unprocessed.
            // debug_assert!({
            //     if let Some(first) = data.get(k + 1..i) {
            //         first.iter().all(|x| is_less(x, low))
            //     } else {
            //         true
            //     }
            // });

            // The middle part contains elements low <= x <= high.
            // debug_assert!(if let Some(middle) = data.get(i..j) {
            //     middle.iter().all(|x| !is_less(x, low) && !is_less(high, x))
            // } else {
            //     true
            // });

            // The last part contains elements x > high.
            // debug_assert!(if let Some(last) = data.get(j..) {
            //     last.iter().all(|x| is_less(high, x))
            // } else {
            //     true
            // });
        }
        (l + i, l + j)
    }
}

/// Partitions the slice into three parts, using the `low` and `high` as pivots. Returns the indices
/// of the first elements of the second and third parts of the partition a tuple `(u, v)`.
///
/// The resulting partition is:
/// ```text
/// ┌───────┬───────────────────┬────────┐
/// │ < low │ low <= .. <= high │ > high │
/// └───────┴───────────────────┴────────┘
///          u                   v
/// ```
///
/// This variant of the algorithm tests the elements first against the higher pivot and
/// conditionally against the lower pivot. If most elements are expected to be greater than
/// the higher pivot, this is faster than the low variant.
///
/// Panics if the indices are out of bounds or if `low > high`.
fn partition_3_dual_pivot_high<T: Ord>(
    data: &mut [T],
    low: &mut T,
    high: &mut T,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    assert!(low <= high);

    unsafe {
        let (mut l, mut r) = (0, data.len() - 1);
        while l < r && is_less(high, data.get_unchecked(r)) {
            r -= 1;
        }
        while l < r && is_less(data.get_unchecked(l), low) {
            l += 1;
        }

        let data = &mut data[l..=r];
        let n = data.len();
        let (mut i, mut j, mut k) = (0, 0, 0);

        let low = Elem::from_mut(low);
        let high = Elem::from_mut(high);
        let mut tmp = Elem::new(data);

        let mut offsets = Block::new();
        let mut num_lt_low = 0;
        let mut num_le_high = 0;
        let mut h: u8 = 0;


        while k < n {
            let size = (n - k).min(BLOCK) as u8;

            //                                       | block |
            // ┌───────┬────────────────────┬────────┬─────────────┐
            // │ < low │ low <= ... <= high │ > high │   ? .. ?    │
            // └───────┴────────────────────┴────────┴─────────────┘
            //          i                    j        k

            // Scan the block beginning at k and store the offsets to elements <= high.
            while h < size {
                offsets.write(num_le_high, h);
                let elem = tmp.get(k + h as usize);
                num_le_high += !is_less(high.element(), elem) as u8;
                h += 1;
            }
            h = 0;

            // Swap each element <= high with the first element > high and store the offsets to
            // elements < low.

            while h < num_le_high {
                let m = k + offsets.get(h);
                tmp.select(m);

                offsets.write(num_lt_low, h);
                num_lt_low += is_less(tmp.element(), low.element()) as u8;
                tmp.swap(j + h as usize);
                h += 1;
            }
            h = 0;

            // Swap each element < low with the first element >= low.
            while h < num_lt_low {
                let m = j + offsets.get(h);
                tmp.select(m);
                tmp.swap(i + h as usize);
                h += 1;
            }
            h = 0;

            // Increment the indices
            k += size as usize;
            j += num_le_high as usize;
            i += num_lt_low as usize;

            // Reset the counters
            (num_lt_low, num_le_high) = (0, 0);

            // The first part contains elements x < low.
            // debug_assert!(data[..i].iter().all(|x| is_less(x, low.element())));

            // The middle part contains elements low <= x <= high.
            // debug_assert!(data[i..j].iter().all(|x| !is_less(x, low.element())));
            // debug_assert!(data[i..j].iter().all(|x| !is_less(high.element(), x)));

            // The last part contains elements x > high. Elements after k have not been scanned
            // yet and are unordered.
            // debug_assert!(data[j..k].iter().all(|x| is_less(high.element(), x)));
        }
        (l + i, l + j)
    }
}
