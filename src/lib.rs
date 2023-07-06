#[cfg(test)]
mod benches;
mod sort;
#[cfg(test)]
mod tests;
mod wyrand;

use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut, Range},
    ptr,
};
use sort::{median_at, sort_at, tinysort};
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
    /// Creates a new `Elem` from a mutable reference. This method can be safely used only if the
    /// caller ensures that the reference is not used for the duration of the `Elem`'s lifetime.
    unsafe fn new(src: *mut T) -> Self {
        Self {
            value: ManuallyDrop::new(ptr::read(src)),
            dst: src,
        }
    }
}

///
fn miniselect<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    fn select_min<T, F>(data: &mut [T], is_less: &mut F) -> (usize, usize)
    where
        F: FnMut(&T, &T) -> bool,
    {
        use core::cmp::Ordering::{Greater, Less};

        let (min, _) = data
            .iter()
            .enumerate()
            .min_by(|&(_, x), &(_, y)| if is_less(x, y) { Less } else { Greater })
            .unwrap();
        data.swap(0, min);
        (0, 0)
    }

    fn select_max<T, F>(data: &mut [T], is_less: &mut F) -> (usize, usize)
    where
        F: FnMut(&T, &T) -> bool,
    {
        use core::cmp::Ordering::{Greater, Less};

        let last = data.len() - 1;
        let (max, _) = data
            .iter()
            .enumerate()
            .max_by(|&(_, x), &(_, y)| if is_less(x, y) { Less } else { Greater })
            .unwrap();
        data.swap(max, last);
        (last, last)
    }

    if index == 0 {
        select_min(data, is_less)
    } else if index == data.len() - 1 {
        select_max(data, is_less)
    } else {
        match data.len() {
            len if len < 9 => tinysort(data, is_less),
            _ => {
                sort_at(data, [0, 1, 2, 3, 4], is_less);
                data.swap(index, 2);
                let mut l = data.as_mut_ptr();
                let (mut r, k) = unsafe { (l.add(data.len() - 1), l.add(index)) };
                while l < r {
                    let mut p = l;
                    let mut q = r;
                    if p == k {
                        let mut min = k;
                        while q > k {
                            unsafe {
                                if is_less(&*q, &*min) {
                                    min = q;
                                }
                                q = q.sub(1);
                            }
                        }
                        unsafe { k.swap(min) };
                        break;
                    }
                    if k == q {
                        let mut max = k;
                        while p < k {
                            unsafe {
                                if is_less(&*max, &*p) {
                                    max = p;
                                }
                                p = p.add(1);
                            }
                        }
                        unsafe { k.swap(max) };
                        break;
                    }
                    unsafe {
                        let x = ManuallyDrop::new(ptr::read(k));
                        loop {
                            while is_less(&*p, &x) {
                                p = p.add(1);
                            }
                            while is_less(&x, &*q) {
                                q = q.sub(1);
                            }
                            if p <= q {
                                p.swap(q);
                                p = p.add(1);
                                q = q.sub(1);
                            }
                            if p > q {
                                break;
                            }
                        }
                    }
                    if q < k {
                        l = p;
                    }
                    if k < p {
                        r = q;
                    }
                }
            }
        }
        (index, index)
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
fn partition_at<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // This ensures that the index is in bounds.
    data.swap(0, index);

    let (head, tail) = data.split_first_mut().unwrap();
    let u = {
        // Read the pivot into the stack. The read below is safe, because the pivot is the first
        // element in the slice.
        let pivot = unsafe { Elem::new(head) };

        // Find the positions of the first pair of out-of-order elements.
        let (mut l, mut r) = (0, tail.len());
        unsafe {
            // The calls to get_unchecked are safe, because the slice is non-empty and we ensure l
            // <= r.
            while l < r && is_less(tail.get_unchecked(l), &*pivot) {
                l += 1;
            }
            while l < r && !is_less(tail.get_unchecked(r - 1), &*pivot) {
                r -= 1;
            }
        }
        l + partition_in_blocks(&mut tail[l..r], &*pivot, is_less)
    };
    data.swap(0, u);
    (u, u)
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
fn partition_equal<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    data.swap(0, index);
    let (head, tail) = data.split_first_mut().unwrap();

    let (u, v) = {
        // Read the pivot into the stack.
        // SAFETY: `head` is not accessed again before the end of this block.
        let pivot = unsafe { Elem::new(head) };
        partition_in_blocks_dual(tail, &*pivot, &*pivot, is_less)
    };
    data.swap(0, u);
    (u, v)
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
fn partition_in_blocks<T, F>(data: &mut [T], pivot: &T, is_less: &mut F) -> usize
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

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        assert!(core::mem::size_of::<T>() > 0);
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
                    end_l = end_l.add(!is_less(&*elem, pivot) as usize);
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
                    end_r = end_r.add(is_less(&*elem, pivot) as usize);
                }
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = core::cmp::min(width(start_l, end_l), width(start_r, end_r));

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

