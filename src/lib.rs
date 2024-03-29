#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod math;

#[cfg(feature = "std")]
#[cfg(test)]
mod benches;
mod sort;

#[cfg(feature = "std")]
#[cfg(test)]
mod tests;

#[cfg(test)]
mod wyrand;

use core::{
    cmp::{self, Ordering},
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{ControlFlow, Deref, DerefMut, Range},
    ptr,
};
use math::{ceil, lerp, sqrt};
use sort::tinysort;

/// Represents an element removed from a slice. When dropped, copies the value into `dst`.
struct Elem<T> {
    value: ManuallyDrop<T>,
    dst: *mut T,
}

impl<T> Deref for Elem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Elem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Drop for Elem<T> {
    fn drop(&mut self) {
        // SAFETY: This is a helper class. We must ensure that `dst` is valid for writes and is
        // not referenced by anyone else.
        unsafe {
            let value = ManuallyDrop::take(&mut self.value);
            ptr::write(self.dst, value)
        }
    }
}

impl<T> Elem<T> {
    /// Creates a new `Elem` from a mutable reference. This method can be safely used only if `src`
    /// is not used for the duration of the `Elem`'s lifetime.
    unsafe fn new(src: *mut T) -> Self {
        Self {
            value: ManuallyDrop::new(ptr::read(src)),
            dst: src,
        }
    }
}

/// Given two values `x` and `y`, and a comparator function that returns `true` if `x < y`,
/// this macro returns `true` if `x >= y`.
///
/// # Arguments
///
/// * `$x` - The first value to compare.
/// * `$y` - The second value to compare.
/// * `$lt` - The comparator function that returns `true` if `$x` is less than `$y`.
macro_rules! ge {
    ($x:expr, $y:expr, $lt:expr) => {
        !(($lt)($x, $y))
    };
}

/// Given two values `x` and `y`, and a comparator function that returns `true` if `x < y`,
/// this macro returns `true` if `x <= y`.
///
/// # Arguments
///
/// * `$x` - The first value to compare.
/// * `$y` - The second value to compare.
/// * `$lt` - The comparator function that returns `true` if `$x` is less than `$y`.
macro_rules! le {
    ($x:expr, $y:expr, $lt:expr) => {
        !($lt)($y, $x)
    };
}

#[derive(PartialEq, Eq)]
/// An enumeration representing the sort order of a slice.
#[repr(isize)]
enum SortOrder {
    /// The slice is sorted in ascending order.
    Ascending = 1,
    /// The slice is sorted in descending order.
    Descending = -1,
    /// The slice is not sorted.
    Unsorted = 0,
}

impl From<isize> for SortOrder {
    fn from(value: isize) -> Self {
        match value {
            1 => Self::Ascending,
            -1 => Self::Descending,
            _ => Self::Unsorted,
        }
    }
}

/// Selects the pivot element for partitioning the slice. Returns `(p, is_repeated)` where `p` is
/// the index of the pivot element and `is_repeated` is a boolean indicating if the pivot is likely
/// to have many duplicates.
fn choose_pivot<T, F>(data: &mut [T], index: usize, lt: &mut F) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    match data.len() {
        // For relatively small slices, we use a `kth-of-nths` strategy.
        len if len <= 256 => kth_of_nths::<3, _, _>(data, index, lt),
        len if len <= 1024 => kth_of_nths::<5, _, _>(data, index, lt),
        len if len <= 4096 => kth_of_nths::<7, _, _>(data, index, lt),
        // Larger slices benefit from more accurate pivot selection.
        len => {
            const ALPHA: f64 = 0.75;
            let count = ceil(ALPHA * sqrt(len as f64)) as usize;
            let index = nudge(index, len);
            let k = (count * index) / len;
            select(sample(data, count), k, lt);
            let is_repeated = match k {
                0 => ge!(&data[k], &data[k + 1], lt),
                k if k == count - 1 => ge!(&data[k - 1], &data[k], lt),
                _ => ge!(&data[k - 1], &data[k + 1], lt),
            };
            (k, is_repeated)
        }
    }
}

#[inline]
/// Chooses a randomized pivot for the given index. First, puts a `N * N` random sample to the
/// beginning of the slice. Then sorts `N` groups of `N` elements in the sample, each `N` elements
/// apart. Finally, sorts the group where the pivot is located. Returns `(p, n)` where `p` is
/// the index of the selected pivot and `n` is the number of elements in the group.
fn kth_of_nths<const N: usize, T, F>(data: &mut [T], index: usize, lt: &mut F) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    // Calculate:
    // - `k`: sample index corresponding to the pivot location.
    // - `g`: index of the `N` element group where the pivot is located.
    let len = data.len();
    let k = (N * N * index) / len;
    let g = k / N;

    // Take the sample and sort the groups
    let sample = sample(data, N * N);
    for chunk in sample.chunks_mut(N) {
        tinysort(chunk, lt);
    }

    // Move the `g`th elements of each group to the beginning of the slice.
    // Then sort the first `N` elements.
    for j in 0..N {
        sample.swap(j, g + N * j);
    }
    tinysort(&mut sample[..N], lt);
    let p = if index.abs_diff(len / 2) > len / 5 {
        (N / 2 + k % N) / 2
    } else {
        N / 2
    };

    // Compare the pivot with the first element of the group. If they are equal, the pivot is
    // likely to have many duplicates.
    let is_repeated = sample[p.saturating_sub(1)..(p + 1).min(N - 1)]
        .windows(2)
        .any(|w| ge!(&w[0], &w[1], lt));
    (p, is_repeated)
}

