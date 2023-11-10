use core::mem::ManuallyDrop;

/// Represents an element that has been removed from the heap, leaving a hole. When dropped, an
/// `Elem` will fill the hole position with the value that was originally removed.
pub(crate) struct Elem<'a, T: 'a> {
    data: &'a mut [T],
    value: core::mem::MaybeUninit<T>,
    index: usize,
}

impl<'a, T> Elem<'a, T> {
    /// Returns a reference to the element.
    #[inline]
    pub(crate) fn as_ref(&self) -> &T {
        unsafe { self.value.assume_init_ref() }
    }

    /// Moves the hole to a new location.
    ///
    /// Unsafe because index must be within the data slice and not equal to pos.
    #[inline]
    pub(crate) unsafe fn move_back(&mut self) {
        debug_assert!(self.index > 0);
        unsafe {
            let ptr = self.data.as_mut_ptr().add(self.index);
            core::ptr::copy_nonoverlapping(ptr, ptr.sub(1), 1);
        }
        self.index -= 1;
    }

    /// Create a new `Elem` from the element at `index`.
    ///
    /// Panics if the index is out of bounds.
    #[inline]
    pub(crate) fn new(data: &'a mut [T], index: usize) -> Self {
        assert!(index < data.len());
        // Safety: we just checked that the index is within the slice.
        let value = unsafe { core::ptr::read(data.get_unchecked(index)) };
        Elem {
            data,
            value: core::mem::MaybeUninit::new(value),
            index,
        }
    }

    /// Returns a reference to the element before `self.index`.
    ///
    /// Unsafe because index must > 0.
    #[inline]
    pub(crate) unsafe fn prev(&self) -> &T {
        debug_assert!(self.index > 0);
        unsafe { self.data.get_unchecked(self.index - 1) }
    }
}

impl<T> Drop for Elem<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Safety: This fills the hole with the value that was originally removed. The caller must
        // ensure that hole position is valid.
        unsafe {
            let pos = self.index;
            core::ptr::copy_nonoverlapping(
                &*self.value.as_ptr(),
                self.data.get_unchecked_mut(pos),
                1,
            );
        }
    }
}

#[inline]
/// Compares the elements at `a` and `b` and swaps them if `a` is greater than `b`. Returns `true`
/// if the elements were swapped.
fn sort2<T, F>(data: &mut [T], a: usize, b: usize, lt: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(a != b);
    debug_assert!(a < data.len());
    debug_assert!(b < data.len());

    unsafe {
        let ptr = data.as_mut_ptr();
        let (a, b) = (a as isize, b as isize);
        let swap = lt(&*ptr.offset(b), &*ptr.offset(a)) as isize;
        let max = ptr.offset(swap * a + (1 - swap) * b);
        let min = ptr.offset(swap * b + (1 - swap) * a);
        let tmp = ManuallyDrop::new(core::ptr::read(max));
        ptr.offset(a).copy_from(min, 1);
        ptr.offset(b).write(ManuallyDrop::into_inner(tmp));
        swap != 0
    }
}