/// Partitions `data` into elements smaller than `low`, followed by elements between `low` and
/// `high` and then elements greater than `high`.
///
/// Returns a tuple `(u, v)` where `u` is the number of elements smaller than `low` and `v` is the
/// number of elements smaller than or equal to `high`.
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
    const BLOCK: usize = 128;
    let Range { start: s, end: e } = data.as_mut_ptr_range();

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
    let mut l = s;
    let mut block_l = BLOCK;
    let mut start_l: *mut u8 = ptr::null_mut();
    let mut end_l: *mut u8 = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())`
    // is always safe`
    let mut r = e;
    let mut block_r = BLOCK;
    let mut start_r = ptr::null_mut();
    let mut end_r = ptr::null_mut();
    let mut offsets_r = [MaybeUninit::<u8>::uninit(); BLOCK];

    // `p` tracks the first element smaller than the lower pivot
    let mut p = l;
    // `q` tracks the element after the last element greater than the higher pivot
    let mut q = r;

    // FIXME: When we get VLAs, try creating one array of length `min(v.len(), 2 * BLOCK)` rather
    // than two fixed-size arrays of length `BLOCK`. VLAs might be more cache-efficient.

    /// Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
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
                // 2. The function `is_less` returns a `bool`. Casting a `bool` will never overflow
                //    `isize`.
                // 3. We have guaranteed that `block_l` will be `<= BLOCK`. Plus, `end_l` was
                //    initially set to the begin pointer of `offsets_` which was declared on the
                //    stack.Thus, we know that even in the worst case (all  invocations of `is_less`
                //    returns false) we will only be at most 1 byte pass the end.
                // Another unsafety operation here is dereferencing `elem`. However, `elem` was
                // initially the begin pointer to the slice which is always valid.
                unsafe {
                    // Branchless comparison.
                    *end_l = i as u8;
                    end_l = end_l.add(!is_less(&*elem, low) as usize);
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
                //
                // 1. `offsets_r` is stack-allocated, and thus considered separate allocated object.
                // 2. The function `is_less` returns a `bool`. Casting a `bool` will never overflow
                //    `isize`.
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
                    elem = elem.sub(1);
                    *end_r = i as u8;
                    end_r = end_r.add(!is_less(high, &*elem) as usize);
                }
            }
        }

        // Number of out-of-order elements to swap between the left and right side.
        let count = core::cmp::min(width(start_l, end_l), width(start_r, end_r));

        if count > 0 {
            macro_rules! left {
                () => {
                    l.add(*start_l as usize)
                };
            }
            macro_rules! right {
                () => {
                    r.sub(*start_r as usize + 1)
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
                let mut mid_l = start_l;
                let mut mid_r = start_r;
                let tmp = ptr::read(left!());

                *mid_l = *start_l;
                mid_l = mid_l.add(!is_less(&*right!(), low) as usize);
                ptr::copy_nonoverlapping(right!(), left!(), 1);
                for _ in 1..count {
                    start_l = start_l.add(1);
                    *mid_r = *start_r;
                    mid_r = mid_r.add(!is_less(high, &*left!()) as usize);
                    ptr::copy_nonoverlapping(left!(), right!(), 1);

                    start_r = start_r.add(1);
                    *mid_l = *start_l;
                    mid_l = mid_l.add(!is_less(&*right!(), low) as usize);
                    ptr::copy_nonoverlapping(right!(), left!(), 1);
                }
                *mid_r = *start_r;
                mid_r = mid_r.add(!is_less(high, &tmp) as usize);
                ptr::copy_nonoverlapping(&tmp, right!(), 1);
                core::mem::forget(tmp);

                start_l = start_l.add(1);
                start_r = start_r.add(1);

                let count_l = width(start_l.sub(count), mid_l);
                mid_l = start_l.sub(count);
                for _ in 0..count_l {
                    ptr::swap(l.add(*mid_l as usize), p);
                    mid_l = mid_l.add(1);
                    p = p.add(1);
                }

                let count_r = width(start_r.sub(count), mid_r);
                mid_r = start_r.sub(count);
                for _ in 0..count_r {
                    ptr::swap(r.sub(*mid_r as usize + 1), q.sub(1));
                    mid_r = mid_r.add(1);
                    q = q.sub(1);
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
                ptr::swap(l.add(*end_l as usize), r.sub(1));
                // Move the elements that should go to the middle to the extreme right.
                if !is_less(high, &*r.sub(1)) {
                    ptr::swap(r.sub(1), q.sub(1));
                    q = q.sub(1);
                }
                r = r.sub(1);
            }
        }
        if r > s {
            l = unsafe { r.sub(1) };
        }
    } else if start_r < end_r {
        // The right block remains.
        // Move its remaining out-of-order elements to the far left.
        debug_assert_eq!(width(l, r), block_r);
        while start_r < end_r {
            // SAFETY: See the reasoning in [remaining-elements-safety].
            unsafe {
                end_r = end_r.sub(1);
                ptr::swap(l, r.sub(*end_r as usize + 1));
                // Move the elements that should go to the middle to the extreme left.
                if !is_less(&*l, low) {
                    ptr::swap(l, p);
                    p = p.add(1);
                }
                l = l.add(1);
            }
        }
        if l < e {
            r = unsafe { l.add(1) };
        }
    } else {
        // Nothing else to do, we're done.
        if l < e {
            r = unsafe { l.add(1) };
        }
    }

    unsafe {
        if l < e && is_less(&*l, low) {
            l = l.add(1);
        }
        if r > s && is_less(high, &*r.sub(1)) {
            r = r.sub(1);
        }
    }

    // Move the temporary partition in the beginning of the slice to the middle.
    let (a, b) = (saturating_width(p, l), width(s, p));
    for offset in 0..core::cmp::min(a, b) {
        unsafe {
            l = l.sub(1);
            ptr::swap_nonoverlapping(s.add(offset), l, 1);
        }
    }

    // Move the temporary partition in the end of the slice to the middle.
    let (c, d) = (saturating_width(r, q), width(q, e));
    for offset in 0..core::cmp::min(c, d) {
        unsafe {
            ptr::swap_nonoverlapping(r, e.sub(offset + 1), 1);
            r = r.add(1);
        }
    }
    let (u, v) = (a, data.len() - c);
    (u, v)
}

