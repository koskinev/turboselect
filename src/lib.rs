#[cfg(test)]
mod benches;
mod sort;
#[cfg(test)]
mod tests;
mod wyrand;

use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};
use sort::{median, sort};
use wyrand::WyRng;

// When dropped, copies from `src` into `dest`.
struct InsertionHole<T> {
    src: *const T,
    dest: *mut T,
}

impl<T> Drop for InsertionHole<T> {
    fn drop(&mut self) {
        // SAFETY: This is a helper class. Please refer to its usage for correctness. Namely, one
        // must be sure that `src` and `dst` does not overlap as required by
        // `ptr::copy_nonoverlapping` and are both valid for writes.
        unsafe {
            ptr::copy_nonoverlapping(self.src, self.dest, 1);
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
fn partition_at_index<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // This ensures that the index is in bounds.
    data.swap(0, index);

    let (head, tail) = data.split_first_mut().unwrap();
    let u = {
        // Read the pivot into the stack. The read below is safe, because the pivot is the first
        // element in the slice.
        let tmp = unsafe { ManuallyDrop::new(ptr::read(head)) };
        let _pivot_guard = InsertionHole {
            src: &*tmp,
            dest: head,
        };
        let pivot = &*tmp;

        // Find the positions of the first pair of out-of-order elements.
        let (mut l, mut r) = (0, tail.len());
        unsafe {
            // The calls to get_unchecked are safe, because the slice is non-empty and we ensure l
            // <= r.
            while l < r && is_less(tail.get_unchecked(l), pivot) {
                l += 1;
            }
            while l < r && !is_less(tail.get_unchecked(r - 1), pivot) {
                r -= 1;
            }
        }
        l + partition_in_blocks(&mut tail[l..r], pivot, is_less)
    };
    data.swap(0, u);
    (u, u)
}

/// Partitions `data` into three parts using the element at `index` as the pivot. Returns `(u, v)`,
/// where `u` is the number of elements less than the pivot, and `v` is the number of elements less
/// than or equal to the pivot.
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
fn partition_at_index_eq<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    data.swap(0, index);
    let (head, tail) = data.split_first_mut().unwrap();

    let (u, v) = {
        // Read the pivot into the stack. The read below is safe, because the pivot is the first
        // element in the slice.
        let tmp = unsafe { ManuallyDrop::new(ptr::read(head)) };
        let _pivot_guard = InsertionHole {
            src: &*tmp,
            dest: head,
        };
        let pivot = &*tmp;

        partition_in_blocks_dual(tail, pivot, pivot, is_less)
    };
    data.swap(0, u);
    (u, v)
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
    // 4. `offsets` - Indices of out-of-order elements within the block.

    // The current block on the left side (from `l` to `l.add(block_l)`).
    let mut l = v.as_mut_ptr();
    let mut block_l = BLOCK;
    let mut start_l = ptr::null_mut();
    let mut end_l = ptr::null_mut();
    let mut offsets_l = [MaybeUninit::<u8>::uninit(); BLOCK];

    // The current block on the right side (from `r.sub(block_r)` to `r`).
    // SAFETY: The documentation for .add() specifically mention that `vec.as_ptr().add(vec.len())`
    // is always safe
    let mut r = unsafe { l.add(v.len()) };
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
        width(v.as_mut_ptr(), r)
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
        width(v.as_mut_ptr(), l)
    } else {
        // Nothing else to do, we're done.
        width(v.as_mut_ptr(), l)
    }
}