fn is_sorted<T, F>(data: &[T], lt: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    const BLOCK: usize = 32;
    let mut stopped = false;
    let mut a = data.as_ptr();
    let mut count = data.len() - 1;
    while count > BLOCK {
        for _ in 0..BLOCK {
            stopped |= unsafe { lt(&*a.add(1), &*a) };
            a = unsafe { a.add(1) };
        }
        if stopped {
            return false;
        }
        count -= BLOCK;
    }
    while count > 0 && unsafe { !lt(&*a.add(1), &*a) } {
        a = unsafe { a.add(1) };
        count -= 1;
    }
    count == 0
}

fn detect_order<T, F>(data: &[T], lt: &mut F) -> Option<Ordering>
where
    F: FnMut(&T, &T) -> bool,
{
    if data.is_empty() {
        return None;
    }

    if is_sorted(data, lt) {
        Some(Ordering::Greater)
    } else if is_sorted(data, &mut |a, b| lt(b, a)) {
        Some(Ordering::Less)
    } else {
        None
    }
}

fn nudge(index: usize, len: usize) -> usize {
    const GAP_START: f64 = 0.01;
    const GAP_END: f64 = 0.001;
    const LIMIT: f64 = 100_000_000.;
    let n = len as f64;
    let g = lerp(GAP_START, GAP_END, n.min(LIMIT) / LIMIT);
    // let g = (GAP_START + 1.0 / sqrt(n)) / 2.0;
    let x = index as f64 / len as f64;
    if index < len / 2 {
        (n * (x + g).min(0.5)) as usize
    } else {
        ceil(n * (x - g).max(0.5)).min(n - 1.) as usize
    }
}

/// Partitions `data` into three parts using the element at `index` as the pivot. Returns `(u, v)`,
/// where `u` is the number of elements less than the pivot, and `v - u` is the number of elements
/// following the pivot equal to it. Note that `v` is not the number of elements less than or equal
/// to the pivot, because the rightmost partition may contain elements equal to the pivot.
///
/// The resulting partitioning is as follows:
///
/// ```text
/// ┌─────────────┬──────────────┬──────────────┐
/// │ x < data[u] │ x == data[u] │ x >= data[u] │
/// └─────────────┴──────────────┴──────────────┘
///                u            v
/// ```
///
/// Panics if `index` is out of bounds.
fn partition_at<T, F>(data: &mut [T], index: usize, lt: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // This ensures that the index is in bounds.
    data.swap(0, index);

    let (elem, tail) = data.split_first_mut().unwrap();
    let (u, mut v);
    {
        // Read the pivot into the stack. The read below is safe, because the pivot is the first
        // element in the slice.
        let pivot = unsafe { Elem::new(elem) };

        // Find the positions of the first pair of out-of-order elements.
        let ptr = tail.as_ptr();
        let (mut l, mut r) = (0, tail.len());

        // SAFETY: The calls to get_unchecked are safe, because the slice is non-empty and we
        // ensure that `l <= r`.
        while l < r && unsafe { lt(&*ptr.add(l), &*pivot) } {
            l += 1;
        }
        while l < r && unsafe { ge!(&*ptr.add(r - 1), &*pivot, lt) } {
            r -= 1;
        }
        u = l + partition_in_blocks(&mut tail[l..r], &*pivot, lt);
        v = u;
        // Scan the elements after the pivot until we find one that is greater than the pivot.
        while v < tail.len() && unsafe { le!(&*ptr.add(v), &*pivot, lt) } {
            v += 1;
        }
    }
    data.swap(0, u);
    (u, v)
}

/// Partitions `data` into three parts using the element at `index` as the pivot.
///
/// Returns `(u, v)`, where `u` is the number of elements less than the pivot, and `v` is the number
/// of elements less than or equal to the pivot.
///
/// The resulting partitioning is:
///
/// ```text
/// ┌─────────────┬──────────────┬─────────────┐
/// │ x < data[u] │ x == data[u] │ x > data[u] │
/// └─────────────┴──────────────┴─────────────┘
///                u            v
/// ```
///
/// Panics if `index` is out of bounds.
fn partition_equal<T, F>(data: &mut [T], index: usize, lt: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let (u, v) = partition_at(data, index, lt);
    let dups = partition_equal_min(data[v..].as_mut(), 0, lt).1;
    (u, v + dups)
}