/// Puts the maximum elements at the end of the slice and returns the indices of the first and
/// last elements equal to the maximum. The `init` argument is the index of the element to use as
/// the initial maximum.
fn partition_max<T, F>(data: &mut [T], init: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let (_, v) = partition_min(data, init, &mut |x, y| is_less(y, x));
    let len = data.len();
    let count = (v + 1).min(len - v - 1);
    let (head, tail) = data.split_at_mut(len - count);
    unsafe { ptr::swap_nonoverlapping(tail.as_mut_ptr(), head.as_mut_ptr(), count) };
    (len - v - 1, len - 1)
}

/// Puts the minimum elements at the beginning of the slice and returns the indices of the first and
/// last elements equal to the minimum. The `init` argument is the index of the element to use as
/// the initial minimum.
fn partition_min<T, F>(data: &mut [T], init: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // If there is only one element, it is the minimum.
    if data.len() < 2 {
        return (0, data.len() - 1);
    }

    // Initialize the minimum by scanning some elements.
    data.swap(0, init);
    sort_at(data, [0, data.len() - 1], is_less);
    sort_at(data, [0, data.len() / 2], is_less);
    sort_at(data, [0, data.len() / 3], is_less);

    // Copy the initial minimum to the stack
    let (head, tail) = data.split_first_mut().unwrap();
    let mut min = unsafe { Elem::new(head) };
    let mut dups = 0;

    let Range { start: l, end: r } = tail.as_mut_ptr_range();
    let mut elem = l;

    // Setup the offsets array.
    const BLOCK: usize = 64;
    let mut offsets = [MaybeUninit::<u8>::uninit(); BLOCK];
    let mut start = offsets.as_mut_ptr().cast();
    let mut end: *mut u8 = start;

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        assert!(core::mem::size_of::<T>() > 0);
        // FIXME: this should *likely* use `offset_from`, but more
        // investigation is needed (including running tests in miri).
        unsafe { r.offset_from(l) as usize }
    }

    while elem < r {
        // Scan the next block.
        let block = core::cmp::min(BLOCK, width(elem, r));
        unsafe {
            // Scan the block and store offsets to the elements less than or equal <= minimum.
            for offset in 0..block {
                end.write(offset as u8);
                let is_le = !is_less(&*min, &*elem.add(offset));
                end = end.add(is_le as usize);
            }
            // Scan the found elements
            for _ in 0..width(start, end) {
                let offset = start.read() as usize;
                if is_less(&*elem.add(offset), &*min) {
                    // We found a new minimum.
                    dups = 0;
                    ptr::swap_nonoverlapping(elem.add(offset), &mut *min, 1);
                } else if !is_less(&*min, &*elem.add(offset)) {
                    // We found an element equal to the minimum.
                    if dups < elem.add(offset).offset_from(l) as usize {
                        ptr::swap_nonoverlapping(elem.add(offset), l.add(dups), 1);
                    }
                    dups += 1;
                }
                start = start.add(1);
            }
            elem = elem.add(block);
            start = offsets.as_mut_ptr().cast();
            end = start;
        }
    }
    (0, dups)
}

