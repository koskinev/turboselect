#![no_std]

#[cfg(feature = "std")]
#[cfg(test)]
mod benches;
mod sort;

#[cfg(feature = "std")]
#[cfg(test)]
mod tests;
mod wyrand;

use core::{
    array,
    cmp::{self, Ordering},
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut, Range},
    ptr,
};
use sort::{sort_at, tinysort};
use wyrand::WyRng;

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
        // SAFETY: This is a helper class. Please refer to its usage for correctness. Namely, one
        // must be sure that `src` and `dst` does not overlap as required by
        // `ptr::copy_nonoverlapping` and are both valid for writes.
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

/// Partitions `data` into two parts using the element at `index` as the pivot. Returns `(u, u)`,
/// where `u` is the number of elements less than the pivot, and the index of the pivot after
/// partitioning.
///
/// The resulting partitioning is as follows:
///
/// ```text
/// ┌─────────────────────────────┬──────────────────────────────┐
/// │ is_less(&data[x], &data[u]) │ !is_less(&data[x], &data[u]) │
/// └─────────────────────────────┴──────────────────────────────┘
///                                u        
/// ```
///
/// Panics if `index` is out of bounds.
fn partition_at<T>(data: &mut [T], index: usize) -> (usize, usize)
where
    T: Ord,
{
    // This ensures that the index is in bounds.
    data.swap(0, index);

    let (head, tail) = data.split_first_mut().unwrap();
    let (u, mut v);
    {
        // Read the pivot into the stack. The read below is safe, because the pivot is the first
        // element in the slice.
        let pivot = unsafe { Elem::new(head) };

        // Find the positions of the first pair of out-of-order elements.
        let (mut l, mut r) = (0, tail.len());
        unsafe {
            // The calls to get_unchecked are safe, because the slice is non-empty and we ensure l
            // <= r.
            while l < r && tail.get_unchecked(l) < &*pivot {
                l += 1;
            }
            while l < r && tail.get_unchecked(r - 1) >= &*pivot {
                r -= 1;
            }
        }
        u = l + partition_in_blocks(&mut tail[l..r], &*pivot);
        v = u;
        while v < tail.len() && unsafe { tail.get_unchecked(v) } == &*pivot {
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
/// ┌─────────────────────────────┬─────────────────────────────────┬─────────────────────────────┐
/// │ is_less(&data[x], &data[u]) │    !is_less(&data[x], &data[u]) │ is_less(&data[v], &data[x]) │
/// │                             │ && !is_less(&data[v], &data[x]) │                             │
/// └─────────────────────────────┴─────────────────────────────────┴─────────────────────────────┘
///                                u                               v
/// ```
///
/// Panics if `index` is out of bounds.
fn partition_equal<T>(data: &mut [T], index: usize) -> (usize, usize)
where
    T: Ord,
{
    let (u, v) = partition_at(data, index);
    let dups = partition_min(data[v..].as_mut(), 0).1;
    (u, v + dups)
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
fn partition_in_blocks<T>(data: &mut [T], pivot: &T) -> usize
where
    T: Ord,
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
                //         According to the conditions required by the function, we satisfy them
                // because:
                //         1. `offsets_l` is stack-allocated, and thus considered separate allocated
                //            object.
                //         2. The function `is_less` returns a `bool`. Casting a `bool` will never
                //            overflow `isize`.
                //         3. We have guaranteed that `block_l` will be `<= BLOCK`. Plus, `end_l`
                //            was initially set to the begin pointer of `offsets_` which was
                //            declared on the stack. Thus, we know that even in the worst case (all
                //            invocations of `is_less` returns false) we will only be at most 1 byte
                //            pass the end.
                //        Another unsafety operation here is dereferencing `elem`.
                //        However, `elem` was initially the begin pointer to the slice which is
                // always valid.
                unsafe {
                    // Branchless comparison.
                    *end_l = i as u8;
                    end_l = end_l.add((&*elem >= pivot) as usize);
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
                //         According to the conditions required by the function, we satisfy them
                // because:
                //         1. `offsets_r` is stack-allocated, and thus considered separate allocated
                //            object.
                //         2. The function `is_less` returns a `bool`. Casting a `bool` will never
                //            overflow `isize`.
                //         3. We have guaranteed that `block_r` will be `<= BLOCK`. Plus, `end_r`
                //            was initially set to the begin pointer of `offsets_` which was
                //            declared on the stack. Thus, we know that even in the worst case (all
                //            invocations of `is_less` returns true) we will only be at most 1 byte
                //            pass the end.
                //        Another unsafety operation here is dereferencing `elem`.
                //        However, `elem` was initially `1 * sizeof(T)` past the end and we
                // decrement it by `1 * sizeof(T)` before accessing it.        Plus,
                // `block_r` was asserted to be less than `BLOCK` and `elem` will therefore at most
                // be pointing to the beginning of the slice.
                unsafe {
                    // Branchless comparison.
                    elem = elem.sub(1);
                    *end_r = i as u8;
                    end_r = end_r.add((&*elem < pivot) as usize);
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

/// Puts the maximum elements at the end of the slice and returns the indices of the first and
/// last elements equal to the maximum. The `init` argument is the index of the element to use as
/// the initial maximum.
fn partition_max<T>(data: &mut [T], init: usize) -> (usize, usize)
where
    T: Ord,
{
    // SAFETY: `Reverse` has the same memory layout as `T`, so we can safely cast the slice to
    // `&mut [Reverse<T>]`.
    let rev: &mut [cmp::Reverse<T>] = unsafe { &mut *(data as *mut [T] as *mut [cmp::Reverse<T>]) };

    let len = rev.len();
    let (_, v) = partition_min(rev, init);
    let count = (v + 1).min(len - v - 1);

    let (head, right) = data.split_at_mut(len - count);
    let (left, _) = head.split_at_mut(count);
    left.swap_with_slice(right);
    (len - v - 1, len - 1)
}

/// Puts the minimum elements at the beginning of the slice and returns the indices of the first and
/// last elements equal to the minimum. The `init` argument is the index of the element to use as
/// the initial minimum.
fn partition_min<T>(data: &mut [T], init: usize) -> (usize, usize)
where
    T: Ord,
{
    // If there is only on element, it is the minimum.
    if data.len() < 2 {
        return (0, data.len() - 1);
    }

    // Initialize the minimum by scanning some elements.
    data.swap(0, init);
    sort_at(data, [0, data.len() - 1]);
    sort_at(data, [0, data.len() / 2]);
    sort_at(data, [0, 1]);

    // Copy the initial minimum to the stack
    let (head, tail) = data.split_first_mut().unwrap();
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
            // Scan the block and store offsets to the elements that satisfy `elem <= minimum`.
            for offset in 0..block {
                end.write(offset as u8);
                let is_le = *elem.add(offset) <= *min;
                end = end.add(is_le as usize);
            }
            // Scan the found elements
            for _ in 0..width(start, end) {
                let next = elem.add(*start as usize);
                match (*next).cmp(&*min) {
                    Ordering::Less => {
                        // We found a new minimum.
                        dup = l;
                        ptr::swap_nonoverlapping(next, &mut *min, 1);
                    }
                    Ordering::Equal => {
                        // We found an element equal to the minimum.
                        if width(l, dup) < width(l, next) {
                            ptr::swap_nonoverlapping(next, dup, 1);
                        }
                        dup = dup.add(1);
                    }
                    _ => {}
                }
                start = start.add(1);
            }
            elem = elem.add(block);
            start = offsets.as_mut_ptr().cast();
            end = start;
        }
    }
    (0, width(l, dup))
}

/// Partitions the slice so that elements in `data[..index]` are less than or equal to the pivot
/// and elements in `data[index..]` are greater than or equal to the pivot.
///
/// Panics if `index >= data.len()`.
fn turboselect<T>(mut data: &mut [T], mut index: usize, rng: &mut WyRng) -> (usize, usize)
where
    T: Ord,
{
    assert!(index < data.len());
    let mut offset = 0;
    let mut was = None;
    while data.len() > 8 {
        let (u, v) = match index {
            0 => partition_min(data, 0),
            i if i == data.len() - 1 => partition_max(data, 0),
            _ => {
                let (p, all_eq) = select_pivot(data, index, rng);
                match was {
                    // Test if the selected pivot is equal to a previous pivot from the left. In
                    // this case we know that the pivot is the minimum of the current slice.
                    Some(w) if w == &data[p] => partition_min(data, p),

                    // If the selected pivot is equal to it's neighbor elements, use ternary
                    // partitioning, which puts the elements equal to the pivot in the
                    // middle. This is necessary to ensure that the algorithm terminates.
                    _ if all_eq => partition_equal(data, p),

                    // Otherwise, use the default binary partioning.
                    _ => partition_at(data, p),
                }
            }
        };
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
            (data, was) = (tail, head.last());
            index -= v + 1;
            offset += v + 1;
        } else {
            return (offset + u, offset + v);
        }
    }
    tinysort(data);
    let u = index + offset;
    (u, u)
}

/// Samples `count` elements randomly and places them into the beginning of the slice. Returns the
/// sample as a slice. Panics if `count > data.len()` or `data.len() == 0`.
fn sample<'a, T>(data: &'a mut [T], count: usize, rng: &mut WyRng) -> &'a mut [T] {
    let len = data.len();
    assert!(count <= len);
    assert!(len > 0);
    unsafe {
        let ptr = data.as_mut_ptr();
        // Read the first element into a temporary location. The read is safe because `ptr` points
        // to the first element of `data` and `data` is non-empty.
        let tmp = ManuallyDrop::new(ptr::read(ptr));
        // Select a random element and swap it with the first element. The `src` pointer is in
        // bounds, because `rng.bounded_usize(0, len)` returns a value in the range `[0,
        // len)`.
        let (mut src, mut dst) = (ptr.add(rng.bounded_usize(0, len)), ptr);
        // Copy the element at `src` to `dst`. The copy is safe, because `src` and `dst` are in
        // bounds.
        ptr::copy(src, dst, 1);
        // Continue until `count` elements have been samples.
        for i in 1..count {
            // Select the next element. This is safe since `count <= len`.
            dst = dst.add(1);
            // See above for why this is safe.
            ptr::copy(dst, src, 1);
            src = ptr.add(rng.bounded_usize(i, len));
            ptr::copy(src, dst, 1);
        }
        // Write the temporary element (i.e the original first element) to the last sampled
        // position. This is safe, because `src` is in bounds.
        src.write(ManuallyDrop::into_inner(tmp));
        &mut data[..count]
    }
}

fn select_min<T, F>(data: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let (min, _) = data
        .iter()
        .enumerate()
        .min_by(|&(_, x), &(_, y)| {
            if is_less(x, y) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .unwrap();
    data.swap(0, min);
}

fn select_max<T, F>(data: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let (max, _) = data
        .iter()
        .enumerate()
        .max_by(|&(_, x), &(_, y)| {
            if is_less(x, y) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .unwrap();
    data.swap(max, data.len() - 1);
}

/// Reorder the slice such that the element at `index` is at its final sorted position.
///
/// This reordering has the additional property that any value at position `i < index` will be
/// less than or equal to any value at a position `j > index`. Additionally, this reordering is
/// unstable (i.e. any number of equal elements may end up at position `index`), in-place
/// (i.e. does not allocate), and *O*(*n*) on average. The worst-case performance is *O*(*n* log
/// *n*). This function is also known as "kth element" in other libraries.
///
/// Returns a triplet of the following from the reordered slice:
/// the subslice prior to `index`, the element at `index`, and the subslice after `index`;
/// accordingly, the values in those two subslices will respectively all be less-than-or-equal-to
/// and greater-than-or-equal-to the value of the element at `index`.
///
/// # The implementation
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
pub fn select_nth_unstable<T>(data: &mut [T], index: usize) -> (&mut [T], &mut T, &mut [T])
where
    T: Ord,
{
    #[cfg(not(debug_assertions))]
    // Use the address of the last element as the seed for the random number generator.
    let seed = data.as_mut_ptr() as u64 + data.len() as u64;

    #[cfg(debug_assertions)]
    let seed = 12345678901234567890;

    let mut rng = WyRng::new(seed);

    if index == 0 {
        select_min(data, &mut T::lt);
    } else if index == data.len() - 1 {
        select_max(data, &mut T::lt);
    } else {
        turboselect(data, index, rng.as_mut());
    }
    let (left, rest) = data.split_at_mut(index);
    let (pivot, right) = rest.split_first_mut().unwrap();
    (left, pivot, right)
}

/// Selects the pivot element for partitioning the slice. Returns `(p, r)` where `p` is the index
/// of the pivot element and `r` is number neighbor elements, used to test for equality.
fn select_pivot<T>(data: &mut [T], index: usize, rng: &mut WyRng) -> (usize, bool)
where
    T: Ord,
{
    match data.len() {
        // If the slice is small, select the pivot as median of five elements.
        // len if len < 32 => {
        //     sort_at(data, [0, len / 4, len / 2, 3 * len / 4, len - 1], is_less);
        //     (len / 2, !is_less(&data[0], &data[len / 2]))
        // }
        // For larger slices, select `N` depending on the slice length, sort N groups of N elements,
        // select group based on the index, sort the group and return the position of the
        // middle element.
        // len if len < 32 => randomize_pivot::<3, _, _>(data, index, is_less, rng),
        len if len < 256 => randomize_pivot::<3, _>(data, index, rng),
        len if len < 2048 => randomize_pivot::<5, _>(data, index, rng),
        len if len < 8192 => randomize_pivot::<7, _>(data, index, rng),
        len if len < 65536 => randomize_pivot::<11, _>(data, index, rng),
        len if len < 1048576 => randomize_pivot::<21, _>(data, index, rng),
        _ => randomize_pivot::<31, _>(data, index, rng),
    }
}

#[inline]
/// Chooses a randomized pivot for the given index. First, puts a `N * N` random sample to the
/// beginning of the slice. Then sorts `N` groups of `N` elements in the sample, each `N` elements
/// apart. Finally, sorts the group where the pivot is located. Returns `(u, all_eq)` where `u` is
/// the index of the selected pivot and `all_eq` is `true` if all elements in the selected group are
/// equal.
fn randomize_pivot<const N: usize, T>(
    data: &mut [T],
    index: usize,
    rng: &mut WyRng,
) -> (usize, bool)
where
    T: Ord,
{
    let len = data.len();
    let sample = sample(data, N * N, rng);
    let g = N * ((N * index) / len);
    for j in 0..N {
        let pos: [_; N] = array::from_fn(|i| j + N * i);
        sort_at(sample, pos);
    }
    let pos: [_; N] = array::from_fn(|i| g + i);
    sort_at(sample, pos);
    (g + N / 2, sample[g] == sample[g + N / 2])
}

// Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
fn width<T>(l: *mut T, r: *mut T) -> usize {
    assert!(mem::size_of::<T>() > 0);
    unsafe { r.offset_from(l).max(0) as usize }
}
