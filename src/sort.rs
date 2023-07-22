use core::{mem::ManuallyDrop, ops::Range};

use crate::Elem;

#[inline]
/// Compares the elements at `a` and `b` and swaps them if `a` is greater than `b`. Returns `true`
/// if the elements were swapped.
fn swap<T, F>(data: &mut [T], a: usize, b: usize, lt: &mut F) -> bool
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
            swap(data, pos[$a], pos[$b], lt);
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
            swap(data, $a, $b, lt);
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
    let Range { start, end } = data.as_mut_ptr_range();
    unsafe {
        let mut l = start.add(1);
        while l < end {
            {
                let mut elem = Elem::new(l);
                while elem.dst > start {
                    if !lt(&elem, &*elem.dst.sub(1)) {
                        break;
                    }
                    core::ptr::copy_nonoverlapping(elem.dst.sub(1), elem.dst, 1);
                    elem.dst = elem.dst.sub(1);
                }
            }
            l = l.add(1);
        }
    }
}