#[rustfmt::skip]
/// Sorts the elements at the positions in `pos` so that each element at `pos[i]` is less than or 
/// equal to the element at `pos[j]` if `i < j`.
/// 
/// The sorting networks are from https://bertdobbelaere.github.io/sorting_networks.html
pub(crate) fn sort_at<T, F, const N: usize>(data: &mut [T], pos: [usize; N], lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            sort2(data, pos[$a], pos[$b], lt);
        };
    }
    match N {
        2 => {
            sort2!(0, 1); 
        }
        3 => {
            sort2!(0, 2); sort2!(0, 1); sort2!(1, 2); 
        }
        4 => {
            sort2!(0, 2); sort2!(1, 3); sort2!(0, 1); sort2!(2, 3); sort2!(1, 2); 
            
        }
        5 => {
            sort2!(0, 3); sort2!(1, 4); sort2!(0, 2); sort2!(1, 3); sort2!(0, 1); 
            sort2!(2, 4); sort2!(1, 2); sort2!(3, 4); sort2!(2, 3); 
        }
        6 => {
            sort2!(1, 3); sort2!(2, 4); sort2!(0, 5); sort2!(1, 2); sort2!(3, 4); 
            sort2!(0, 3); sort2!(2, 5); sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); 
            sort2!(1, 2); sort2!(3, 4); 
        }
        7 => {
            sort2!(2, 3); sort2!(4, 5); sort2!(0, 6); sort2!(0, 2); sort2!(1, 4); 
            sort2!(3, 6); sort2!(0, 1); sort2!(3, 4); sort2!(2, 5); sort2!(1, 2); 
            sort2!(4, 6); sort2!(2, 3); sort2!(4, 5); sort2!(1, 2); sort2!(3, 4); 
            sort2!(5, 6); 
        }
        8 => {
            sort2!(0, 2); sort2!(1, 3); sort2!(4, 6); sort2!(5, 7); sort2!(0, 4); 
            sort2!(1, 5); sort2!(2, 6); sort2!(3, 7); sort2!(0, 1); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(2, 4); sort2!(3, 5); sort2!(1, 4); 
            sort2!(3, 6); sort2!(1, 2); sort2!(3, 4); sort2!(5, 6); 
        }
        9 => {
            sort2!(0, 3); sort2!(2, 5); sort2!(1, 7); sort2!(4, 8); sort2!(2, 4); 
            sort2!(5, 6); sort2!(0, 7); sort2!(3, 8); sort2!(0, 2); sort2!(1, 3); 
            sort2!(4, 5); sort2!(7, 8); sort2!(1, 4); sort2!(3, 6); sort2!(5, 7); 
            sort2!(0, 1); sort2!(2, 4); sort2!(3, 5); sort2!(6, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(1, 2); sort2!(3, 4); sort2!(5, 6); 
            
        }
        10 => {
            sort2!(3, 5); sort2!(4, 6); sort2!(2, 7); sort2!(0, 8); sort2!(1, 9); 
            sort2!(0, 2); sort2!(1, 4); sort2!(5, 8); sort2!(7, 9); sort2!(0, 3); 
            sort2!(2, 4); sort2!(5, 7); sort2!(6, 9); sort2!(0, 1); sort2!(3, 6); 
            sort2!(8, 9); sort2!(2, 3); sort2!(1, 5); sort2!(6, 7); sort2!(4, 8); 
            sort2!(1, 2); sort2!(3, 5); sort2!(4, 6); sort2!(7, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(3, 4); sort2!(5, 6); 
        }
        11 => {
            sort2!(2, 4); sort2!(1, 6); sort2!(3, 7); sort2!(5, 8); sort2!(0, 9); 
            sort2!(0, 1); sort2!(3, 5); sort2!(7, 8); sort2!(6, 9); sort2!(4, 10); 
            sort2!(1, 3); sort2!(2, 5); sort2!(4, 7); sort2!(8, 10); sort2!(1, 2); 
            sort2!(0, 4); sort2!(3, 7); sort2!(6, 8); sort2!(5, 9); sort2!(0, 1); 
            sort2!(4, 5); sort2!(2, 6); sort2!(7, 8); sort2!(9, 10); sort2!(2, 4); 
            sort2!(3, 6); sort2!(5, 7); sort2!(8, 9); sort2!(1, 2); sort2!(3, 4); 
            sort2!(5, 6); sort2!(7, 8); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); 
            
        }
        12 => {
            sort2!(2, 6); sort2!(1, 7); sort2!(0, 8); sort2!(5, 9); sort2!(4, 10); 
            sort2!(3, 11); sort2!(0, 1); sort2!(3, 4); sort2!(2, 5); sort2!(7, 8); 
            sort2!(6, 9); sort2!(10, 11); sort2!(0, 2); sort2!(1, 6); sort2!(5, 10); 
            sort2!(9, 11); sort2!(1, 2); sort2!(0, 3); sort2!(4, 6); sort2!(5, 7); 
            sort2!(9, 10); sort2!(8, 11); sort2!(1, 4); sort2!(3, 5); sort2!(6, 8); 
            sort2!(7, 10); sort2!(1, 3); sort2!(2, 5); sort2!(6, 9); sort2!(8, 10); 
            sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); sort2!(4, 6); 
            sort2!(5, 7); sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); 
        }
        13 => {
            sort2!(3, 7); sort2!(6, 8); sort2!(2, 9); sort2!(1, 10); sort2!(5, 11); 
            sort2!(0, 12); sort2!(2, 3); sort2!(1, 6); sort2!(7, 9); sort2!(8, 10); 
            sort2!(4, 11); sort2!(1, 2); sort2!(0, 4); sort2!(3, 6); sort2!(7, 8); 
            sort2!(9, 10); sort2!(11, 12); sort2!(4, 6); sort2!(5, 9); sort2!(8, 11); 
            sort2!(10, 12); sort2!(0, 5); sort2!(4, 7); sort2!(3, 8); sort2!(9, 10); 
            sort2!(6, 11); sort2!(0, 1); sort2!(2, 5); sort2!(7, 8); sort2!(6, 9); 
            sort2!(10, 11); sort2!(1, 3); sort2!(2, 4); sort2!(5, 6); sort2!(9, 10); 
            sort2!(1, 2); sort2!(3, 4); sort2!(5, 7); sort2!(6, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); sort2!(3, 4); sort2!(5, 6); 
            
        }
        14 => {
            sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); 
            sort2!(10, 11); sort2!(12, 13); sort2!(0, 2); sort2!(1, 3); sort2!(4, 8); 
            sort2!(5, 9); sort2!(10, 12); sort2!(11, 13); sort2!(1, 2); sort2!(0, 4); 
            sort2!(3, 7); sort2!(5, 8); sort2!(6, 10); sort2!(11, 12); sort2!(9, 13); 
            sort2!(1, 5); sort2!(0, 6); sort2!(3, 9); sort2!(4, 10); sort2!(8, 12); 
            sort2!(7, 13); sort2!(4, 6); sort2!(7, 9); sort2!(2, 10); sort2!(3, 11); 
            sort2!(1, 3); sort2!(6, 7); sort2!(2, 8); sort2!(5, 11); sort2!(10, 12); 
            sort2!(1, 4); sort2!(3, 5); sort2!(2, 6); sort2!(8, 10); sort2!(7, 11); 
            sort2!(9, 12); sort2!(2, 4); sort2!(3, 6); sort2!(5, 8); sort2!(7, 10); 
            sort2!(9, 11); sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); sort2!(9, 10); 
            sort2!(6, 7); 
        }
        15 => {
            sort2!(1, 2); sort2!(5, 8); sort2!(3, 10); sort2!(9, 11); sort2!(7, 12); 
            sort2!(6, 13); sort2!(4, 14); sort2!(1, 5); sort2!(3, 7); sort2!(2, 8); 
            sort2!(6, 9); sort2!(10, 12); sort2!(11, 13); sort2!(0, 14); sort2!(1, 6); 
            sort2!(0, 7); sort2!(2, 9); sort2!(4, 10); sort2!(5, 11); sort2!(8, 13); 
            sort2!(12, 14); sort2!(2, 4); sort2!(3, 5); sort2!(0, 6); sort2!(8, 10); 
            sort2!(7, 11); sort2!(9, 12); sort2!(13, 14); sort2!(1, 2); sort2!(0, 3); 
            sort2!(4, 7); sort2!(6, 8); sort2!(5, 9); sort2!(10, 11); sort2!(12, 13); 
            sort2!(0, 1); sort2!(2, 3); sort2!(4, 6); sort2!(7, 9); sort2!(10, 12); 
            sort2!(11, 13); sort2!(1, 2); sort2!(3, 5); sort2!(8, 10); sort2!(11, 12); 
            sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); sort2!(9, 10); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); sort2!(10, 11); sort2!(5, 6); 
            sort2!(7, 8); 
        }
        16 => {
            sort2!(5, 6); sort2!(4, 8); sort2!(9, 10); sort2!(7, 11); sort2!(1, 12); 
            sort2!(0, 13); sort2!(3, 14); sort2!(2, 15); sort2!(3, 4); sort2!(0, 5); 
            sort2!(1, 7); sort2!(2, 9); sort2!(11, 12); sort2!(6, 13); sort2!(8, 14); 
            sort2!(10, 15); sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); sort2!(6, 8); 
            sort2!(7, 9); sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(0, 2); 
            sort2!(1, 3); sort2!(6, 7); sort2!(8, 9); sort2!(4, 10); sort2!(5, 11); 
            sort2!(12, 14); sort2!(13, 15); sort2!(1, 2); sort2!(4, 6); sort2!(5, 7); 
            sort2!(8, 10); sort2!(9, 11); sort2!(3, 12); sort2!(13, 14); sort2!(1, 4); 
            sort2!(2, 6); sort2!(5, 8); sort2!(7, 10); sort2!(9, 13); sort2!(11, 14); 
            sort2!(2, 4); sort2!(3, 6); sort2!(9, 12); sort2!(11, 13); sort2!(3, 5); 
            sort2!(6, 8); sort2!(7, 9); sort2!(10, 12); sort2!(3, 4); sort2!(5, 6); 
            sort2!(7, 8); sort2!(9, 10); sort2!(11, 12); sort2!(6, 7); sort2!(8, 9); 	
        }
        _ => unimplemented!("not implemented for N={N}"),
    }
}

