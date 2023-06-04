mod pcg_rng;
use core::{mem::MaybeUninit, ptr};
use pcg_rng::PCGRng;
use std::mem::ManuallyDrop;

#[cfg(test)]
mod tests;

const ALPHA: f64 = 0.25;
const BETA: f64 = 0.15;
const CUT: usize = 2000;

struct Block<const N: usize> {
    offsets: [MaybeUninit<u8>; N],
}

impl<const N: usize> Block<N> {
    const fn new() -> Self {
        Self {
            offsets: [MaybeUninit::uninit(); N],
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

struct Elem<T> {
    /// A pointer to the first element of the slice.
    origin: *mut T,

    /// A pointer to the position of the current element.
    ptr: Option<*mut T>,

    /// A temporary storage for the value of the current element.
    tmp: MaybeUninit<T>,
}

impl<T> Elem<T> {
    /// Returns a reference to the current element. Unsafe because the element may not be selected.
    unsafe fn element(&self) -> &T {
        debug_assert!(self.ptr.is_some());

        self.tmp.assume_init_ref()
    }

    #[inline]
    /// Returns a reference to the element at `index`. Unsafe because index must be in bounds.
    unsafe fn get(&self, index: usize) -> &T {
        &*self.origin.add(index)
    }

    const fn new(origin: *mut T) -> Self {
        Self {
            origin,
            ptr: None,
            tmp: MaybeUninit::uninit(),
        }
    }

    /// Selects the element at `index` as the current element. Unsafe because the index must be in
    /// bounds.
    unsafe fn select(&mut self, index: usize) {
        debug_assert!(self.ptr.is_none());
        let src = self.origin.add(index);
        self.ptr = Some(src);
        self.tmp.write(ptr::read(src));
    }

    #[inline]
    /// Sets the position of the current element to `index`. This also moves the position of the
    /// element at `index` to the previous position of the current element.
    ///
    /// Unsafe because index must be in bounds and the current element must be selected.
    unsafe fn set(&mut self, index: usize) {
        debug_assert!(self.ptr.is_some());
        let src = self.origin.add(index);
        self.ptr.unwrap_unchecked().write(src.read());
        self.ptr = Some(src);
    }

    #[inline]
    /// Swaps the current element with the element at `index`. Unsafe because index must be in
    /// bounds and the current element must be selected.
    unsafe fn swap(&mut self, index: usize) {
        debug_assert!(self.ptr.is_some());
        let dst = self.origin.add(index);
        self.ptr.unwrap_unchecked().write(dst.read());
        dst.write(self.tmp.assume_init_read());
        self.ptr = None;
    }
}

impl<T> Drop for Elem<T> {
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

macro_rules! unroll {
    ($body:stmt) => {
        $body
        $body
        $body
        $body
        $body
        $body
        $body
        $body
    };
}

fn median_5<T, F>(
    data: &mut [T],
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    sort_2(data, a, b, is_less);
    sort_2(data, c, d, is_less);
    sort_2(data, a, c, is_less);
    sort_2(data, b, d, is_less);
    sort_2(data, c, e, is_less);
    sort_2(data, b, c, is_less);
    sort_2(data, c, e, is_less);
    c
}

/// Puts the minimum elements at the beginning of the slice and returns the indices of the first and
/// last elements equal to the minimum.
fn select_min<T, F>(data: &mut [T], is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // The index of the last element that is equal to the minimum element.
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

/// Puts the maximum elements at the end of the slice and returns the indices of the first and
/// last elements equal to the maximum.
fn select_max<T, F>(data: &mut [T], is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
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

fn prepare<T, F>(data: &mut [T], index: usize, is_less: &mut F, rng: &mut PCGRng) -> usize
where
    F: FnMut(&T, &T) -> bool,
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
    let (low, _high) = floyd_rivest_select(sample, index, is_less, rng);
    low
}

fn prepare_dual<T, F>(
    data: &mut [T],
    index: usize,
    is_less: &mut F,
    rng: &mut PCGRng,
) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // Take a random sample from the data for pivot selection
    let len = data.len();
    let (count, p, q) = sample_parameters(index, len);
    let sample = sample(data, count, rng);

    // Find the pivots
    let (q_low, q_high) = floyd_rivest_select(sample, q, is_less, rng);

    let (p_high, q_low) = if p < q_low {
        // The lower pivot must be less than the higher pivot
        let (_, p_high) = floyd_rivest_select(&mut sample[..q_low], p, is_less, rng);
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

/// Takes a `count` element random sample from the slice, placing it into the beginning of the
/// slice. Returns the sample as a slice.
fn sample<'a, T>(data: &'a mut [T], count: usize, rng: &mut PCGRng) -> &'a mut [T] {
    let len = data.len();
    assert!(count <= len);
    unsafe {
        let mut elem = Elem::new(data.as_mut_ptr());
        elem.select(0);
        for i in 1..count {
            let j = rng.bounded_usize(i, len);
            elem.set(j);
            elem.set(i);
        }
        let j = rng.bounded_usize(0, len);
        elem.set(j);
    }
    &mut data[..count]
}

pub fn select_nth_unstable<T: Ord>(data: &mut [T], index: usize) -> &T {
    let mut rng = PCGRng::new(0);
    if data.len() < CUT {
        quickselect(data, index, &mut T::lt, rng.as_mut());
    } else {
        floyd_rivest_select(data, index, &mut T::lt, rng.as_mut());
    }
    &data[index]
}

#[inline]
fn sort_2<T, F>(data: &mut [T], a: usize, b: usize, is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(a != b);
    debug_assert!(a < data.len());
    debug_assert!(b < data.len());

    unsafe {
        let a = data.get_unchecked_mut(a) as *mut T;
        let b = data.get_unchecked_mut(b);
        let swap = is_less(b, &*a);
        if swap {
            ptr::swap(a, b);
        }
        swap
    }
}

fn sort_3<T, F>(data: &mut [T], a: usize, b: usize, c: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    sort_2(data, a, c, is_less);
    sort_2(data, a, b, is_less);
    sort_2(data, b, c, is_less);
}

fn sort_4<T, F>(data: &mut [T], a: usize, b: usize, c: usize, d: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    sort_2(data, a, c, is_less);
    sort_2(data, b, d, is_less);
    sort_2(data, a, b, is_less);
    sort_2(data, c, d, is_less);
    sort_2(data, b, c, is_less);
}

#[inline]
fn sort_5<T, F>(data: &mut [T], a: usize, b: usize, c: usize, d: usize, e: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    sort_2(data, a, d, is_less);
    sort_2(data, b, e, is_less);
    sort_2(data, a, c, is_less);
    sort_2(data, b, d, is_less);
    sort_2(data, a, b, is_less);
    sort_2(data, c, e, is_less);
    sort_2(data, b, c, is_less);
    sort_2(data, d, e, is_less);
    sort_2(data, c, d, is_less);
}

/// Performs an unordered swap of the first `count` elements starting from `left` with the last
/// `count` elements ending at and including`right`.
fn unordered_swap<T>(data: &mut [T], mut left: usize, mut right: usize, count: usize) {
    if count == 0 {
        return;
    }
    debug_assert!(left + count <= right);
    debug_assert!(right <= data.len());
    let inner = data[left..=right].as_mut();
    (left, right) = (0, inner.len() - 1);
    let mut elem = Elem::new(inner.as_mut_ptr());
    unsafe {
        elem.select(left);
        elem.set(right);
        for _ in 1..count {
            left += 1;
            elem.set(left);
            right -= 1;
            elem.set(right);
        }
    }
}
// Reorders the slice so that the element at `index` is at its sorted position. Returns the
// indices of the first and last elements equal to the element at `index`.
fn floyd_rivest_select<T, F>(
    mut data: &mut [T],
    mut index: usize,
    is_less: &mut F,
    rng: &mut PCGRng,
) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let (mut offset, mut delta) = (0, usize::MAX);
    let (mut u, mut v);
    loop {
        let len = data.len();
        // When selecting the minimum or maximum, partioning is not necessary
        if index == 0 {
            (u, v) = select_min(data, is_less);
            break;
        } else if index == data.len() - 1 {
            (u, v) = select_max(data, is_less);
            break;
        }
        // When the slice is small enough, use quickselect
        else if data.len() < CUT {
            (u, v) = quickselect(data, index, is_less, rng);
            break;
        } else if delta > 8 {
            let (p, q) = prepare_dual(data, index, is_less, rng);

            // If p = 0 or q = count - 1, dual-pivot parititioning is not necessary, use normal
            // Hoare partitioning instead
            let (l, m) = if p == 0 {
                partition_at_index(data[p..=q].as_mut(), q, is_less)
            } else if q == len - 1 {
                partition_at_index(data[p..=q].as_mut(), 0, is_less)
            } else {
                partition_at_index_dual(data[p..=q].as_mut(), (0, q - p), is_less)
            };
            (u, v) = (l + p, m + p);
        } else {
            let p = prepare(data, index, is_less, rng);
            let (l, m) = partition_at_index_eq(&mut data[p..], 0, is_less);
            (u, v) = (l + p, m + p);
        }

        // Test if the pivot is at its sorted position and if not, recurse on the appropriate
        // partition
        if index < u {
            data = &mut data[..u];
        } else if index > v {
            data = &mut data[v + 1..];
            offset += v + 1;
            index -= v + 1;
        } else if !is_less(&data[u], &data[v]) {
            break;
        } else {
            data = &mut data[u..=v];
            index -= u;
            offset += u;
        }
        delta = len - data.len();
    }
    (u + offset, v + offset)
}

fn quickselect<T, F>(
    mut data: &mut [T],
    mut index: usize,
    is_less: &mut F,
    rng: &mut PCGRng,
) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let (mut offset, mut delta) = (0, usize::MAX);
    assert!(index < data.len());
    let (mut u, mut v);
    loop {
        let len = data.len();
        match (index, len) {
            (0, _) => {
                (u, v) = select_min(data, is_less);
            }
            (i, len) if i == len - 1 => {
                (u, v) = select_max(data, is_less);
            }
            (_, 25..) => {
                let sample = sample(data, 25, rng);
                for j in 0..5 {
                    sort_5(sample, j, j + 5, j + 10, j + 15, j + 20, is_less);
                }
                let p = 5 * ((5 * index) / len);
                sort_5(sample, p, p + 1, p + 2, p + 3, p + 4, is_less);
                if delta > 8 {
                    (u, v) = partition_at_index(data, p + 2, is_less);
                } else {
                    (u, v) = partition_at_index_dual(data, (p + 1, p + 3), is_less);
                }
            }
            (_, 6..) => {
                median_5(data, 0, 1, 2, 3, 4, is_less);
                (u, v) = partition_at_index_eq(data, 2, is_less);
            }
            (_, 5) => {
                sort_5(data, 0, 1, 2, 3, 4, is_less);
                (u, v) = (index, index);
            }
            (_, 4) => {
                sort_4(data, 0, 1, 2, 3, is_less);
                (u, v) = (index, index);
            }
            (_, 3) => {
                sort_3(data, 0, 1, 2, is_less);
                (u, v) = (index, index);
            }
            (_, 2) => {
                sort_2(data, 0, 1, is_less);
                (u, v) = (index, index);
            }
            _ => {
                (u, v) = (index, index);
            }
        }
        if index < u {
            data = &mut data[..u];
        } else if index > v {
            data = &mut data[v + 1..];
            offset += v + 1;
            index -= v + 1;
        } else if !is_less(&data[u], &data[v]) {
            break;
        } else {
            data = &mut data[u..=v];
            index -= u;
            offset += u;
        }
        delta = len - data.len();
    }
    (u + offset, v + offset)
}

/// Partitions the slice into two parts using the element at `p` as the pivot. Returns the index
/// of the pivot after partitioning.
///
/// Using `u` to denote the index returned by the function, the resulting partitioning is:
/// ```text
/// ┌───────────┬────────────┐
/// │ < data[u] │ >= data[u] │
/// └───────────┴────────────┘
///              u        
/// ```
///
/// Panics if `p` is out of bounds.
fn hoare_dyad<T, F>(data: &mut [T], p: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    data.swap(0, p);
    let (head, tail) = data.split_first_mut().unwrap();
    let tmp = unsafe { ManuallyDrop::new(ptr::read(head)) };
    let pivot = &*tmp;

    // Find the first pair of elements that are out of order.
    let (mut l, mut r, e);
    unsafe {
        l = tail.as_mut_ptr();
        e = l.add(tail.len());
        r = e.sub(1);
        while l < r && is_less(&*l, pivot) {
            l = l.add(1)
        }
        while l < r && !is_less(&*r, pivot) {
            r = r.sub(1);
        }
    }

    // let mut tmp = Elem::new(tail.as_mut_ptr());
    let mut h: u8;
    let mut num_ge: u8 = 0;
    let mut num_lt: u8 = 0;
    let mut start_ge: u8 = 0;
    let mut start_lt: u8 = 0;

    const BLOCK: usize = 128;
    let mut offsets_ge: [MaybeUninit<u8>; BLOCK] = [MaybeUninit::uninit(); BLOCK];
    let mut offsets_lt: [MaybeUninit<u8>; BLOCK] = [MaybeUninit::uninit(); BLOCK];

    fn width<U>(l: *mut U, r: *mut U) -> usize {
        unsafe { r.offset_from(l) as usize + 1 }
    }

    // Repeat while the blocks don't overlap.
    while width(l, r) > 2 * BLOCK {
        // If the block is empty, scan the next elements.
        if num_ge == 0 {
            start_ge = 0;
            h = 0;
            // Store the offsets of the elements >= pivot.
            while h < BLOCK as u8 {
                unroll!(unsafe {
                    offsets_ge.get_unchecked_mut(num_ge as usize).write(h);
                    let elem = &*l.add(h as usize);
                    num_ge += !is_less(elem, pivot) as u8;
                    h += 1;
                });
            }
        }
        if num_lt == 0 {
            start_lt = 0;
            h = 0;
            // Store the offsets of elements < pivot.
            while h < BLOCK as u8 {
                unroll!(unsafe {
                    offsets_lt.get_unchecked_mut(num_lt as usize).write(h);
                    let elem = &*r.sub(h as usize);
                    num_lt += is_less(elem, pivot) as u8;
                    h += 1;
                });
            }
        }

        let num = num_ge.min(num_lt);
        if num > 0 {
            // Swap the out-of-order pairs.
            unsafe {
                let mut m = offsets_ge.get_unchecked_mut(start_ge as usize).as_mut_ptr();
                let mut n = offsets_lt.get_unchecked_mut(start_lt as usize).as_mut_ptr();
                let tmp = ptr::read(l.add(*m as usize));
                ptr::copy_nonoverlapping(r.sub(*n as usize), l.add(*m as usize), 1);
                h = 1;
                while h < num {
                    m = m.add(1);
                    ptr::copy_nonoverlapping(l.add(*m as usize), r.sub(*n as usize), 1);
                    n = n.add(1);
                    ptr::copy_nonoverlapping(r.sub(*n as usize), l.add(*m as usize), 1);
                    h += 1;
                }
                ptr::copy_nonoverlapping(&tmp, r.sub(*n as usize), 1);

                // let mut m = l.add(offsets_ge.get(start_ge));
                // let mut n = r.sub(offsets_lt.get(start_lt));
                // let tmp = ptr::read(m);
                // m.copy_from_nonoverlapping(n, 1);
                // h = 1;
                // while h < num {
                //     m = l.add(offsets_ge.get(start_ge + h));
                //     n.copy_from_nonoverlapping(m, 1);
                //     n = r.sub(offsets_lt.get(start_lt + h));
                //     m.copy_from_nonoverlapping(n, 1);
                //     h += 1;
                // }
                // n.copy_from_nonoverlapping(&tmp, 1);
            }
            num_ge -= num;
            num_lt -= num;
            start_ge += num;
            start_lt += num;
        }

        unsafe {
            // If the left block is finished, move it to the right by BLOCK elements.
            l = l.add(BLOCK * (num_ge == 0) as usize);
            // If the right block is finished, move it to the left by BLOCK elements.
            r = r.sub(BLOCK * (num_lt == 0) as usize);
        }
    }

    // Process the remaining elements.
    unsafe {
        l = l.add((start_ge as usize) * (num_ge > 0) as usize);
        r = r.sub((start_lt as usize) * (num_lt > 0) as usize);
        loop {
            while l < r && is_less(&*l, pivot) {
                l = l.add(1);
            }
            while l < r && !is_less(&*r, pivot) {
                r = r.sub(1);
            }
            if l < r {
                l.swap(r);
                l = l.add(1);
                r = r.sub(1);
            } else {
                break;
            }
        }
        l = l.sub(1);
        while l.add(1) < e && is_less(&*l.add(1), pivot) {
            l = l.add(1);
        }
        let u = l.offset_from(head) as usize;
        ptr::swap(head, l);
        (u, u)
    }
}

/// Partitions the slice into three parts using the elements at indices `p` and `q` as the pivot
/// values. Returns the indices of the first and last elements of between or equal to the pivot
/// values.
///
/// Using `(u, v)` to denote the indices returned by the function, the slice is partitioned as
/// follows:
/// ```text
/// ┌───────────┬──────────────────────────┬───────────┐
/// │ < data[u] │ data[u] <= .. <= data[v] │ > data[v] │
/// └───────────┴──────────────────────────┴───────────┘
///              u                        v
/// ```
///
/// Panics if `p` or `q` are out of bounds.
fn hoare_trinity<T, F>(data: &mut [T], p: usize, q: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    const BLOCK: usize = 128;

    assert!(p < data.len() && q < data.len());
    sort_2(data, p, q, is_less);
    data.swap(0, p);
    data.swap(q, data.len() - 1);

    // Copy the pivots to the stack.
    let ptr = unsafe { data.get_unchecked_mut(0) } as *mut T;
    let mut low = Elem::new(ptr);
    let mut high = Elem::new(ptr);
    unsafe {
        low.select(0);
        high.select(data.len() - 1);
    }

    let (_, tail) = data.split_first_mut().unwrap();
    let (_, middle) = tail.split_last_mut().unwrap();

    // Find the first pair of elements that are out of order.
    let (mut l, mut r) = (0, middle.len() - 1);
    unsafe {
        while l < r && is_less(middle.get_unchecked(l), low.element()) {
            l += 1;
        }
        while l < r && is_less(high.element(), middle.get_unchecked(r)) {
            r -= 1;
        }
    }

    let n = middle.len();
    let (mut i, mut j, mut p, mut q) = (l, r, 0, n - 1);
    let mut tmp = Elem::new(middle.as_mut_ptr());

    let mut h: u8;

    // The block lenghts
    let mut n_lr: u8 = 0;
    let mut n_rl: u8 = 0;

    // The indices of first unprocessed element in each block.
    let mut s_lr: u8 = 0;
    let mut s_rl: u8 = 0;

    // The offset blocks.
    let mut offsets_lr = Block::<BLOCK>::new();
    let mut offsets_rl = Block::<BLOCK>::new();

    while j - i + 1 > 2 * BLOCK {
        if n_lr == 0 {
            s_lr = 0;
            h = 0;
            // Collect the offsets to elements >= low
            while h < BLOCK as u8 {
                unroll!(unsafe {
                    offsets_lr.write(n_lr, h);
                    let elem = tmp.get(i + h as usize);
                    n_lr += !is_less(elem, low.element()) as u8;
                    h += 1;
                });
            }
        }
        if n_rl == 0 {
            s_rl = 0;
            h = 0;
            // Collect the offsets to elements <= high
            while h < BLOCK as u8 {
                unroll!(unsafe {
                    offsets_rl.write(n_rl, h);
                    let elem = tmp.get(j - h as usize);
                    n_rl += !is_less(high.element(), elem) as u8;
                    h += 1;
                });
            }
        }

        // We use the beginning and the end of the slice as a temporary store for the elements
        // that belong to the middle:
        //  ┌─────────────────┬───────┬─────┬────────┬──────────────────┐
        //  │low <= .. < high │ < low │  ?  │ > high │ low < .. <= high │
        //  └─────────────────┴───────┴─────┴────────┴──────────────────┘
        //   0                 p       i   j        q                    n

        let num = n_lr.min(n_rl);
        if num > 0 {
            // Swap the out-of-order pairs and store the indices of the elements that belong to
            // the middle.
            h = 0;
            while h < num {
                unsafe {
                    let f = offsets_lr.get(s_lr + h);
                    let g = offsets_rl.get(s_rl + h);

                    let k = i + f;
                    let m = j - g;

                    // offsets_rm.write(n_rm, g as u8);
                    // offsets_lm.write(n_lm, f as u8);

                    tmp.select(k);
                    let swap_rm = is_less(tmp.element(), high.element());
                    // n_rm += swap_rm as u8;
                    tmp.swap(m);
                    let swap_lm = is_less(low.element(), tmp.get(k));
                    // n_lm += swap_lm as u8;

                    if swap_rm {
                        tmp.select(m);
                        tmp.swap(q);
                        q -= 1;
                    }

                    if swap_lm {
                        tmp.select(k);
                        tmp.swap(p);
                        p += 1;
                    }
                }
                h += 1;
            }

            n_lr -= num;
            n_rl -= num;
            s_lr += num;
            s_rl += num;
        }

        i += BLOCK * (n_lr == 0) as usize;
        j -= BLOCK * (n_rl == 0) as usize;
    }

    // Process the remaining elements
    i += (s_lr as usize) * (n_lr > 0) as usize;
    j -= (s_rl as usize) * (n_rl > 0) as usize;
    unsafe {
        loop {
            while i < j && is_less(tmp.get(i), low.element()) {
                i += 1;
            }
            while i < j && is_less(high.element(), tmp.get(j)) {
                j -= 1;
            }
            if i < j {
                tmp.select(i);
                tmp.swap(j);
                if !is_less(tmp.get(i), low.element()) {
                    tmp.select(i);
                    tmp.swap(p);
                    p += 1;
                }
                if !is_less(high.element(), tmp.get(j)) {
                    tmp.select(j);
                    tmp.swap(q);
                    q -= 1;
                }
                i += 1;
                j -= 1;
            } else {
                break;
            }
        }
        while i < n && is_less(tmp.get(i), low.element()) {
            i += 1;
        }
        while j > 0 && is_less(high.element(), tmp.get(j)) {
            j -= 1;
        }
        if i == j {
            tmp.select(i);
            if !is_less(tmp.element(), low.element()) && !is_less(high.element(), tmp.element()) {
                tmp.swap(p);
                p += 1;
                i += 1;
            }
        }
    }

    //  Move the temporary parts to the middle:
    //  ┌─────────────────┬───────┬────────┬──────────────────┐
    //  │low <= .. < high │ < low │ > high │ low < .. <= high │
    //  └─────────────────┴───────┴────────┴──────────────────┘
    //   0                 p     j i      q                    n

    let s_lm = p.min(i - p);
    let s_rm = (n - q - 1).min(q + 1 - i);

    unordered_swap(middle, 0, j, s_lm);
    unordered_swap(middle, i, n - 1, s_rm);

    // let (left, right) = middle.split_at_mut(i);

    // let (left_a, tail) = left.split_at_mut(s_lm);
    // let (_, left_b) = tail.split_at_mut(tail.len() - s_lm);
    // left_a.swap_with_slice(left_b);

    // let (right_a, tail) = right.split_at_mut(s_rm);
    // let (_, right_b) = tail.split_at_mut(tail.len() - s_rm);
    // right_a.swap_with_slice(right_b);

    let u = i - p;
    let v = i + n - q;
    unsafe {
        low.set(u);
        high.set(v);
    }
    (u, v)
}

/// Partitions the slice into three parts using the element at index `p` as the pivot. Returns
/// the indices of the first and last elements of equal to the pivot.
///
/// Using `(u, v)` to denote the indices returned by the function, the slice is partitioned as
/// follows:
/// ```text
/// ┌───────────┬────────────┬───────────┐
/// │ < data[u] │ == data[u] │ > data[u] │
/// └───────────┴────────────┴───────────┘
///              u          v
/// ```
///
/// Panics if the slice is empty or if `p` is out of bounds.
fn lomuto_trinity<T, F>(data: &mut [T], p: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    data.swap(0, p);
    let (head, tail) = data.split_first_mut().unwrap();
    let tmp = unsafe { ManuallyDrop::new(ptr::read(head)) };
    let pivot = &*tmp;

    let (mut l, mut r, e);
    unsafe {
        l = tail.as_mut_ptr();
        e = l.add(tail.len());
        r = e.sub(1);
        while l < r && is_less(&*l, pivot) {
            l = l.add(1)
        }
        while l < r && !is_less(&*r, pivot) {
            r = r.sub(1);
        }
    }

    let (mut i, mut j, mut k) = (l, l, l);

    const BLOCK: usize = 128;
    let mut offsets: [MaybeUninit<u8>; BLOCK] = [MaybeUninit::uninit(); BLOCK];

    let mut num_lt: u8 = 0;
    let mut num_le: u8 = 0;
    let mut h: u8 = 0;

    while k < e {
        let size = unsafe { (e.offset_from(k) as usize).min(BLOCK) as u8 };

        //                                | block |
        // ┌─────────┬──────────┬─────────┬─────────────┐
        // │ < pivot │ == pivot │ > pivot │   ? .. ?    │
        // └─────────┴──────────┴─────────┴─────────────┘
        //            i          j         k

        // Scan the block beginning at k and store the offsets to elements <= pivot.
        while h < size {
            unsafe {
                let elem = &*k.add(h as usize);
                offsets.get_unchecked_mut(num_le as usize).write(h);
                num_le += !is_less(pivot, elem) as u8;
            }
            h += 1;
        }
        h = 0;

        // Swap each element <= pivot with the first element > pivot and store the offsets to
        // elements < pivot.
        unsafe {
            let mut m = offsets.get_unchecked_mut(0).as_mut_ptr();
            while h < num_le {
                let elem = k.add(*m as usize);
                offsets.get_unchecked_mut(num_lt as usize).write(h);
                num_lt += is_less(&*elem, pivot) as u8;
                let other = j.add(h as usize);
                ptr::swap(elem, other);
                m = m.add(1);
                h += 1;
            }
        }
        h = 0;

        // Swap each element < pivot with the first element >= pivot.
        unsafe {
            let mut m = offsets.get_unchecked_mut(0).as_mut_ptr();
            while h < num_lt {
                let elem = j.add(*m as usize);
                let other = i.add(h as usize);
                ptr::swap(elem, other);
                m = m.add(1);
                h += 1;
            }
        }
        h = 0;

        // Increment the indices
        unsafe {
            k = k.add(size as usize);
            j = j.add(num_le as usize);
            i = i.add(num_lt as usize);
        }

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
    unsafe {
        let u = i.offset_from(head) as usize - 1;
        let v = j.offset_from(head) as usize - 1;
        i = i.sub(1);
        ptr::swap(head, i);
        (u, v)
    }
}

/// Partitions `data` into two parts using the element at `index` as the pivot. Returns `(u, u)`,
/// where `u` is the number of elements less than the pivot, and the index of the pivot after
/// partitioning.
///
/// The resulting partitioning is as follows:
///
/// ```text
/// ┌───────────┬────────────┐
/// │ < data[u] │ >= data[u] │
/// └───────────┴────────────┘
///              u        
/// ```
///
/// Panics if `index` is out of bounds.
fn partition_at_index<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    data.swap(0, index);
    let (head, tail) = data.split_first_mut().unwrap();
    let tmp = unsafe { ManuallyDrop::new(ptr::read(head)) };
    let pivot = &*tmp;
    let u = partition_in_blocks(tail, pivot, is_less);
    data.swap(0, u);
    (u, u)
}

/// Partitions `data` into three parts using the element at `index` as the pivot. Returns `(u, v)`,
/// where `u` is the number of elements less than the pivot, and `v` is the number of elements less
/// than or equal to the pivot.
///
/// The resulting partitioning is as follows:
///
/// ```text
/// ┌───────────┬───────────────────────────┬────────────┐
/// │ < data[u] │ data[u] == ... == data[v] │ >= data[u] │
/// └───────────┴───────────────────────────┴────────────┘
///              u                         v 
/// ```
///
/// Panics if `index` is out of bounds.
fn partition_at_index_eq<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    data.swap(0, index);
    let (head, tail) = data.split_first_mut().unwrap();
    let tmp = unsafe { ManuallyDrop::new(ptr::read(head)) };
    let pivot = &*tmp;
    let (u,v) = partition_in_blocks_dual(tail, pivot, pivot, is_less);
    data.swap(0, u);
    (u, v)
}

/// Partitions `data` into three parts using the elements at `index.0` and `index.1` as the pivot
/// values. Returns `(u, v)` where `u` is the number of elements less than the lower pivot value,
/// and `v` is the number of elements less than or equal to the upper pivot value. The indices of
/// the pivot values after partitioning are `(u, v)`.
///  
/// ```text
/// ┌───────────┬──────────────────────────┬───────────┐
/// │ < data[u] │ data[u] <= .. <= data[v] │ > data[v] │
/// └───────────┴──────────────────────────┴───────────┘
///              u                        v
/// ```
///
/// Panics if `p` or `q` are out of bounds.
fn partition_at_index_dual<T, F>(
    data: &mut [T],
    index: (usize, usize),
    is_less: &mut F,
) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let (p, q) = index;
    let len = data.len();

