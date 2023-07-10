use core::mem::ManuallyDrop;

#[inline]
/// Compares the elements at `a` and `b` and swaps them if `a` is greater than `b`. Returns `true`
/// if the elements were swapped.
fn swap<T, F>(data: &mut [T], a: usize, b: usize, is_less: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    debug_assert!(a != b);
    debug_assert!(a < data.len());
    debug_assert!(b < data.len());

    unsafe {
        let ptr = data.as_mut_ptr();
        let (a, b) = (a as isize, b as isize);
        let swap = is_less(&*ptr.offset(b), &*ptr.offset(a)) as isize;
        let max = ptr.offset(swap * a + (1 - swap) * b);
        let min = ptr.offset(swap * b + (1 - swap) * a);
        let tmp = ManuallyDrop::new(core::ptr::read(max));
        ptr.offset(a).copy_from(min, 1);
        ptr.offset(b).write(ManuallyDrop::into_inner(tmp));
        swap != 0
    }
}

#[rustfmt::skip]
/// Sorts the elements at the positions in `pos` so that the smallest element becomes `pos0,` and
/// each element `pos[i]` is less than or equal to the element at `pos[j]` if `i < j`.
/// 
/// The sorting networks are from https://bertdobbelaere.github.io/sorting_networks.html
pub(crate) fn sort_at<T, F, const N: usize>(data: &mut [T], pos: [usize; N], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            swap(data, pos[$a], pos[$b], is_less);
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
        11 => {
            sort2!(2, 4); sort2!(1, 6); sort2!(3, 7); sort2!(5, 8); sort2!(0, 9); 
            sort2!(0, 1); sort2!(3, 5); sort2!(7, 8); sort2!(6, 9); sort2!(4, 10); 
            sort2!(1, 3); sort2!(2, 5); sort2!(4, 7); sort2!(8, 10); sort2!(1, 2); 
            sort2!(0, 4); sort2!(3, 7); sort2!(6, 8); sort2!(5, 9); sort2!(0, 1); 
            sort2!(4, 5); sort2!(2, 6); sort2!(7, 8); sort2!(9, 10); sort2!(2, 4); 
            sort2!(3, 6); sort2!(5, 7); sort2!(8, 9); sort2!(1, 2); sort2!(3, 4); 
            sort2!(5, 6); sort2!(7, 8); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); 
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
        21 => {
            sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); 
            sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(16, 17); sort2!(18, 19); 
            sort2!(0, 2); sort2!(1, 3); sort2!(4, 6); sort2!(5, 7); sort2!(8, 10); 
            sort2!(9, 11); sort2!(12, 14); sort2!(13, 15); sort2!(16, 18); sort2!(17, 19); 
            sort2!(0, 8); sort2!(1, 9); sort2!(2, 10); sort2!(3, 11); sort2!(4, 12); 
            sort2!(5, 13); sort2!(6, 14); sort2!(7, 15); sort2!(0, 4); sort2!(1, 5); 
            sort2!(3, 7); sort2!(8, 12); sort2!(9, 13); sort2!(10, 14); sort2!(15, 19); 
            sort2!(6, 20); sort2!(2, 6); sort2!(3, 18); sort2!(7, 20); sort2!(3, 6); 
            sort2!(2, 16); sort2!(7, 17); sort2!(5, 18); sort2!(11, 20); sort2!(0, 2); 
            sort2!(3, 8); sort2!(7, 10); sort2!(6, 12); sort2!(11, 15); sort2!(9, 16); 
            sort2!(13, 17); sort2!(14, 18); sort2!(19, 20); sort2!(2, 3); sort2!(1, 7); 
            sort2!(4, 9); sort2!(10, 11); sort2!(13, 16); sort2!(15, 18); sort2!(17, 19); 
            sort2!(1, 4); sort2!(7, 8); sort2!(5, 10); sort2!(6, 13); sort2!(11, 14); 
            sort2!(12, 16); sort2!(15, 17); sort2!(18, 19); sort2!(1, 2); sort2!(3, 4); 
            sort2!(5, 6); sort2!(10, 12); sort2!(11, 13); sort2!(14, 16); sort2!(17, 18); 
            sort2!(2, 3); sort2!(4, 5); sort2!(6, 9); sort2!(10, 11); sort2!(12, 13); 
            sort2!(14, 15); sort2!(16, 17); sort2!(6, 7); sort2!(8, 9); sort2!(15, 16); 
            sort2!(4, 6); sort2!(7, 8); sort2!(9, 12); sort2!(13, 15); sort2!(3, 4); 
            sort2!(5, 7); sort2!(8, 10); sort2!(9, 11); sort2!(12, 14); sort2!(5, 6); 
            sort2!(7, 8); sort2!(9, 10); sort2!(11, 12); sort2!(13, 14); 
        }
        31 => {
            sort2!(0, 1); sort2!(2, 3); sort2!(4, 5); sort2!(6, 7); sort2!(8, 9); 
            sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(16, 17); sort2!(18, 19); 
            sort2!(20, 21); sort2!(22, 23); sort2!(24, 25); sort2!(26, 27); sort2!(28, 29); 
            sort2!(0, 2); sort2!(1, 3); sort2!(4, 6); sort2!(5, 7); sort2!(8, 10); 
            sort2!(9, 11); sort2!(12, 14); sort2!(13, 15); sort2!(16, 18); sort2!(17, 19); 
            sort2!(20, 22); sort2!(21, 23); sort2!(24, 26); sort2!(25, 27); sort2!(28, 30); 
            sort2!(0, 4); sort2!(1, 5); sort2!(2, 6); sort2!(3, 7); sort2!(8, 12); 
            sort2!(9, 13); sort2!(10, 14); sort2!(11, 15); sort2!(16, 20); sort2!(17, 21); 
            sort2!(18, 22); sort2!(19, 23); sort2!(24, 28); sort2!(25, 29); sort2!(26, 30); 
            sort2!(0, 8); sort2!(1, 9); sort2!(2, 10); sort2!(3, 11); sort2!(4, 12); 
            sort2!(5, 13); sort2!(6, 14); sort2!(7, 15); sort2!(16, 24); sort2!(17, 25); 
            sort2!(18, 26); sort2!(19, 27); sort2!(20, 28); sort2!(21, 29); sort2!(22, 30); 
            sort2!(2, 4); sort2!(1, 8); sort2!(6, 9); sort2!(5, 10); sort2!(3, 12); 
            sort2!(11, 13); sort2!(7, 14); sort2!(0, 16); sort2!(18, 20); sort2!(17, 24); 
            sort2!(22, 25); sort2!(21, 26); sort2!(19, 28); sort2!(27, 29); sort2!(23, 30); 
            sort2!(1, 2); sort2!(3, 5); sort2!(4, 8); sort2!(7, 11); sort2!(10, 12); 
            sort2!(13, 14); sort2!(17, 18); sort2!(19, 21); sort2!(6, 22); sort2!(20, 24); 
            sort2!(9, 25); sort2!(23, 27); sort2!(26, 28); sort2!(29, 30); sort2!(5, 10); 
            sort2!(1, 17); sort2!(2, 18); sort2!(3, 19); sort2!(4, 20); sort2!(7, 23); 
            sort2!(8, 24); sort2!(21, 26); sort2!(11, 27); sort2!(12, 28); sort2!(13, 29); 
            sort2!(14, 30); sort2!(7, 9); sort2!(4, 16); sort2!(3, 17); sort2!(6, 18); 
            sort2!(8, 20); sort2!(5, 21); sort2!(11, 23); sort2!(22, 24); sort2!(13, 25); 
            sort2!(10, 26); sort2!(15, 27); sort2!(14, 28); sort2!(1, 4); sort2!(3, 8); 
            sort2!(5, 16); sort2!(7, 17); sort2!(11, 19); sort2!(12, 20); sort2!(9, 21); 
            sort2!(10, 22); sort2!(14, 24); sort2!(15, 26); sort2!(23, 28); sort2!(27, 30); 
            sort2!(2, 5); sort2!(7, 8); sort2!(12, 16); sort2!(11, 17); sort2!(9, 18); 
            sort2!(15, 19); sort2!(14, 20); sort2!(13, 22); sort2!(23, 24); sort2!(26, 29); 
            sort2!(2, 4); sort2!(10, 11); sort2!(6, 12); sort2!(9, 16); sort2!(13, 17); 
            sort2!(14, 18); sort2!(20, 21); sort2!(15, 22); sort2!(19, 25); sort2!(27, 29); 
            sort2!(5, 6); sort2!(9, 10); sort2!(8, 12); sort2!(11, 13); sort2!(14, 16); 
            sort2!(15, 17); sort2!(18, 20); sort2!(21, 22); sort2!(19, 23); sort2!(25, 26); 
            sort2!(3, 5); sort2!(6, 7); sort2!(8, 9); sort2!(10, 12); sort2!(11, 14); 
            sort2!(13, 16); sort2!(15, 18); sort2!(17, 20); sort2!(19, 21); sort2!(22, 23); 
            sort2!(24, 25); sort2!(26, 28); sort2!(3, 4); sort2!(5, 6); sort2!(7, 8); 
            sort2!(9, 10); sort2!(11, 12); sort2!(13, 14); sort2!(15, 16); sort2!(17, 18); 
            sort2!(19, 20); sort2!(21, 22); sort2!(23, 24); sort2!(25, 26); sort2!(27, 28); 
            
        }
        _ => unimplemented!("sort not implemented for N={N}"),
    } 
}

#[rustfmt::skip]
pub(crate) fn tinysort<T, F>(data: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            swap(data, $a, $b, is_less);
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
        _ => unimplemented!("tinysort not implemented for data.len() > 8")
    }
}