/// Puts the minimum elements at the beginning of the slice and returns the indices of the first and
/// last elements equal to the minimum. The `init` argument is the index of the element to use as
/// the initial minimum. Returns `(u, v)`, where `u` is 0 and `v` is the number of elements equal
/// to the minimum.
///
/// The resulting partitioning is as follows:
///
/// ```text
/// ┌──────────┬─────────┐
/// │ x == min │ x > min │
/// └──────────┴─────────┘
///  u == 0   v
/// ```
fn partition_equal_min<T, F>(data: &mut [T], init: usize, lt: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // If the slice is empty or it has only one element, there is nothing to do.
    if data.len() < 2 {
        return (0, data.len() - 1);
    }

    // Initialize the minimum
    data.swap(0, init);

    // Copy the initial minimum to the stack
    let (head, tail) = data.split_first_mut().unwrap();
    // SAFETY: `head` is not used after this point.
    let mut min = unsafe { Elem::new(head) };

    let Range { start: l, end: r } = tail.as_mut_ptr_range();
    let mut elem = l;
    let mut dup = l;

    // Setup the offsets array.
    const BLOCK: usize = 64;
    let mut offsets = [MaybeUninit::<u8>::uninit(); BLOCK];
    let mut start = offsets.as_mut_ptr().cast();
    let mut end: *mut u8 = start;

    while elem < r {
        // Scan the next block.
        let block = cmp::min(BLOCK, width(elem, r));
        unsafe {
            // Scan the block and store offsets to the elements that satisfy `elem <= min`.
            // SAFETY: The unsafety operations below involve the usage of the `offset`.
            // According to the conditions required by the function, we satisfy them
            // because:
            // 1. `offsets` is stack-allocated, and thus considered separate allocated object.
            // 2. The comparison returns a `bool`. Casting a `bool` will never overflow `isize`.
            // 3. We have guaranteed that `block` will be `<= BLOCK`. Plus, `end` was initially set
            //    to the begin pointer of `offsets` which was declared on the stack. Thus, we know
            //    that even in the worst case (all comparisons return true) we will only be at most
            //    1 byte pass the end
            //
            // Another unsafety operation here is dereferencing `elem`. However, `elem` was
            // initially the begin pointer to the slice which is always valid.
            for offset in 0..block {
                end.write(offset as u8);
                let is_le = le!(&*elem.add(offset), &*min, lt);
                end = end.add(is_le as usize);
            }
            // Scan the found elements
            for _ in 0..width(start, end) {
                // SAFETY: We know that the element is in bounds because we just scanned it.
                let next = elem.add(*start as usize);
                if lt(&*next, &*min) {
                    // We found a new minimum.
                    dup = l;
                    // SAFETY: `next` and `min` are both valid and they cannot overlap because
                    // `min` is allocated on the stack, while `next` points to an element of the
                    // slice.
                    ptr::swap_nonoverlapping(next, &mut *min, 1);
                } else if le!(&*next, &*min, lt) {
                    // We found an element equal to the minimum.
                    if width(l, dup) < width(l, next) {
                        // SAFETY: The above condition ensures that `next` and `dup` don't
                        // overlap. Also, `dup` cannot be off bounds (see below).
                        ptr::swap_nonoverlapping(next, dup, 1);
                    }
                    // SAFETY: `dup` is guaranteed to be in bounds, since it's incremented at
                    // most `tail.len()` times.
                    dup = dup.add(1);
                }
                // SAFETY: `start` is guaranteed to be in bounds, since `width(start, end) <=
                // BLOCK`.
                start = start.add(1);
            }
            elem = elem.add(block);
            start = offsets.as_mut_ptr().cast();
            end = start;
        }
    }
    (0, width(l, dup))
}