#[rustfmt::skip]
pub(crate) fn tinysort<T,F>(data: &mut [T],  lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            sort2(data, $a, $b, lt);
        };
    }
    match data.len() {
        0 | 1 => {}
        2 => { sort2!(0, 1); }
        3 => { sort2!(0, 2); sort2!(0, 1); sort2!(1, 2); }
        4 => { sort2!(0, 2); sort2!(1, 3); sort2!(0, 1); sort2!(2, 3); sort2!(1, 2); }
        5 => { 
            sort2!(0, 3); sort2!(1, 4); sort2!(0, 2); sort2!(1, 3); sort2!(0, 1); 
            sort2!(2, 4); sort2!(1, 2); sort2!(3, 4); sort2!(2, 3); 
        }
        6 => { 
            sort2!(0, 5); sort2!(1, 3); sort2!(2, 4); sort2!(1, 2); sort2!(3, 4); 
            sort2!(0, 3); sort2!(2, 5); sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); 
            sort2!(1, 2); sort2!(3, 4); 
        }
        7 => { 
            sort2!(0, 6); sort2!(2, 3); sort2!(4, 5); sort2!(0, 2); sort2!(1, 4); 
            sort2!(3, 6); sort2!(0, 1); sort2!(2, 5); sort2!(3, 4); sort2!(1, 2); 
            sort2!(4, 6); sort2!(2, 3); sort2!(4, 5); sort2!(1, 2); sort2!(3, 4); 
            sort2!(5, 6); 
        }
        8 => { 
            sort2!(0, 2); sort2!(1, 3); sort2!(4, 6); sort2!(5, 7); sort2!(0, 4); 
            sort2!(1, 5); sort2!(2, 6); sort2!(3, 7); sort2!(0, 1); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(2, 4); sort2!(3, 5); sort2!(1, 4); 
            sort2!(3, 6); sort2!(1, 2); sort2!(3, 4); sort2!(5, 6); 
        }
        9 => {
            sort2!(0, 3); sort2!(2, 5); sort2!(1, 7); sort2!(4, 8); sort2!(2, 4); 
            sort2!(5, 6); sort2!(0, 7); sort2!(3, 8); sort2!(0, 2); sort2!(1, 3); 
            sort2!(4, 5); sort2!(7, 8); sort2!(1, 4); sort2!(3, 6); sort2!(5, 7); 
            sort2!(0, 1); sort2!(2, 4); sort2!(3, 5); sort2!(6, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(1, 2); sort2!(3, 4); sort2!(5, 6); 
            
        }
        10 => {
            sort2!(3, 5); sort2!(4, 6); sort2!(2, 7); sort2!(0, 8); sort2!(1, 9); 
            sort2!(0, 2); sort2!(1, 4); sort2!(5, 8); sort2!(7, 9); sort2!(0, 3); 
            sort2!(2, 4); sort2!(5, 7); sort2!(6, 9); sort2!(0, 1); sort2!(3, 6); 
            sort2!(8, 9); sort2!(2, 3); sort2!(1, 5); sort2!(6, 7); sort2!(4, 8); 
            sort2!(1, 2); sort2!(3, 5); sort2!(4, 6); sort2!(7, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(3, 4); sort2!(5, 6); 
        }
        11 => {
            sort2!(2, 4); sort2!(1, 6); sort2!(3, 7); sort2!(5, 8); sort2!(0, 9); 
            sort2!(0, 1); sort2!(3, 5); sort2!(7, 8); sort2!(6, 9); sort2!(4, 10); 
            sort2!(1, 3); sort2!(2, 5); sort2!(4, 7); sort2!(8, 10); sort2!(1, 2); 
            sort2!(0, 4); sort2!(3, 7); sort2!(6, 8); sort2!(5, 9); sort2!(0, 1); 
            sort2!(4, 5); sort2!(2, 6); sort2!(7, 8); sort2!(9, 10); sort2!(2, 4); 
            sort2!(3, 6); sort2!(5, 7); sort2!(8, 9); sort2!(1, 2); sort2!(3, 4); 
            sort2!(5, 6); sort2!(7, 8); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); 
            
        }
        12 => {
            sort2!(2, 6); sort2!(1, 7); sort2!(0, 8); sort2!(5, 9); sort2!(4, 10); 
            sort2!(3, 11); sort2!(0, 1); sort2!(3, 4); sort2!(2, 5); sort2!(7, 8); 
            sort2!(6, 9); sort2!(10, 11); sort2!(0, 2); sort2!(1, 6); sort2!(5, 10); 
            sort2!(9, 11); sort2!(1, 2); sort2!(0, 3); sort2!(4, 6); sort2!(5, 7); 
            sort2!(9, 10); sort2!(8, 11); sort2!(1, 4); sort2!(3, 5); sort2!(6, 8); 
            sort2!(7, 10); sort2!(1, 3); sort2!(2, 5); sort2!(6, 9); sort2!(8, 10); 
            sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); sort2!(4, 6); 
            sort2!(5, 7); sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); 
        }
        13 => {
            sort2!(3, 7); sort2!(6, 8); sort2!(2, 9); sort2!(1, 10); sort2!(5, 11); 
            sort2!(0, 12); sort2!(2, 3); sort2!(1, 6); sort2!(7, 9); sort2!(8, 10); 
            sort2!(4, 11); sort2!(1, 2); sort2!(0, 4); sort2!(3, 6); sort2!(7, 8); 
            sort2!(9, 10); sort2!(11, 12); sort2!(4, 6); sort2!(5, 9); sort2!(8, 11); 
            sort2!(10, 12); sort2!(0, 5); sort2!(4, 7); sort2!(3, 8); sort2!(9, 10); 
            sort2!(6, 11); sort2!(0, 1); sort2!(2, 5); sort2!(7, 8); sort2!(6, 9); 
            sort2!(10, 11); sort2!(1, 3); sort2!(2, 4); sort2!(5, 6); sort2!(9, 10); 
            sort2!(1, 2); sort2!(3, 4); sort2!(5, 7); sort2!(6, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); sort2!(3, 4); sort2!(5, 6); 
            
        }
        14 => {
            sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); 
            sort2!(10, 11); sort2!(12, 13); sort2!(0, 2); sort2!(1, 3); sort2!(4, 8); 
            sort2!(5, 9); sort2!(10, 12); sort2!(11, 13); sort2!(1, 2); sort2!(0, 4); 
            sort2!(3, 7); sort2!(5, 8); sort2!(6, 10); sort2!(11, 12); sort2!(9, 13); 
            sort2!(1, 5); sort2!(0, 6); sort2!(3, 9); sort2!(4, 10); sort2!(8, 12); 
            sort2!(7, 13); sort2!(4, 6); sort2!(7, 9); sort2!(2, 10); sort2!(3, 11); 
            sort2!(1, 3); sort2!(6, 7); sort2!(2, 8); sort2!(5, 11); sort2!(10, 12); 
            sort2!(1, 4); sort2!(3, 5); sort2!(2, 6); sort2!(8, 10); sort2!(7, 11); 
            sort2!(9, 12); sort2!(2, 4); sort2!(3, 6); sort2!(5, 8); sort2!(7, 10); 
            sort2!(9, 11); sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); sort2!(9, 10); 
            sort2!(6, 7); 
        }
        15 => {
            sort2!(1, 2); sort2!(5, 8); sort2!(3, 10); sort2!(9, 11); sort2!(7, 12); 
            sort2!(6, 13); sort2!(4, 14); sort2!(1, 5); sort2!(3, 7); sort2!(2, 8); 
            sort2!(6, 9); sort2!(10, 12); sort2!(11, 13); sort2!(0, 14); sort2!(1, 6); 
            sort2!(0, 7); sort2!(2, 9); sort2!(4, 10); sort2!(5, 11); sort2!(8, 13); 
            sort2!(12, 14); sort2!(2, 4); sort2!(3, 5); sort2!(0, 6); sort2!(8, 10); 
            sort2!(7, 11); sort2!(9, 12); sort2!(13, 14); sort2!(1, 2); sort2!(0, 3); 
            sort2!(4, 7); sort2!(6, 8); sort2!(5, 9); sort2!(10, 11); sort2!(12, 13); 
            sort2!(0, 1); sort2!(2, 3); sort2!(4, 6); sort2!(7, 9); sort2!(10, 12); 
            sort2!(11, 13); sort2!(1, 2); sort2!(3, 5); sort2!(8, 10); sort2!(11, 12); 
            sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); sort2!(9, 10); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); sort2!(10, 11); sort2!(5, 6); 
            sort2!(7, 8); 
        }
        16 => {
            sort2!(5, 6); sort2!(4, 8); sort2!(9, 10); sort2!(7, 11); sort2!(1, 12); 
            sort2!(0, 13); sort2!(3, 14); sort2!(2, 15); sort2!(3, 4); sort2!(0, 5); 
            sort2!(1, 7); sort2!(2, 9); sort2!(11, 12); sort2!(6, 13); sort2!(8, 14); 
            sort2!(10, 15); sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); sort2!(6, 8); 
            sort2!(7, 9); sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(0, 2); 
            sort2!(1, 3); sort2!(6, 7); sort2!(8, 9); sort2!(4, 10); sort2!(5, 11); 
            sort2!(12, 14); sort2!(13, 15); sort2!(1, 2); sort2!(4, 6); sort2!(5, 7); 
            sort2!(8, 10); sort2!(9, 11); sort2!(3, 12); sort2!(13, 14); sort2!(1, 4); 
            sort2!(2, 6); sort2!(5, 8); sort2!(7, 10); sort2!(9, 13); sort2!(11, 14); 
            sort2!(2, 4); sort2!(3, 6); sort2!(9, 12); sort2!(11, 13); sort2!(3, 5); 
            sort2!(6, 8); sort2!(7, 9); sort2!(10, 12); sort2!(3, 4); sort2!(5, 6); 
            sort2!(7, 8); sort2!(9, 10); sort2!(11, 12); sort2!(6, 7); sort2!(8, 9); 
        }
        _ => insertion_sort(data, lt),
    }
}

pub(crate) fn insertion_sort<T, F>(data: &mut [T], lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut index = 1;
    let end = data.len();
    while index < end {
        let mut elem = Elem::new(data, index);
        while elem.index > 0 {
            if unsafe { !lt(elem.as_ref(), elem.prev()) } {
                break;
            }
            unsafe { elem.move_back() };
        }
        index += 1;
    }
}
