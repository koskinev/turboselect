use core::convert::identity;

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
    /// Unsafe because self.index must be > 0.
    #[inline]
    pub(crate) unsafe fn move_back(&mut self) {
        debug_assert!(self.index > 0);
        unsafe {
            let ptr = self.data.as_mut_ptr().add(self.index);
            core::ptr::copy_nonoverlapping(ptr.sub(1), ptr, 1);
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
/// if the elements were swapped. Panics if `a` or `b` is out of bounds or `a == b`.
fn sort2<T, F>(data: &mut [T], a: usize, b: usize, lt: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    assert!(a != b);
    assert!(a < data.len());
    assert!(b < data.len());
    unsafe {
        let ptr = data.as_mut_ptr();
        let swap = lt(&*ptr.add(b), &*ptr.add(a)) as usize;
        let max = ptr.add(swap * a + (1 - swap) * b);
        let min = ptr.add(swap * b + (1 - swap) * a);
        let tmp = max.read();
        ptr.add(a).copy_from(min, 1);
        ptr.add(b).write(tmp);
        swap != 0
    }
}

#[rustfmt::skip]
pub(crate) fn sort_at<T, M, F>(data: &mut [T], map: &M, n: usize, lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
    M: Fn(usize) -> usize,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            sort2(data, map($a), map($b), lt);
        };
    }

    match n {
        0 | 1 => {}
        2 => { sort2!(0, 1); }
        3 => { sort2!(0, 2); sort2!(0, 1); sort2!(1, 2); }
        4 => { 
            sort2!(0, 2); sort2!(1, 3); sort2!(0, 1); sort2!(2, 3); sort2!(1, 2); 
        }
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
            sort2!(2, 6);  sort2!(1, 7);   sort2!(0, 8); sort2!(5, 9); sort2!(4, 10); 
            sort2!(3, 11); sort2!(0, 1);   sort2!(3, 4); sort2!(2, 5); sort2!(7, 8); 
            sort2!(6, 9);  sort2!(10, 11); sort2!(0, 2); sort2!(1, 6); sort2!(5, 10); 
            sort2!(9, 11); sort2!(1, 2);   sort2!(0, 3); sort2!(4, 6); sort2!(5, 7); 
            sort2!(9, 10); sort2!(8, 11);  sort2!(1, 4); sort2!(3, 5); sort2!(6, 8); 
            sort2!(7, 10); sort2!(1, 3);   sort2!(2, 5); sort2!(6, 9); sort2!(8, 10); 
            sort2!(2, 3);  sort2!(4, 5);   sort2!(6, 7); sort2!(8, 9); sort2!(4, 6); 
            sort2!(5, 7);  sort2!(3, 4);   sort2!(5, 6); sort2!(7, 8); 
        }
        13 => {
            sort2!(3, 7);   sort2!(6, 8);   sort2!(2, 9); sort2!(1, 10); sort2!(5, 11); 
            sort2!(0, 12);  sort2!(2, 3);   sort2!(1, 6); sort2!(7, 9);  sort2!(8, 10); 
            sort2!(4, 11);  sort2!(1, 2);   sort2!(0, 4); sort2!(3, 6);  sort2!(7, 8); 
            sort2!(9, 10);  sort2!(11, 12); sort2!(4, 6); sort2!(5, 9);  sort2!(8, 11); 
            sort2!(10, 12); sort2!(0, 5);   sort2!(4, 7); sort2!(3, 8);  sort2!(9, 10); 
            sort2!(6, 11);  sort2!(0, 1);   sort2!(2, 5); sort2!(7, 8);  sort2!(6, 9); 
            sort2!(10, 11); sort2!(1, 3);   sort2!(2, 4); sort2!(5, 6);  sort2!(9, 10); 
            sort2!(1, 2);   sort2!(3, 4);   sort2!(5, 7); sort2!(6, 8);  sort2!(2, 3); 
            sort2!(4, 5);   sort2!(6, 7);   sort2!(8, 9); sort2!(3, 4);  sort2!(5, 6); 
        }
        14 => {
            sort2!(0, 1);   sort2!(2, 3);   sort2!(4, 5);   sort2!(6, 7);   sort2!(8, 9); 
            sort2!(10, 11); sort2!(12, 13); sort2!(0, 2);   sort2!(1, 3);   sort2!(4, 8); 
            sort2!(5, 9);   sort2!(10, 12); sort2!(11, 13); sort2!(1, 2);   sort2!(0, 4); 
            sort2!(3, 7);   sort2!(5, 8);   sort2!(6, 10);  sort2!(11, 12); sort2!(9, 13); 
            sort2!(1, 5);   sort2!(0, 6);   sort2!(3, 9);   sort2!(4, 10);  sort2!(8, 12); 
            sort2!(7, 13);  sort2!(4, 6);   sort2!(7, 9);   sort2!(2, 10);  sort2!(3, 11); 
            sort2!(1, 3);   sort2!(6, 7);   sort2!(2, 8);   sort2!(5, 11);  sort2!(10, 12); 
            sort2!(1, 4);   sort2!(3, 5);   sort2!(2, 6);   sort2!(8, 10);  sort2!(7, 11); 
            sort2!(9, 12);  sort2!(2, 4);   sort2!(3, 6);   sort2!(5, 8);   sort2!(7, 10); 
            sort2!(9, 11);  sort2!(3, 4);   sort2!(5, 6);   sort2!(7, 8);   sort2!(9, 10); 
            sort2!(6, 7); 
        }
        15 => {
            sort2!(1, 2);   sort2!(5, 8);   sort2!(3, 10);  sort2!(9, 11);  sort2!(7, 12); 
            sort2!(6, 13);  sort2!(4, 14);  sort2!(1, 5);   sort2!(3, 7);   sort2!(2, 8); 
            sort2!(6, 9);   sort2!(10, 12); sort2!(11, 13); sort2!(0, 14);  sort2!(1, 6); 
            sort2!(0, 7);   sort2!(2, 9);   sort2!(4, 10);  sort2!(5, 11);  sort2!(8, 13); 
            sort2!(12, 14); sort2!(2, 4);   sort2!(3, 5);   sort2!(0, 6);   sort2!(8, 10); 
            sort2!(7, 11);  sort2!(9, 12);  sort2!(13, 14); sort2!(1, 2);   sort2!(0, 3); 
            sort2!(4, 7);   sort2!(6, 8);   sort2!(5, 9);   sort2!(10, 11); sort2!(12, 13); 
            sort2!(0, 1);   sort2!(2, 3);   sort2!(4, 6);   sort2!(7, 9);   sort2!(10, 12); 
            sort2!(11, 13); sort2!(1, 2);   sort2!(3, 5);   sort2!(8, 10);  sort2!(11, 12); 
            sort2!(3, 4);   sort2!(5, 6);   sort2!(7, 8);   sort2!(9, 10);  sort2!(2, 3); 
            sort2!(4, 5);   sort2!(6, 7);   sort2!(8, 9);   sort2!(10, 11); sort2!(5, 6); 
            sort2!(7, 8); 
        }
        16 => {
            sort2!(5, 6);   sort2!(4, 8);   sort2!(9, 10);  sort2!(7, 11);  sort2!(1, 12); 
            sort2!(0, 13);  sort2!(3, 14);  sort2!(2, 15);  sort2!(3, 4);   sort2!(0, 5); 
            sort2!(1, 7);   sort2!(2, 9);   sort2!(11, 12); sort2!(6, 13);  sort2!(8, 14); 
            sort2!(10, 15); sort2!(0, 1);   sort2!(2, 3);   sort2!(4, 5);   sort2!(6, 8); 
            sort2!(7, 9);   sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(0, 2); 
            sort2!(1, 3);   sort2!(6, 7);   sort2!(8, 9);   sort2!(4, 10);  sort2!(5, 11); 
            sort2!(12, 14); sort2!(13, 15); sort2!(1, 2);   sort2!(4, 6);   sort2!(5, 7); 
            sort2!(8, 10);  sort2!(9, 11);  sort2!(3, 12);  sort2!(13, 14); sort2!(1, 4); 
            sort2!(2, 6);   sort2!(5, 8);   sort2!(7, 10);  sort2!(9, 13);  sort2!(11, 14); 
            sort2!(2, 4);   sort2!(3, 6);   sort2!(9, 12);  sort2!(11, 13); sort2!(3, 5); 
            sort2!(6, 8);   sort2!(7, 9);   sort2!(10, 12); sort2!(3, 4);   sort2!(5, 6); 
            sort2!(7, 8);   sort2!(9, 10);  sort2!(11, 12); sort2!(6, 7);   sort2!(8, 9); 
        }
        n => unimplemented!("sorting network for size {n} not implemented"),
    }
}

pub(crate) fn tinysort<T, F>(data: &mut [T], lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    match data.len() {
        0 | 1 => {}
        len if len <= 16 => sort_at(data, &identity, len, lt),
        len => {
            let parts = (len + 15) / 16;
            for p in 0..parts {
                let n = (len + parts - p - 1) / parts;
                sort_at(data, &|i| i * parts + p, n, lt);
            }
            insertion_sort(data, lt)
        }
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
            // Safety: elem.prev() and elem.move_back() require that elem.index > 0, which is
            // guaranteed by the loop condition.
            if unsafe { !lt(elem.as_ref(), elem.prev()) } {
                break;
            }
            unsafe { elem.move_back() };
        }
        index += 1;
    }
}