/// Partitions `data` into elements smaller than `pivot`, followed by elements greater than or equal
/// to `pivot`. Returns the number of elements smaller than `pivot`.
///
/// This function is a slightly modified version of `core::slice::sort::partition_in_blocks`.
///
/// Partitioning is performed block-by-block in order to minimize the cost of branching operations.
/// This idea is presented in the [BlockQuicksort][pdf] paper.
///
/// [pdf]: https://drops.dagstuhl.de/opus/volltexte/2016/6389/pdf/LIPIcs-ESA-2016-38.pdf
fn partition_in_blocks<T, F>(data: &mut [T], pivot: &T, lt: &mut F) -> usize
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
    // 4. `offsets` - Indices of out-of-order elements within the block.

    let Range {
        start: mut l,
        end: mut r,
    } = data.as_mut_ptr_range();

    // The current block on the left side (from `l` to `l.add(block_l)`).
    let mut block_l = BLOCK;
    let mut start_l = ptr::null_mut();
    let mut end_l = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())`
    // is always safe
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

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
                // 2. The comparison returns a `bool`. Casting a `bool` will never overflow `isize`.
                // 3. We have guaranteed that `block_l` will be `<= BLOCK`. Plus, `end_l` was
                //    initially set to the begin pointer of `offsets_l` which was declared on the
                //    stack. Thus, we know that even in the worst case (all comparisons return true)
                //    we will only be at most 1 byte pass the end
                //
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially the begin pointer to the slice which is always valid.
                unsafe {
                    // Branchless comparison.
                    *end_l = i as u8;
                    end_l = end_l.add(ge!(&*elem, pivot, lt) as usize);
                    elem = elem.add(1);
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
                // 1. `offsets_r` is stack-allocated, and thus considered separate allocated object.
                // 2. The comparison returns a `bool`. Casting a `bool` will never overflow `isize`.
                // 3. We have guaranteed that `block_r` will be `<= BLOCK`. Plus, `end_r` was
                //    initially set to the begin pointer of `offsets_r` which was declared on the
                //    stack. Thus, we know that even in the worst case (all all comparisons return
                //    true) we will only be at most 1 byte pass the end.
                //
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially `1 * sizeof(T)` past the end and we decrement it by `1 * sizeof(T)`
                // before accessing it. Plus, `block_r` was asserted  to be less than `BLOCK` and
                // `elem` will therefore at most be pointing to the  beginning of the slice.
                unsafe {
                    // Branchless comparison.
                    elem = elem.sub(1);
                    *end_r = i as u8;
                    end_r = end_r.add(lt(&*elem, pivot) as usize);
                }
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = cmp::min(width(start_l, end_l), width(start_r, end_r));

        if count > 0 {
            if count < BLOCK {
                macro_rules! left {
                    () => {
                        l.add(usize::from(*start_l))
                    };
                }
                macro_rules! right {
                    () => {
                        r.sub(usize::from(*start_r) + 1)
                    };
                }

                // Instead of swapping one pair at the time, it is more efficient to perform a
                // cyclic permutation. This is not strictly equivalent to swapping,
                // but produces a similar result using fewer memory operations.

                // SAFETY: The use of `ptr::read` is valid because there is at least one element in
                // both `offsets_l` and `offsets_r`, so `left!` is a valid pointer to read from.
                //
                // The uses of `left!` involve calls to `offset` on `l`, which points to the
                // beginning of `v`. All the offsets pointed-to by `start_l` are at most `block_l`,
                // so these `offset` calls are safe as all reads are within the
                // block. The same argument applies for the uses of `right!`.
                //
                // The calls to `start_l.offset` are valid because there are at most `count-1` of
                // them, plus the final one at the end of the unsafe block, where
                // `count` is the minimum number of collected offsets in `offsets_l`
                // and `offsets_r`, so there is no risk of there not being enough
                // elements. The same reasoning applies to the calls to
                // `start_r.offset`.
                //
                // The calls to `copy_nonoverlapping` are safe because `left!` and `right!` are
                // guaranteed not to overlap, and are valid because of the reasoning above.
                unsafe {
                    let tmp = ptr::read(left!());
                    ptr::copy_nonoverlapping(right!(), left!(), 1);

                    for _ in 1..count {
                        start_l = start_l.add(1);
                        ptr::copy_nonoverlapping(left!(), right!(), 1);
                        start_r = start_r.add(1);
                        ptr::copy_nonoverlapping(right!(), left!(), 1);
                    }

                    ptr::copy_nonoverlapping(&tmp, right!(), 1);
                    core::mem::forget(tmp);
                    start_l = start_l.add(1);
                    start_r = start_r.add(1);
                }
            } else {
                // If both blocks are full, we can swap them as a whole.
                unsafe {
                    ptr::swap_nonoverlapping(l, r.sub(BLOCK), BLOCK);
                    start_l = end_l;
                    start_r = end_r;
                }
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
            r = unsafe { r.sub(block_r) };
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
                end_l = end_l.sub(1);
                ptr::swap(l.add(usize::from(*end_l)), r.sub(1));
                r = r.sub(1);
            }
        }
        width(data.as_mut_ptr(), r)
    } else if start_r < end_r {
        // The right block remains.
        // Move its remaining out-of-order elements to the far left.
        debug_assert_eq!(width(l, r), block_r);
        while start_r < end_r {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                end_r = end_r.sub(1);
                ptr::swap(l, r.sub(usize::from(*end_r) + 1));
                l = l.add(1);
            }
        }
        width(data.as_mut_ptr(), l)
    } else {
        // Nothing else to do, we're done.
        width(data.as_mut_ptr(), l)
    }
}

/// Samples `count` elements evenly and places them into the beginning of the slice. Returns the
/// sampled elements as a slice. Panics if `data.len() / (count + 1) == 0`.
fn sample<T>(data: &mut [T], count: usize) -> &mut [T] {
    let step = data.len() / count;
    assert!(step > 0);
    unsafe {
        let ptr = data.as_mut_ptr();
        // Read the first element into a temporary location.
        // SAFETY: The read is safe because `ptr` points to the first element of `data` and `data`
        // is non-empty.
        let tmp = ManuallyDrop::new(ptr::read(ptr));
        let (mut src, mut dst) = (ptr, ptr);

        for _ in 1..count {
            // Select the next element.
            // SAFETY: This is safe since `count <= len`.
            dst = dst.add(1);
            // SAFETY: See above for why this is safe.
            ptr::copy(dst, src, 1);
            src = src.add(step);
            ptr::copy(src, dst, 1);
        }
        // Write the temporary element (i.e the original first element) to the last sampled
        // position.
        // SAFETY: This is safe, because `src` is in bounds.
        src.write(ManuallyDrop::into_inner(tmp));
        &mut data[..count]
    }
}

/// Partitions the slice so that elements in `data[..index]` are less than or equal to the pivot
/// and elements in `data[index..]` are greater than or equal to the pivot.
///
/// Panics if `index >= data.len()`.
fn select<T, F>(mut data: &mut [T], mut index: usize, lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(index < data.len());

    fn descend<'a, T>(
        mut data: &'a mut [T],
        mut index: usize,
        u: usize,
        v: usize,
        mut previous_pivot: Option<&'a T>,
    ) -> ControlFlow<(), (&'a mut [T], usize, Option<&'a T>)> {
        // Descend into the appropriate part of the slice or terminate if the pivot is in the
        // correct position.
        if index < u {
            // Select the left part. We don't store the pivot, since all elements on the left
            // are smaller than the pivot.
            data = data[..u].as_mut();
        } else if index > v {
            // Select the right part. Elements on the right can be equal to the pivot,
            // so we store it.
            let (head, tail) = data.split_at_mut(v + 1);
            (data, previous_pivot) = (tail, head.last());
            index -= v + 1;
        } else {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue((data, index, previous_pivot))
    }

    // If the slice is likely to be sorted, try partitioning with the pivot at the index (ascending)
    // or slightly before the corresponding index from the back (descending).
    let mut previous_pivot = None;
    match detect_order(data, lt) {
        Some(Ordering::Greater) => return,
        Some(Ordering::Less) => return data.reverse(),
        _ => {}
    }

    while data.len() > 24 {
        let (u, v) = match index {
            0 => select_min(data, lt),
            i if i == data.len() - 1 => select_max(data, lt),
            _ => {
                let (p, is_repeated) = choose_pivot(data, index, lt);
                match previous_pivot {
                    // Test if the selected pivot is equal to a previous pivot from the left. In
                    // this case we know that the pivot is the minimum of the current slice.
                    Some(was) if ge!(was, &data[p], lt) => partition_equal_min(data, p, lt),

                    // If the selected pivot is equal to it's neighbor elements, use ternary
                    // partitioning, which puts the elements equal to the pivot in the
                    // middle. This is necessary to ensure that the algorithm terminates.
                    _ if is_repeated => partition_equal(data, p, lt),

                    // Otherwise, use the default binary partioning.
                    _ => partition_at(data, p, lt),
                }
            }
        };
        match descend(data, index, u, v, previous_pivot) {
            ControlFlow::Continue(result) => (data, index, previous_pivot) = result,
            ControlFlow::Break(_) => return,
        }
    }
    tinysort(data, lt);
}

/// Finds the minimum element and puts it at the beginning of the slice.
fn select_min<T, F>(data: &mut [T], lt: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // If the slice is empty or it has only one element, there is nothing to do.
    if data.len() < 2 {
        return (0, data.len() - 1);
    }

    // Copy the initial minimum to the stack
    let (head, tail) = data.split_first_mut().unwrap();
    // SAFETY: `head` is not used after this point.
    let mut min = unsafe { Elem::new(head) };

    let Range { start: l, end: r } = tail.as_mut_ptr_range();
    let mut elem = l;

    // Setup the offsets array.
    const BLOCK: usize = 64;
    let mut offsets = [MaybeUninit::<u8>::uninit(); BLOCK];
    let mut start = offsets.as_mut_ptr().cast();
    let mut end: *mut u8 = start;

    while elem < r {
        // Scan the next block.
        let block = cmp::min(BLOCK, width(elem, r));
        unsafe {
            // Scan the block and store offsets to the elements that satisfy `elem <= min`.
            // SAFETY: The unsafety operations below involve the usage of the `offset`.
            // According to the conditions required by the function, we satisfy them
            // because:
            // 1. `offsets` is stack-allocated, and thus considered separate allocated object.
            // 2. The comparison returns a `bool`. Casting a `bool` will never overflow `isize`.
            // 3. We have guaranteed that `block` will be `<= BLOCK`. Plus, `end` was initially set
            //    to the begin pointer of `offsets` which was declared on the stack. Thus, we know
            //    that even in the worst case (all comparisons return true) we will only be at most
            //    1 byte pass the end
            //
            // Another unsafety operation here is dereferencing `elem`. However, `elem` was
            // initially the begin pointer to the slice which is always valid.
            for offset in 0..block {
                end.write(offset as u8);
                let is_lt = lt(&*elem.add(offset), &*min);
                end = end.add(is_lt as usize);
            }
            // Scan the found elements
            for _ in 0..width(start, end) {
                // SAFETY: We know that the element is in bounds because we just scanned it.
                let next = elem.add(*start as usize);
                if lt(&*next, &*min) {
                    // We found a new minimum.
                    // SAFETY: `next` and `min` are both valid and they cannot overlap because
                    // `min` is allocated on the stack, while `next` points to an element of the
                    // slice.
                    ptr::swap_nonoverlapping(next, &mut *min, 1);
                }
                // SAFETY: `start` is guaranteed to be in bounds, since `width(start, end) <=
                // BLOCK`.
                start = start.add(1);
            }
            elem = elem.add(block);
            start = offsets.as_mut_ptr().cast();
            end = start;
        }
    }
    (0, 0)
}