/// Partitions the slice so that elements in `data[..index]` are less than or equal to the pivot
/// and elements in `data[index..]` are greater than or equal to the pivot.
///
/// Panics if `index >= data.len()`.
fn quickselect<T, F>(
    mut data: &mut [T],
    mut index: usize,
    is_less: &mut F,
    rng: &mut WyRng,
) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(index < data.len());
    let mut offset = 0;
    let mut was = None;
    while data.len() > 24 {
        let (u, v) = match index {
            0 => partition_min(data, 0, is_less),
            i if i == data.len() - 1 => partition_max(data, 0, is_less),
            _ => {
                let (p, all_eq) = select_pivot(data, index, is_less, rng);
                match was {
                    // Test if the selected pivot is equal to a previous pivot from the left. In
                    // this case we know that the pivot is the minimum of the current slice.
                    Some(w) if !is_less(w, &data[p]) => partition_min(data, p, is_less),

                    // If the selected pivot is equal to it's neighbor elements, use ternary
                    // partitioning, which puts the elements equal to the pivot in the
                    // middle. This is necessary to ensure that the algorithm terminates.
                    _ if all_eq => partition_equal(data, p, is_less),

                    // Otherwise, use the default binary partioning.
                    _ => partition_at(data, p, is_less),
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
    miniselect(data, index, is_less);
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

    quickselect(data, index, &mut T::lt, rng.as_mut());

    let (left, rest) = data.split_at_mut(index);
    let (pivot, right) = rest.split_first_mut().unwrap();
    (left, pivot, right)
}

/// Selects the pivot element for partitioning the slice. Returns `(p, r)` where `p` is the index
/// of the pivot element and `r` is number neighbor elements, used to test for equality.
fn select_pivot<T, F>(
    data: &mut [T],
    index: usize,
    is_less: &mut F,
    rng: &mut WyRng,
) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    match data.len() {
        // If the slice is small, use median of three.
        len if len < 32 => {
            let p = len / 2;
            sort_at(data, [0, p, len - 1], is_less);
            (p, !is_less(&data[0], &data[len - 1]))
        }
        // For slightly larger slices, use the median of 5 elements.
        len if len < 128 => {
            let p = len / 2;
            median_at(data, [0, p / 2, p, p + p / 2, len - 1], is_less);
            (p, !is_less(&data[p - 1], &data[p + 1]))
        }
        // For slices of size 128 to 1024, sort 5 groups of 5 elements each, then select the
        // group based on the index, sort the group and return the position of the middle element.
        len if len < 1024 => {
            let s = len / 5;
            let g = s * ((5 * index) / len);
            sort_at(data, [0, s, 2 * s, 3 * s, 4 * s], is_less);
            sort_at(data, [1, s + 1, 2 * s + 1, 3 * s + 1, 4 * s + 1], is_less);
            sort_at(data, [2, s + 2, 2 * s + 2, 3 * s + 2, 4 * s + 2], is_less);
            sort_at(data, [3, s + 3, 2 * s + 3, 3 * s + 3, 4 * s + 3], is_less);
            sort_at(data, [4, s + 4, 2 * s + 4, 3 * s + 4, 4 * s + 4], is_less);

            // The pivot is the middle element of the selected group
            sort_at(data, [g, g + 1, g + 2, g + 3, g + 4], is_less);
            (g + 2, !is_less(&data[g], &data[g + 4]))
        }
        // For larger slices, a similar technique is used, but with randomly sampled elements and
        // larger group sizes.
        len if len < 4096 => select_pivot_randomized::<5, _, _>(data, index, is_less, rng),
        len if len < 65536 => select_pivot_randomized::<9, _, _>(data, index, is_less, rng),
        _ => select_pivot_randomized::<21, _, _>(data, index, is_less, rng),
    }
}

#[inline]
/// Selects a pivot for the given index. First, puts a `N * N` random sample to the beginning
/// of the slice. Then sorts `N` groups of `N` elements in the sample, each `N` elements apart.
/// Finally, sorts the group where the pivot is located. Returns `(u, all_eq)` where `u` is the
/// index of the selected pivot and `all_eq` is `true` if all elements in the selected group are
/// equal.
fn select_pivot_randomized<const N: usize, T, F>(
    data: &mut [T],
    index: usize,
    is_less: &mut F,
    rng: &mut WyRng,
) -> (usize, bool)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = data.len();
    let sample = sample(data, N * N, rng);
    let g = N * ((N * index) / len);
    for j in 0..N {
        let pos: [_; N] = core::array::from_fn(|i| j + N * i);
        sort_at(sample, pos, is_less);
    }
    let pos: [_; N] = core::array::from_fn(|i| g + i);
    sort_at(sample, pos, is_less);
    (g + N / 2, !is_less(&sample[g], &sample[g + N / 2]))
}