    sort_2(data, p, q, is_less);
    data.swap(0, p);
    data.swap(q, len - 1);

    let (first, tail) = data.split_first_mut().unwrap();
    let (last, inner) = tail.split_last_mut().unwrap();
    let tmp_low = unsafe { ManuallyDrop::new(ptr::read(first)) };
    let tmp_high = unsafe { ManuallyDrop::new(ptr::read(last)) };

    let low = &*tmp_low;
    let high = &*tmp_high;

    let (u, v) = partition_in_blocks_dual(inner, low, high, is_less);
    data.swap(0, u);
    data.swap(v + 1, len - 1);
    (u, v + 1)
}

/// Partitions `v` into elements smaller than `pivot`, followed by elements greater than or equal
/// to `pivot`.
///
/// Returns the number of elements smaller than `pivot`.
///
/// Partitioning is performed block-by-block in order to minimize the cost of branching operations.
/// This idea is presented in the [BlockQuicksort][pdf] paper.
///
/// [pdf]: https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
fn partition_in_blocks<T, F>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    // Number of elements in a typical block.
    const BLOCK: usize = 128;

    // The partitioning algorithm repeats the following steps until completion:
    //
    // 1. Trace a block from the left side to identify elements greater than or equal to the pivot.
    // 2. Trace a block from the right side to identify elements smaller than the pivot.
    // 3. Exchange the identified elements between the left and right side.
    //
    // We keep the following variables for a block of elements:
    //
    // 1. `block` - Number of elements in the block.
    // 2. `start` - Start pointer into the `offsets` array.
    // 3. `end` - End pointer into the `offsets` array.
    // 4. `offsets - Indices of out-of-order elements within the block.

    // The current block on the left side (from `l` to `l.add(block_l)`).
    let mut l = v.as_mut_ptr();
    let mut block_l = BLOCK;
    let mut start_l: *mut u8 = ptr::null_mut();
    let mut end_l: *mut u8 = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())`
    // is always safe`
    let mut r = unsafe { l.add(v.len()) };
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        // FIXME: this should *likely* use `offset_from`, but more
        // investigation is needed (including running tests in miri).
        unsafe { r.offset_from(l) as usize }
    }

    loop {
        // We are done with partitioning block-by-block when `l` and `r` get very close. Then we do
        // some patch-up work in order to partition the remaining elements in between.
        let is_done = width(l, r) <= 2 * BLOCK;

        if is_done {
            // Number of remaining elements (still not compared to the pivot).
            let mut rem = width(l, r);
            if start_l < end_l || start_r < end_r {
                rem -= BLOCK;
            }

            // Adjust block sizes so that the left and right block don't overlap, but get perfectly
            // aligned to cover the whole remaining gap.
            if start_l < end_l {
                block_r = rem;
            } else if start_r < end_r {
                block_l = rem;
            } else {
                // There were the same number of elements to switch on both blocks during the last
                // iteration, so there are no remaining elements on either block. Cover the
                // remaining items with roughly equally-sized blocks.
                block_l = rem / 2;
                block_r = rem - block_l;
            }
            debug_assert!(block_l <= BLOCK && block_r <= BLOCK);
            debug_assert!(width(l, r) == block_l + block_r);
        }

        if start_l == end_l {
            // Trace `block_l` elements from the left side.
            start_l = offsets_l.as_mut_ptr().cast();
            end_l = start_l;
            let mut elem = l;

            for i in 0..block_l {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                // According to the conditions required by the function, we satisfy them
                // because:
                // 1. `offsets_l` is stack-allocated, and thus considered separate allocated object.
                // 2. The function `is_less` returns a `bool`. Casting a `bool` will
                //    never overflow `isize`.
                // 3. We have guaranteed that `block_l` will be `<= BLOCK`. Plus, `end_l` was
                //    initially set to the begin pointer of `offsets_` which was declared on the
                //    stack.Thus, we know that even in the worst case (all  invocations of `is_less`
                //    returns false) we will only be at most 1 byte pass the end.
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially the begin pointer to the slice which is always valid.
                unsafe {
                    // Branchless comparison.
                    *end_l = i as u8;
                    end_l = end_l.offset(!is_less(&*elem, pivot) as isize);
                    elem = elem.offset(1);
                }
            }
        }

        if start_r == end_r {
            // Trace `block_r` elements from the right side.
            start_r = offsets_r.as_mut_ptr().cast();
            end_r = start_r;
            let mut elem = r;

            for i in 0..block_r {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                // According to the conditions required by the function, we satisfy them
                // because:
                //
                // 1. `offsets_r` is stack-allocated, and thus considered separate allocated object.
                // 2. The function `is_less` returns a `bool`. Casting a `bool` will
                //    never overflow `isize`.
                // 3. We have guaranteed that `block_r` will be `<= BLOCK`. Plus, `end_r` was
                //    initially set to the begin pointer of `offsets_` which was declared on the
                //    stack. Thus, we know that even in the worst case (all invocations of `is_less`
                //    returns true) we will only be at most 1 byte pass the end.
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially `1 * sizeof(T)` past the end and we decrement it by `1 * sizeof(T)`
                // before accessing it.
                // Plus, `block_r` was asserted to be less than `BLOCK` and `elem` will therefore
                // at most be pointing to the beginning of the slice.
                unsafe {
                    // Branchless comparison.
                    elem = elem.offset(-1);
                    *end_r = i as u8;
                    end_r = end_r.offset(is_less(&*elem, pivot) as isize);
                }
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = core::cmp::min(width(start_l, end_l), width(start_r, end_r));

        if count > 0 {
            macro_rules! left {
                () => {
                    l.offset(*start_l as isize)
                };
            }
            macro_rules! right {
                () => {
                    r.offset(-(*start_r as isize) - 1)
                };
            }

            // Instead of swapping one pair at the time, it is more efficient to perform a cyclic
            // permutation. This is not strictly equivalent to swapping, but produces a similar
            // result using fewer memory operations.

            // SAFETY: The use of `ptr::read` is valid because there is at least one element in
            // both `offsets_l` and `offsets_r`, so `left!` is a valid pointer to read from.
            //
            // The uses of `left!` involve calls to `offset` on `l`, which points to the
            // beginning of `v`. All the offsets pointed-to by `start_l` are at most `block_l`, so
            // these `offset` calls are safe as all reads are within the block. The same argument
            // applies for the uses of `right!`.
            //
            // The calls to `start_l.offset` are valid because there are at most `count-1` of them,
            // plus the final one at the end of the unsafe block, where `count` is the minimum
            // number of collected offsets in `offsets_l` and `offsets_r`, so there is
            // no risk of there not being enough elements. The same reasoning applies to
            // the calls to `start_r.offset`.
            //
            // The calls to `copy_nonoverlapping` are safe because `left!` and `right!` are
            // guaranteed not to overlap, and are valid because of the reasoning above.
            unsafe {
                let tmp = ptr::read(left!());
                ptr::copy_nonoverlapping(right!(), left!(), 1);

                for _ in 1..count {
                    start_l = start_l.offset(1);
                    ptr::copy_nonoverlapping(left!(), right!(), 1);
                    start_r = start_r.offset(1);
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }

                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                core::mem::forget(tmp);
                start_l = start_l.offset(1);
                start_r = start_r.offset(1);
            }
        }

        if start_l == end_l {
            // All out-of-order elements in the left block were moved. Move to the next block.
            // block-width-guarantee
            // SAFETY: if `!is_done` then the slice width is guaranteed to be at least `2*BLOCK`
            // wide. There are at most `BLOCK` elements in `offsets_l` because of its
            // size, so the `offset` operation is safe. Otherwise, the debug assertions
            // in the `is_done` case guarantee that `width(l, r) == block_l + block_r`,
            // namely, that the block sizes have been adjusted to account
            // for the smaller number of remaining elements.
            l = unsafe { l.offset(block_l as isize) };
        }

        if start_r == end_r {
            // All out-of-order elements in the right block were moved. Move to the previous block.

            // SAFETY: Same argument as [block-width-guarantee]. Either this is a full block
            // `2*BLOCK`-wide, or `block_r` has been adjusted for the last handful of
            // elements.
            r = unsafe { r.offset(-(block_r as isize)) };
        }

        if is_done {
            break;
        }
    }

    // All that remains now is at most one block (either the left or the right) with out-of-order
    // elements that need to be moved. Such remaining elements can be simply shifted to the end
    // within their block.

    if start_l < end_l {
        // The left block remains.
        // Move its remaining out-of-order elements to the far right.
        debug_assert_eq!(width(l, r), block_l);
        while start_l < end_l {
            // remaining-elements-safety
            // SAFETY: while the loop condition holds there are still elements in `offsets_l`, so it
            // is safe to point `end_l` to the previous element.
            //
            // The `ptr::swap` is safe if both its arguments are valid for reads and writes:
            //  - Per the debug assert above, the distance between `l` and `r` is `block_l`
            //    elements, so there can be at most `block_l` remaining offsets between `start_l`
            //    and `end_l`. This means `r` will be moved at most `block_l` steps back, which
            //    makes the `r.offset` calls valid (at that point `l == r`).
            //  - `offsets_l` contains valid offsets into `v` collected during the partitioning of
            //    the last block, so the `l.offset` calls are valid.
            unsafe {
                end_l = end_l.offset(-1);
                ptr::swap(l.offset(*end_l as isize), r.offset(-1));
                r = r.offset(-1);
            }
        }
        width(v.as_mut_ptr(), r)
    } else if start_r < end_r {
        // The right block remains.
        // Move its remaining out-of-order elements to the far left.
        debug_assert_eq!(width(l, r), block_r);
        while start_r < end_r {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                end_r = end_r.offset(-1);
                ptr::swap(l, r.offset(-(*end_r as isize) - 1));
                l = l.offset(1);
            }
        }
        width(v.as_mut_ptr(), l)
    } else {
        // Nothing else to do, we're done.
        width(v.as_mut_ptr(), l)
    }
}