/// Finds the maximum element and puts it at the end of the slice.
fn select_max<T, F>(data: &mut [T], lt: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // If the slice is empty or it has only one element, there is nothing to do.
    if data.len() < 2 {
        return (0, data.len() - 1);
    }

    // Copy the initial minimum to the stack
    let (pivot, rest) = data.split_last_mut().unwrap();
    // SAFETY: `pivot` is not used after this point.
    let mut max = unsafe { Elem::new(pivot) };

    let Range { start: l, end: r } = rest.as_mut_ptr_range();
    let mut elem = l;

    // Setup the offsets array.
    const BLOCK: usize = 64;
    let mut offsets = [MaybeUninit::<u8>::uninit(); BLOCK];
    let mut start = offsets.as_mut_ptr().cast();
    let mut end: *mut u8 = start;

    while elem < r {
        // Scan the next block.
        let block = cmp::min(BLOCK, width(elem, r));
        unsafe {
            // Scan the block and store offsets to the elements that satisfy `elem <= min`.
            // SAFETY: The unsafety operations below involve the usage of the `offset`.
            // According to the conditions required by the function, we satisfy them
            // because:
            // 1. `offsets` is stack-allocated, and thus considered separate allocated object.
            // 2. The comparison returns a `bool`. Casting a `bool` will never overflow `isize`.
            // 3. We have guaranteed that `block` will be `<= BLOCK`. Plus, `end` was initially set
            //    to the begin pointer of `offsets` which was declared on the stack. Thus, we know
            //    that even in the worst case (all comparisons return true) we will only be at most
            //    1 byte pass the end
            //
            // Another unsafety operation here is dereferencing `elem`. However, `elem` was
            // initially the begin pointer to the slice which is always valid.
            for offset in 0..block {
                end.write(offset as u8);
                let is_gt = lt(&*max, &*elem.add(offset));
                end = end.add(is_gt as usize);
            }
            // Scan the found elements
            for _ in 0..width(start, end) {
                // SAFETY: We know that the element is in bounds because we just scanned it.
                let next = elem.add(*start as usize);
                if lt(&*max, &*next) {
                    // We found a new minimum.
                    // SAFETY: `next` and `min` are both valid and they cannot overlap because
                    // `min` is allocated on the stack, while `next` points to an element of the
                    // slice.
                    ptr::swap_nonoverlapping(next, &mut *max, 1);
                }
                // SAFETY: `start` is guaranteed to be in bounds, since `width(start, end) <=
                // BLOCK`.
                start = start.add(1);
            }
            elem = elem.add(block);
            start = offsets.as_mut_ptr().cast();
            end = start;
        }
    }
    (data.len() - 1, data.len() - 1)
}