/// Partitions `v` into elements smaller than `low`, followed by elements between `low` and `high`
/// and then elements greater than `high`.
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
    let s = data.as_mut_ptr();
    let e = unsafe { s.add(data.len()) };

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

    // unsafe {
    //     while p < e && !is_less(&*p, low) && !is_less(low, &*p) {
    //         p = p.add(1);
    //     }
    //     while q.sub(1) > p && !is_less(high, &*(q.sub(1))) && !is_less(&*(q.sub(1)), high) {
    //         q = q.sub(1);
    //     }
    // }
    // l = p;
    // r = q;

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

                // Test if the moved elements should go to the middle.
                for _ in 0..count {
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
                // Move the elements that should go to the middle to the extreme right.
                if !is_less(high, &*r.offset(-1)) {
                    ptr::swap(r.offset(-1), q.offset(-1));
                    q = q.offset(-1);
                }
                r = r.offset(-1);
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
                end_r = end_r.offset(-1);
                ptr::swap(l, r.offset(-(*end_r as isize) - 1));
                // Move the elements that should go to the middle to the extreme left.
                if !is_less(&*l, low) {
                    ptr::swap(l, p);
                    p = p.offset(1);
                }
                l = l.offset(1);
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

/// Puts the minimum elements at the beginning of the slice and returns the indices of the first and
/// last elements equal to the minimum.
fn partition_min<T, F>(data: &mut [T], is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // Number of elements in a typical block.
    const BLOCK: usize = 64;

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        assert!(core::mem::size_of::<T>() > 0);
        // FIXME: this should *likely* use `offset_from`, but more
        // investigation is needed (including running tests in miri).
        unsafe { r.offset_from(l) as usize }
    }

    // The index of the last element that is equal to the minimum element.
    let min = data.as_mut_ptr();
    let mut elem = unsafe { min.add(1) };
    let stop = unsafe { min.add(data.len()) };
    let mut offsets = [MaybeUninit::<u8>::uninit(); BLOCK];
    let mut start = offsets.as_mut_ptr().cast();
    let mut end: *mut u8 = start;
    let mut dups = 0;

    while elem < stop {
        // Scan the next block.
        let block = core::cmp::min(BLOCK, width(elem, stop));
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
                    ptr::swap_nonoverlapping(elem.add(offset), min, 1);
                } else if !is_less(&*min, &*elem.add(offset)) {
                    // We found an element equal to the minimum.
                    dups += 1;
                    if dups < elem.add(offset).offset_from(min) as usize {
                        ptr::swap_nonoverlapping(elem.add(offset), min.add(dups), 1);
                    }
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

/// Puts the maximum elements at the end of the slice and returns the indices of the first and
/// last elements equal to the maximum.
fn partition_max<T, F>(data: &mut [T], is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    // Number of elements in a typical block.
    const BLOCK: usize = 64;

    // Returns the number of elements between pointers `l` (inclusive) and `r` (exclusive).
    fn width<T>(l: *mut T, r: *mut T) -> usize {
        assert!(core::mem::size_of::<T>() > 0);
        unsafe { r.offset_from(l) as usize }
    }

    let max = data.as_mut_ptr();
    let mut elem = unsafe { max.add(1) };
    let stop = unsafe { max.add(data.len()) };
    let mut offsets = [MaybeUninit::<u8>::uninit(); BLOCK];
    let start = offsets.as_mut_ptr().cast();
    let mut end: *mut u8 = start;
    let last = data.len() - 1;
    let mut dups = 0;

    unsafe {
        while elem < stop.sub(dups) {
            // Scan the next block.
            let block = core::cmp::min(BLOCK, width(elem, stop.sub(dups)));
            // Scan the block and store offsets to the elements greater than or equal to the current
            // maximum.
            for offset in 0..block {
                end.write(offset as u8);
                let is_ge = !is_less(&*elem.add(offset), &*max);
                end = end.add(is_ge as usize);
            }
            // Scan the found elements
            for _ in 0..width(start, end) {
                end = end.sub(1);
                let offset = end.read() as usize;
                if is_less(&*max, &*elem.add(offset)) {
                    // We found a new maximum.
                    dups = 0;
                    ptr::swap_nonoverlapping(elem.add(offset), max, 1);
                } else if !is_less(&*elem.add(offset), &*max) {
                    // We found an element equal to the maximum
                    dups += 1;
                    if dups < max.offset_from(elem.add(offset)) as usize {
                        ptr::swap_nonoverlapping(elem.add(offset), stop.sub(dups), 1);
                    }
                }
            }
            elem = elem.add(block);
        }
        // Place the maximum elements at the end of the slice.
        ptr::swap_nonoverlapping(max, stop.sub(dups + 1), 1);
    }
    (last - dups, last)
}

/// Partitions the slice so that elements in `data[..index]` are less than or equal to the pivot
/// and elements in `data[index..]` are greater than or equal to the pivot.
///
/// Panics if `index >= data.len()`.
fn quickselect<T, F>(mut data: &mut [T], mut index: usize, is_less: &mut F, rng: &mut WyRng)
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(index < data.len());
    let mut was: Option<&T> = None;
    loop {
        let len = data.len();
        let (u, v) = match (index, len) {
            // If the first element is needed, select the minimum.
            (0, _) => partition_min(data, is_less),

            // If the last element is needed, select the maximum.
            (i, _) if i == len - 1 => partition_max(data, is_less),

            // Unless the slice is very small, select a pivot and partition the slice.
            (_, 6..) => {
                let p = select_pivot(data, index, is_less, rng);
                if was.take().filter(|w| !is_less(w, &data[p])).is_some() {
                    data.swap(p, 0);
                    partition_min(data, is_less)
                } else if is_less(&data[p - 1], &data[p + 1]) {
                    partition_at_index(data, p, is_less)
                } else {
                    // If the element before the pivot is equal to the pivot, use the ternary
                    // partitioning algorithm, which puts the elements equal to the pivot in the
                    // middle. This is necessary to ensure that the algorithm terminates.
                    partition_at_index_eq(data, p, is_less)
                }
            }
            // For very small slices, use sorting networks.
            (i, 5) => {
                sort(data, [0, 1, 2, 3, 4], is_less);
                (i, i)
            }
            (i, 4) => {
                sort(data, [0, 1, 2, 3], is_less);
                (i, i)
            }
            (i, 3) => {
                sort(data, [0, 1, 2], is_less);
                (i, i)
            }
            (i, 2) => {
                sort(data, [0, 1], is_less);
                (i, i)
            }
            (i, _) => (i, i),
        };
        // Recurse into the appropriate part of the slice or terminate if the pivot is in the
        // correct position.
        if index < u {
            (data, _) = data.split_at_mut(u);
        } else if index > v {
            index -= v + 1;
            let (head, tail) = data.split_at_mut(v + 1);
            (data, was) = (tail, head.last());
        } else {
            return;
        }
    }
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
    let mut rng = WyRng::new(0);
    quickselect(data, index, &mut T::lt, rng.as_mut());
    let (left, rest) = data.split_at_mut(index);
    let (pivot, right) = rest.split_first_mut().unwrap();
    (left, pivot, right)
}

/// Selects the pivot element for partitioning the slice. Returns the position of the pivot.
fn select_pivot<T, F>(data: &mut [T], index: usize, is_less: &mut F, rng: &mut WyRng) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    match data.len() {
        // If the slice is small, just use the middle element as the pivot.
        len if len < 32 => len / 2,
        // For slightly larger slices, use the median of 5 elements.
        len if len < 128 => {
            // The elements are s positions apart.
            let s = len / 5;
            median(data, [0, s, 2 * s, 3 * s, 4 * s], is_less);
            2 * s
        }
        // For slices of size 128 to 1024, sort 5 groups of 5 elements each, then select the
        // group based on the index, sort the group and return the position of the middle element.
        len if len < 1024 => {
            let s = len / 5;
            // Each group is `s` elements apart.
            for j in 0..5 {
                sort(data, [j, s + j, 2 * s + j, 3 * s + j, 4 * s + j], is_less);
            }
            // The pivot is the middle element of the group at position k.
            let k = s * ((5 * index) / len);
            sort(data, [k, k + 1, k + 2, k + 3, k + 4], is_less);
            k + 2
        }
        // For larger slices, a similar technique is used, but with randomly sampled elements and
        // larger group sizes.
        len if len < 4096 => {
            let sample = sample(data, 25, rng);
            for j in 0..5 {
                sort(sample, [j, j + 5, j + 10, j + 15, j + 20], is_less);
            }
            let p = 5 * ((5 * index) / len);
            sort(sample, [p, p + 1, p + 2, p + 3, p + 4], is_less);
            p + 2
        }
        len if len < 65536 => {
            let sample = sample(data, 81, rng);
            for j in 0..9 {
                sort(
                    sample,
                    [
                        j,
                        j + 9,
                        j + 18,
                        j + 27,
                        j + 36,
                        j + 45,
                        j + 54,
                        j + 63,
                        j + 72,
                    ],
                    is_less,
                );
            }
            let p = 9 * ((9 * index) / len);
            sort(
                sample,
                [p, p + 1, p + 2, p + 3, p + 4, p + 5, p + 6, p + 7, p + 8],
                is_less,
            );
            p + 4
        }
        len => {
            let sample = sample(data, 441, rng);
            for j in 0..21 {
                sort(
                    sample,
                    [
                        j,
                        j + 21,
                        j + 42,
                        j + 63,
                        j + 84,
                        j + 105,
                        j + 126,
                        j + 147,
                        j + 168,
                        j + 189,
                        j + 210,
                        j + 231,
                        j + 252,
                        j + 273,
                        j + 294,
                        j + 315,
                        j + 336,
                        j + 357,
                        j + 378,
                        j + 399,
                        j + 420,
                    ],
                    is_less,
                );
            }
            let p = 21 * ((21 * index) / len);
            sort(
                sample,
                [
                    p,
                    p + 1,
                    p + 2,
                    p + 3,
                    p + 4,
                    p + 5,
                    p + 6,
                    p + 7,
                    p + 8,
                    p + 9,
                    p + 10,
                    p + 11,
                    p + 12,
                    p + 13,
                    p + 14,
                    p + 15,
                    p + 16,
                    p + 17,
                    p + 18,
                    p + 19,
                    p + 20,
                ],
                is_less,
            );
            p + 10
        }
    }
}
