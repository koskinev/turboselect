mod pcg_rng;
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr, slice,
};
use pcg_rng::PCGRng;

#[cfg(test)]
mod tests;

const ALPHA: f64 = 0.25;
const BETA: f64 = 0.15;
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

    /// Creates a new `Elem` from a single element.
    fn from_mut(elem: &mut T) -> Self {
        let origin = elem as *mut T;
        unsafe {
            let val = origin.read();
            let tmp = MaybeUninit::new(val);
            Self {
                origin,
                ptr: Some(origin),
                tmp,
            }
        }
    }

    /// Returns a reference to the element at `index`. Unsafe because index must be in bounds.
    unsafe fn get(&self, index: usize) -> &T {
        &*self.origin.add(index)
    }

    fn new(origin: *mut T) -> Self {
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

/// Puts the minimum elements at the beginning of the slice and returns the indices of the first and
/// last elements equal to the minimum.
fn select_min<T: Ord>(data: &mut [T], is_less: impl Fn(&T, &T) -> bool) -> (usize, usize) {
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
fn select_max<T: Ord>(data: &mut [T], is_less: impl Fn(&T, &T) -> bool) -> (usize, usize) {
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
    let (low, _high) = floyd_rivest_select(sample, index, is_less, rng);
    low
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
        quickselect(data, index, T::lt, rng.as_mut());
    } else {
        floyd_rivest_select(data, index, T::lt, rng.as_mut());
    }
    &data[index]
}

fn sort_2<T: Ord>(data: &mut [T], a: usize, b: usize) -> bool {
    debug_assert!(a != b);
    debug_assert!(a < data.len());
    debug_assert!(b < data.len());

    let origin = data.as_mut_ptr();
    unsafe {
        let a = origin.add(a) as *mut T;
        let b = origin.add(b) as *mut T;
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

// Reorders the slice so that the element at `index` is at its sorted position. Returns the
// indices of the first and last elements equal to the element at `index`.
fn floyd_rivest_select<T: Ord, F>(
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
            break select_min(inner, is_less);
        } else if inner_index == inner.len() - 1 {
            break select_max(inner, is_less);
        } else if inner.len() < CUT {
            break quickselect(inner, inner_index, is_less, rng);
        } else if delta > 0 {
            let p = prepare_partition_2(inner, inner_index, is_less, rng);
            let sub = &mut inner[p..];
            let u = hoare_partition(sub, 0, is_less);
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
            let p = prepare_partition_2(inner, inner_index, is_less, rng);
            let sub = &mut inner[p..];
            let (u, v) = lomuto_ternary_partition(sub, 0, is_less);
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

fn quickselect<T, F>(data: &mut [T], index: usize, is_less: F, rng: &mut PCGRng) -> (usize, usize)
where
    T: Ord,
    F: Fn(&T, &T) -> bool + Copy,
{
    let (mut d, mut i) = (data, index);
    let (mut offset, mut delta) = (0, usize::MAX);
    assert!(i < d.len());
    let (u, v) = loop {
        match (i, d.len()) {
            (0, _) => break select_min(d, is_less),
            (i, len) if i == len - 1 => break select_max(d, is_less),
            (_, 25..) => {
                let len = d.len();
                let sample = sample(d, 25, rng);
                for j in 0..5 {
                    median_5(sample, j, j + 5, j + 10, j + 15, j + 20);
                }
                sort_5(sample, 10, 11, 12, 13, 14);
                if delta == 0 {
                    match lomuto_ternary_partition(d, 12, is_less) {
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
                    match hoare_partition(d, 12, is_less) {
                        u if i < u => {
                            d = &mut d[..u];
                        }
                        u => {
                            d = &mut d[u..];
                            i -= u;
                            offset += u;
                        }
                    }
                }
                delta = len - d.len();
            }
            (_, 6..) => {
                median_5(d, 0, 1, 2, 3, 4);
                match lomuto_ternary_partition(d, 2, is_less) {
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

fn hoare_partition<T: Ord>(data: &mut [T], p: usize, is_less: impl Fn(&T, &T) -> bool) -> usize {
    data.swap(0, p);
    let (head, tail) = data.split_first_mut().unwrap();
    let mut pivot = Elem::from_mut(head);

    let (mut l, mut r) = (0, tail.len() - 1);
    unsafe {
        while l < r && is_less(pivot.element(), tail.get_unchecked(r)) {
            r -= 1;
        }
        while l < r && is_less(tail.get_unchecked(l), pivot.element()) {
            l += 1;
        }
    }

    let n = r - l + 1;
    let mut tmp = Elem::new(tail[l..=r].as_mut_ptr());
    let (mut i, mut j) = (0, n - 1);
    let mut h: u8;
    let mut num_ge: u8 = 0;
    let mut num_lt: u8 = 0;
    let mut start_ge: u8 = 0;
    let mut start_lt: u8 = 0;

    let mut offsets_ge = Block::new();
    let mut offsets_lt = Block::new();

    while j - i + 1 > 2 * BLOCK {
        if num_ge == 0 {
            start_ge = 0;
            h = 0;
            while h < BLOCK as u8 {
                unsafe {
                    offsets_ge.write(num_ge, h);
                    let elem = tmp.get(i + h as usize);
                    num_ge += !is_less(elem, pivot.element()) as u8;
                }
                h += 1;
            }
        }
        if num_lt == 0 {
            start_lt = 0;
            h = 0;
            while h < BLOCK as u8 {
                unsafe {
                    offsets_lt.write(num_lt, h);
                    let elem = tmp.get(j - h as usize);
                    num_lt += is_less(elem, pivot.element()) as u8;
                }
                h += 1;
            }
        }

        let num = num_ge.min(num_lt);
        if num > 0 {
            h = 0;
            while h < num {
                unsafe {
                    let m = i + offsets_ge.get(start_ge + h);
                    let n = j - offsets_lt.get(start_lt + h);
                    tmp.select(m);
                    tmp.swap(n);
                }
                h += 1;
            }

            num_ge -= num;
            num_lt -= num;
            start_ge += num;
            start_lt += num;
        }

        i += BLOCK * (num_ge == 0) as usize;
        j -= BLOCK * (num_lt == 0) as usize;
    }

    // Process the remaining elements
    i += (start_ge as usize) * (num_ge > 0) as usize;
    j -= (start_lt as usize) * (num_lt > 0) as usize;
    unsafe {
        loop {
            while i < j && tmp.get(i) < head {
                i += 1;
            }
            while i < j && tmp.get(j) >= head {
                j -= 1;
            }
            if i < j {
                tmp.select(i);
                tmp.swap(j);
                i += 1;
                j -= 1;
            } else {
                break;
            }
        }
        while i < n && tmp.get(i) < head {
            i += 1;
        }
    }

    let u = l + i;
    unsafe {
        pivot.set(u);
    }
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
fn lomuto_ternary_partition<T: Ord>(
    data: &mut [T],
    p: usize,
    is_less: impl Fn(&T, &T) -> bool,
) -> (usize, usize) {
    debug_assert!(!data.is_empty());
    data.swap(0, p);
    let (head, tail) = data.split_first_mut().unwrap();
    let mut pivot = Elem::from_mut(head);
    let (mut l, mut r) = (0, tail.len() - 1);
    let (mut i, mut j, mut k) = (0, 0, 0);
    unsafe {
        while l < r && is_less(pivot.element(), tail.get_unchecked(r)) {
            r -= 1;
        }
        while l < r && is_less(tail.get_unchecked(l), pivot.element()) {
            l += 1;
        }
    }
    let n = r - l + 1;

    let mut tmp = Elem::new(tail[l..=r].as_mut_ptr());
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
            unsafe {
                let elem = tmp.get(k + h as usize);
                offsets.write(num_le, h);
                num_le += !is_less(pivot.element(), elem) as u8;
            }
            h += 1;
        }
        h = 0;

        // Swap each element <= pivot with the first element > pivot and store the offsets to
        // elements < pivot.
        while h < num_le {
            unsafe {
                let m = k + offsets.get(h);
                tmp.select(m);
                offsets.write(num_lt, h);
                num_lt += is_less(tmp.element(), pivot.element()) as u8;
                tmp.swap(j + h as usize);
            }
            h += 1;
        }
        h = 0;

        // Swap each element < pivot with the first element >= pivot.
        while h < num_lt {
            unsafe {
                let m = j + offsets.get(h);
                tmp.select(m);
                tmp.swap(i + h as usize);
            }
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
    let (u, v) = (l + i, l + j);
    unsafe {
        pivot.set(u);
    }
    (u, v)
}