/// Reorder the slice such that the element at `index` is at its final sorted position.
///
/// This reordering has the additional property that any value at position `i < index` will be
/// less than or equal to any value at a position `j > index`. Additionally, this reordering is
/// unstable (i.e. any number of equal elements may end up at position `index`), in-place
/// (i.e. does not allocate), and *O*(*n*) on average. The worst-case performance is *O*(*n* log
/// *n*). This function is also known as "kth element" in other libraries.
///
/// Returns a triplet of the following from the reordered slice: the subslice prior to `index`, the
/// element at `index`, and the subslice after `index`; accordingly, the values in those two
/// subslices will respectively all be less-than-or-equal-to and greater-than-or-equal-to the value
/// of the element at `index`.
///
/// # Implementation
///
/// The implementation is similar to `core::slice::select_nth_unstable`, but it uses an adaptive
/// pivot selection algorithm. This usually improves performance substantially, especially when
/// `index` is far from the median.
///
/// # Panics
///
/// Panics when `index >= len()`, meaning it always panics on empty slices.
///
/// # Examples
///
/// ```
/// use turboselect::select_nth_unstable;
/// let mut v = [-5i32, 4, 1, -3, 2];
///
/// // Find the median
/// select_nth_unstable(v.as_mut(), 2);
///
/// // We are only guaranteed the slice will be one of the following, based on the way we sort
/// // about the specified index.
/// assert!(
///     v == [-3, -5, 1, 2, 4]
///         || v == [-5, -3, 1, 2, 4]
///         || v == [-3, -5, 1, 4, 2]
///         || v == [-5, -3, 1, 4, 2]
/// );
/// ```
#[inline]
pub fn select_nth_unstable<T>(data: &mut [T], index: usize) -> (&mut [T], &mut T, &mut [T])
where
    T: Ord,
{
    // If there are less than two elements, there is nothing to do. If `T` is a zero sized type, it
    // cannot have any meaningful ordering, so we just return.
    if data.len() < 2 || mem::size_of::<T>() == 0 {
        return split_partition(data, index);
    }

    if index == 0 {
        select_min(data, &mut T::lt);
    } else if index == data.len() - 1 {
        select_max(data, &mut T::lt);
    } else {
        select(data, index, &mut T::lt);
    }
    split_partition(data, index)
}