fn partition_in_blocks_dual<T, F>(
    data: &mut [T],
    low: &T,
    high: &T,
    is_less: &mut F,
) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // Number of elements in a typical block.
    const BLOCK: usize = 4;

    // The partitioning algorithm repeats the following steps until completion:
    //
    // 1. Trace a block from the left side to identify elements greater than or equal to the pivot.
    // 2. Trace a block from the right side to identify elements smaller than the pivot.
    // 3. Exchange the identified elements between the left and right side.
    //
    // We keep the following variables for a block of elements:
    //
    // 1. `block` - Number of elements in the block.
    // 2. `start` - Start pointer into the `offsets` array.
    // 3. `end` - End pointer into the `offsets` array.
    // 4. `offsets - Indices of out-of-order elements within the block.

    // The current block on the left side (from `l` to `l.add(block_l)`).
    let s = data.as_mut_ptr();
    let e = unsafe { s.add(data.len()) };
    let mut l = s;
    let mut p = l;
    let mut block_l = BLOCK;
    let mut start_l: *mut u8 = ptr::null_mut();
    let mut end_l: *mut u8 = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())`
    // is always safe`
    let mut r = e;
    let mut q = r;
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        unsafe { r.offset_from(l) as usize }
    }

    fn saturating_width<T>(l: *mut T, r: *mut T) -> usize {
        if l <= r {
            width(l, r)
        } else {
            0
        }
    }

    loop {
        // We are done with partitioning block-by-block when `l` and `r` get very close. Then we do
        // some patch-up work in order to partition the remaining elements in between.
        let is_done = width(l, r) <= 2 * BLOCK;

        if is_done {
            // Number of remaining elements (still not compared to the pivot).
            let mut rem = width(l, r);
            if start_l < end_l || start_r < end_r {
                rem -= BLOCK;
            }

            // Adjust block sizes so that the left and right block don't overlap, but get perfectly
            // aligned to cover the whole remaining gap.
            if start_l < end_l {
                block_r = rem;
            } else if start_r < end_r {
                block_l = rem;
            } else {
                // There were the same number of elements to switch on both blocks during the last
                // iteration, so there are no remaining elements on either block. Cover the
                // remaining items with roughly equally-sized blocks.
                block_l = rem / 2;
                block_r = rem - block_l;
            }
            debug_assert!(block_l <= BLOCK && block_r <= BLOCK);
            debug_assert!(width(l, r) == block_l + block_r);
        }

        if start_l == end_l {
            // Trace `block_l` elements from the left side.
            start_l = offsets_l.as_mut_ptr().cast();
            end_l = start_l;
            let mut elem = l;

            for i in 0..block_l {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                // According to the conditions required by the function, we satisfy them
                // because:
                // 1. `offsets_l` is stack-allocated, and thus considered separate allocated object.
                // 2. The function `is_less` returns a `bool`. Casting a `bool` will
                //    never overflow `isize`.
                // 3. We have guaranteed that `block_l` will be `<= BLOCK`. Plus, `end_l` was
                //    initially set to the begin pointer of `offsets_` which was declared on the
                //    stack.Thus, we know that even in the worst case (all  invocations of `is_less`
                //    returns false) we will only be at most 1 byte pass the end.
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially the begin pointer to the slice which is always valid.
                unsafe {
                    // Branchless comparison.
                    *end_l = i as u8;
                    end_l = end_l.offset(!is_less(&*elem, low) as isize);
                    elem = elem.offset(1);
                }
            }
        }

        if start_r == end_r {
            // Trace `block_r` elements from the right side.
            start_r = offsets_r.as_mut_ptr().cast();
            end_r = start_r;
            let mut elem = r;

            for i in 0..block_r {
                // SAFETY: The unsafety operations below involve the usage of the `offset`.
                // According to the conditions required by the function, we satisfy them
                // because:
                //
                // 1. `offsets_r` is stack-allocated, and thus considered separate allocated object.
                // 2. The function `is_less` returns a `bool`. Casting a `bool` will
                //    never overflow `isize`.
                // 3. We have guaranteed that `block_r` will be `<= BLOCK`. Plus, `end_r` was
                //    initially set to the begin pointer of `offsets_` which was declared on the
                //    stack. Thus, we know that even in the worst case (all invocations of `is_less`
                //    returns true) we will only be at most 1 byte pass the end.
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially `1 * sizeof(T)` past the end and we decrement it by `1 * sizeof(T)`
                // before accessing it.
                // Plus, `block_r` was asserted to be less than `BLOCK` and `elem` will therefore
                // at most be pointing to the beginning of the slice.
                unsafe {
                    // Branchless comparison.
                    elem = elem.offset(-1);
                    *end_r = i as u8;
                    // end_r = end_r.offset(is_less(&*elem, high) as isize);
                    end_r = end_r.offset(!is_less(high, &*elem) as isize);
                }
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = core::cmp::min(width(start_l, end_l), width(start_r, end_r)) as isize;

        if count > 0 {
            macro_rules! left {
                () => {
                    l.offset(*start_l as isize)
                };
            }
            macro_rules! right {
                () => {
                    r.offset(-(*start_r as isize) - 1)
                };
            }

            // Instead of swapping one pair at the time, it is more efficient to perform a cyclic
            // permutation. This is not strictly equivalent to swapping, but produces a similar
            // result using fewer memory operations.

            // SAFETY: The use of `ptr::read` is valid because there is at least one element in
            // both `offsets_l` and `offsets_r`, so `left!` is a valid pointer to read from.
            //
            // The uses of `left!` involve calls to `offset` on `l`, which points to the
            // beginning of `v`. All the offsets pointed-to by `start_l` are at most `block_l`, so
            // these `offset` calls are safe as all reads are within the block. The same argument
            // applies for the uses of `right!`.
            //
            // The calls to `start_l.offset` are valid because there are at most `count-1` of them,
            // plus the final one at the end of the unsafe block, where `count` is the minimum
            // number of collected offsets in `offsets_l` and `offsets_r`, so there is
            // no risk of there not being enough elements. The same reasoning applies to
            // the calls to `start_r.offset`.
            //
            // The calls to `copy_nonoverlapping` are safe because `left!` and `right!` are
            // guaranteed not to overlap, and are valid because of the reasoning above.
            unsafe {
                let tmp = ptr::read(left!());
                ptr::copy_nonoverlapping(right!(), left!(), 1);
                for _ in 1..count {
                    start_l = start_l.offset(1);
                    ptr::copy_nonoverlapping(left!(), right!(), 1);
                    start_r = start_r.offset(1);
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }
                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                core::mem::forget(tmp);

                start_l = start_l.offset(1 - count);
                start_r = start_r.offset(1 - count);

                for _ in 0..count {
                    let _l = left!();
                    let _r = right!();
                    if !is_less(&*left!(), low) {
                        ptr::swap(left!(), p);
                        p = p.offset(1);
                    }
                    if !is_less(high, &*right!()) {
                        ptr::swap(right!(), q.offset(-1));
                        q = q.offset(-1);
                    }
                    start_l = start_l.offset(1);
                    start_r = start_r.offset(1);
                }

                // start_l = start_l.offset(1);
                // start_r = start_r.offset(1);
            }
        }

        if start_l == end_l {
            // All out-of-order elements in the left block were moved. Move to the next block.

            // block-width-guarantee
            // SAFETY: if `!is_done` then the slice width is guaranteed to be at least `2*BLOCK`
            // wide. There are at most `BLOCK` elements in `offsets_l` because of its
            // size, so the `offset` operation is safe. Otherwise, the debug assertions
            // in the `is_done` case guarantee that `width(l, r) == block_l + block_r`,
            // namely, that the block sizes have been adjusted to account
            // for the smaller number of remaining elements.
            l = unsafe { l.add(block_l) };
        }

        if start_r == end_r {
            // All out-of-order elements in the right block were moved. Move to the previous block.

            // SAFETY: Same argument as [block-width-guarantee]. Either this is a full block
            // `2*BLOCK`-wide, or `block_r` has been adjusted for the last handful of
            // elements.
            r = unsafe { r.offset(-(block_r as isize)) };
        }

        if is_done {
            break;
        }
    }

    // All that remains now is at most one block (either the left or the right) with out-of-order
    // elements that need to be moved. Such remaining elements can be simply shifted to the end
    // within their block.

    if start_l < end_l {
        // The left block remains.
        // Move its remaining out-of-order elements to the far right.
        debug_assert_eq!(width(l, r), block_l);
        while start_l < end_l {
            // remaining-elements-safety
            // SAFETY: while the loop condition holds there are still elements in `offsets_l`, so it
            // is safe to point `end_l` to the previous element.
            //
            // The `ptr::swap` is safe if both its arguments are valid for reads and writes:
            //  - Per the debug assert above, the distance between `l` and `r` is `block_l`
            //    elements, so there can be at most `block_l` remaining offsets between `start_l`
            //    and `end_l`. This means `r` will be moved at most `block_l` steps back, which
            //    makes the `r.offset` calls valid (at that point `l == r`).
            //  - `offsets_l` contains valid offsets into `v` collected during the partitioning of
            //    the last block, so the `l.offset` calls are valid.
            unsafe {
                end_l = end_l.offset(-1);
                ptr::swap(l.offset(*end_l as isize), r.offset(-1));
                if !is_less(high, &*r.offset(-1)) {
                    ptr::swap(r.offset(-1), q.offset(-1));
                    q = q.offset(-1);
                }
                r = r.offset(-1);
            }
        }
        l = unsafe { r.offset(-1) };
    } else if start_r < end_r {
        // The right block remains.
        // Move its remaining out-of-order elements to the far left.
        debug_assert_eq!(width(l, r), block_r);
        while start_r < end_r {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                end_r = end_r.offset(-1);
                ptr::swap(l, r.offset(-(*end_r as isize) - 1));
                if !is_less(&*l, low) {
                    ptr::swap(l, p);
                    p = p.offset(1);
                }
                l = l.offset(1);
            }
        }
        r = unsafe { l.add(1) };
    } else {
        // Nothing else to do, we're done.
        r = unsafe { l.add(1) };
    }

    unsafe {
        if l < e && is_less(&*l, low) {
            l = l.add(1);
        }
        if r > s && is_less(high, &*r.sub(1)) {
            r = r.sub(1);
        }
    }

    let (a, b) = (saturating_width(p, l), width(s, p));
    for offset in 0..core::cmp::min(a, b) {
        unsafe {
            l = l.sub(1);
            ptr::swap(s.add(offset), l);
        }
    }

    let (c, d) = (saturating_width(r, q), width(q, e));
    for offset in 0..core::cmp::min(c, d) {
        unsafe {
            ptr::swap(r, e.sub(offset + 1));
            r = r.add(1);
        }
    }
    let (u, v) = (a, data.len() - c);
    (u, v)
}