/// Reorder the slice with a comparator function such that the element at `index` is at its
/// final sorted position.
///
/// This reordering has the additional property that any value at position `i < index` will be
/// less than or equal to any value at a position `j > index` using the comparator function.
/// Additionally, this reordering is unstable (i.e. any number of equal elements may end up at
/// position `index`), in-place (i.e. does not allocate), and *O*(*n*) on average.
/// The worst-case performance is *O*(*n* log *n*). This function is also known as
/// "kth element" in other libraries.
///
/// It returns a triplet of the following from the slice reordered according to the provided
/// comparator function: the subslice prior to `index`, the element at `index`, and the subslice
/// after `index`; accordingly, the values in those two subslices will respectively all be
/// less-than-or-equal-to and greater-than-or-equal-to the value of the element at `index`.
///
/// # Implementation
///
/// The implementation is similar to `core::slice::select_nth_unstable_by`, but it uses an adaptive
/// pivot selection algorithm. This usually improves performance substantially, especially when
/// `index` is far from the median.
///
/// # Panics
///
/// Panics when `index >= len()`, meaning it always panics on empty slices.
///
/// # Examples
///
/// ```
/// use turboselect::select_nth_unstable_by;
/// let mut v = [-5i32, 4, 1, -3, 2];
///
/// // Find the median as if the slice were sorted in descending order.
/// select_nth_unstable_by(&mut v, 2, |a: &i32, b: &i32| b.cmp(a));
///
/// // We are only guaranteed the slice will be one of the following, based on the way we sort
/// // about the specified index.
/// assert!(
///     v == [2, 4, 1, -5, -3]
///         || v == [2, 4, 1, -3, -5]
///         || v == [4, 2, 1, -5, -3]
///         || v == [4, 2, 1, -3, -5]
/// );
/// ```
#[inline]
pub fn select_nth_unstable_by<T, F>(
    data: &mut [T],
    index: usize,
    mut compare: F,
) -> (&mut [T], &mut T, &mut [T])
where
    F: FnMut(&T, &T) -> Ordering,
{
    // If there are less than two elements, there is nothing to do. If `T` is a zero sized type, it
    // cannot have any meaningful ordering, so we just return.
    if data.len() < 2 || mem::size_of::<T>() == 0 {
        return split_partition(data, index);
    }

    let mut lt = |x: &T, y: &T| compare(x, y) == Ordering::Less;

    if index == 0 {
        select_min(data, &mut lt);
    } else if index == data.len() - 1 {
        select_max(data, &mut lt);
    } else {
        select(data, index, &mut lt);
    }
    split_partition(data, index)
}

/// Reorder the slice with a key extraction function such that the element at `index` is at its
/// final sorted position.
///
/// This reordering has the additional property that any value at position `i < index` will be
/// less than or equal to any value at a position `j > index` using the key extraction function.
/// Additionally, this reordering is unstable (i.e. any number of equal elements may end up at
/// position `index`), in-place (i.e. does not allocate), and *O*(*n*) on average.
/// The worst-case performance is *O*(*n* log *n*). This function is also known as "kth element" in
/// other libraries.
///
/// Returns a triplet of the following from the slice reordered according to the provided key
/// extraction function: the subslice prior to `index`, the element at `index`, and the subslice
/// after `index`; accordingly, the values in those two subslices will respectively all be
/// less-than-or-equal-to and greater-than-or-equal-to the value of the element at `index`.
///
/// # Implementation
///
/// The implementation is similar to `core::slice::select_nth_unstable_by_key`, but it uses an
/// adaptive pivot selection algorithm. This usually improves performance substantially, especially
/// when `index` is far from the median.
///
/// # Panics
///
/// Panics when `index >= len()`, meaning it always panics on empty slices.
///
/// # Examples
///
/// ```
/// use turboselect::select_nth_unstable_by_key;
/// let mut v = [-5i32, 4, 1, -3, 2];
///
/// // Return the median as if the array were sorted according to absolute value.
/// select_nth_unstable_by_key(&mut v, 2, |a| a.abs());
///
/// // We are only guaranteed the slice will be one of the following, based on the way we sort
/// // about the specified index.
/// assert!(
///     v == [1, 2, -3, 4, -5]
///         || v == [1, 2, -3, -5, 4]
///         || v == [2, 1, -3, 4, -5]
///         || v == [2, 1, -3, -5, 4]
/// );
/// ```
#[inline]
pub fn select_nth_unstable_by_key<T, K, F>(
    data: &mut [T],
    index: usize,
    mut f: F,
) -> (&mut [T], &mut T, &mut [T])
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    // If there are less than two elements, there is nothing to do. If `T` is a zero sized type, it
    // cannot have any meaningful ordering, so we just return.
    if data.len() < 2 || mem::size_of::<T>() == 0 {
        return split_partition(data, index);
    }

    let mut lt = |x: &T, y: &T| f(x).lt(&f(y));

    if index == 0 {
        select_min(data, &mut lt);
    } else if index == data.len() - 1 {
        select_max(data, &mut lt);
    } else {
        select(data, index, &mut lt);
    }
    split_partition(data, index)
}

#[cfg(feature = "std")]
/// Reorder the slice with a key extraction function such that the element at `index` is at its
/// final sorted position. During selection, the key function is called at most once per element, by
/// using temporary storage to remember the results of key evaluation.
///
/// This reordering has the additional property that any value at position `i < index` will be
/// less than or equal to any value at a position `j > index` using the key extraction function.
/// Additionally, this reordering is unstable (i.e. any number of equal elements may end up at
/// position `index`) and *O*(*n*) on average. The worst-case performance is *O*(*n* log *n*).
/// This function is also known as "kth element" in other libraries.
///
/// Returns a triplet of the following from the slice reordered according to the provided key
/// extraction function: the subslice prior to `index`, the element at `index`, and the subslice
/// after `index`; accordingly, the values in those two subslices will respectively all be
/// less-than-or-equal-to and greater-than-or-equal-to the value of the element at `index`.
///
/// # Implementation
///
/// The implementation is similar to `core::slice::select_nth_unstable_by`, but it uses an adaptive
/// pivot selection algorithm. This usually improves performance substantially, especially when
/// `index` is far from the median.
///
/// In the worst case, the algorithm allocates temporary storage in a `Vec<(K, usize)>` the
/// length of the slice.
///
/// # Examples
///
/// ```
/// use turboselect::select_nth_unstable_by_cached_key;
/// let mut v = [-5i32, 4, 32, -3, 2];
///
/// // Return the median as if the array were sorted according to absolute value.
/// select_nth_unstable_by_cached_key(&mut v, 2, |a| a.to_string());
///
/// // We are only guaranteed the slice will be one of the following, based on the way we sort
/// // about the specified index.
/// assert!(
///     v == [-3, -5, 2, 32, 4]
///         || v == [-5, -3, 2, 32, 4]
///         || v == [-3, -5, 2, 4, 32]
///         || v == [-5, -3, 2, 4, 32]
/// );
/// ```
#[inline]
pub fn select_nth_unstable_by_cached_key<T, K, F>(data: &mut [T], index: usize, f: F)
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    use std::vec::Vec;

    // Helper macro for indexing our vector by the smallest possible type, to reduce allocation.
    macro_rules! select_nth_by_key {
        ($t:ty, $slice:ident, $index:ident, $f:ident) => {{
            let mut indices: Vec<_> = $slice
                .iter()
                .map($f)
                .enumerate()
                .map(|(i, k)| (k, i as $t))
                .collect();
            // The elements of `indices` are unique, as they are indexed, so any sort will be
            // stable with respect to the original slice. We use `sort_unstable` here because
            // it requires less memory allocation.
            select_nth_unstable(&mut indices, index);
            for i in 0..$slice.len() {
                let mut index = indices[i].1;
                while (index as usize) < i {
                    index = indices[index as usize].1;
                }
                indices[i].1 = index;
                $slice.swap(i, index as usize);
            }
        }};
    }

    let sz_u8 = mem::size_of::<(K, u8)>();
    let sz_u16 = mem::size_of::<(K, u16)>();
    let sz_u32 = mem::size_of::<(K, u32)>();
    let sz_usize = mem::size_of::<(K, usize)>();

    let len = data.len();
    if len < 2 {
        return;
    }
    if sz_u8 < sz_u16 && len <= (u8::MAX as usize) {
        return select_nth_by_key!(u8, data, index, f);
    }
    if sz_u16 < sz_u32 && len <= (u16::MAX as usize) {
        return select_nth_by_key!(u16, data, index, f);
    }
    if sz_u32 < sz_usize && len <= (u32::MAX as usize) {
        return select_nth_by_key!(u32, data, index, f);
    }
    select_nth_by_key!(usize, data, index, f)
}

fn split_partition<T>(data: &mut [T], index: usize) -> (&mut [T], &mut T, &mut [T]) {
    let (left, rest) = data.split_at_mut(index);
    let (pivot, right) = rest.split_first_mut().unwrap();
    (left, pivot, right)
}

// Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
fn width<T>(l: *const T, r: *const T) -> usize {
    assert!(mem::size_of::<T>() > 0);
    // SAFETY: This is a helper function, refer to the usage of `ptr::offset_from` for
    // safety.
    unsafe { r.offset_from(l).max(0) as usize }
}
