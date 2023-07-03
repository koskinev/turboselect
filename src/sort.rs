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
/// Reorders the element at the positions in `pos` so that the median becomes `pos[N / 2]`.
/// Element at `pos[i]` is less than or equal to the element at `pos[N / 2]` for all `i < N / 2`
/// and greater than or equal to the element at `pos[N / 2]` for all `i > N / 2`.
pub(crate) fn median_at<T, F, const N: usize>(data: &mut [T], pos: [usize; N], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            swap(data, pos[$a], pos[$b], is_less);
        };
    }
    
    match N {
        2 => { sort2!(0, 1); }
        3 => { sort2!(0, 2); sort2!(0, 1); sort2!(1, 2); }
        4 => { sort2!(0, 2); sort2!(1, 3); sort2!(0, 1); sort2!(2, 3); sort2!(1, 2); }
        5 => { 
            sort2!(0, 1); sort2!(2, 3); sort2!(0, 2); sort2!(1, 3); sort2!(2, 4); sort2!(1, 2); 
            sort2!(2, 4); 
        }
        6 => { 
            sort2!(0, 1); sort2!(4, 5); sort2!(0, 5); sort2!(1, 3); sort2!(2, 4); sort2!(0, 2); 
            sort2!(1, 4); sort2!(3, 5); sort2!(1, 2); sort2!(3, 4); sort2!(2, 3); 
        }
        9 => { 
            sort2!(0, 7); sort2!(1, 2); sort2!(3, 5); sort2!(4, 8); sort2!(0, 2); sort2!(1, 5); 
            sort2!(3, 8); sort2!(4, 7); sort2!(0, 3); sort2!(1, 4); sort2!(2, 8); sort2!(5, 7);
            sort2!(3, 4); sort2!(5, 6); sort2!(2, 5); sort2!(4, 6); sort2!(2, 3); sort2!(4, 5);
            sort2!(3, 4); 
        }
        21 => {
            sort2!(0, 1);   sort2!(2, 3);   sort2!(4, 5);   sort2!(6, 7);   sort2!(8, 9); 
            sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(16, 17); sort2!(18, 19); 
            sort2!(0, 2);   sort2!(1, 3);   sort2!(4, 6);   sort2!(5, 7);   sort2!(8, 10);
            sort2!(9, 11);  sort2!(12, 14); sort2!(13, 15); sort2!(16, 18); sort2!(17, 19); 
            sort2!(1, 5);   sort2!(2, 6);   sort2!(3, 15);  sort2!(4, 16);  sort2!(13, 17); 
            sort2!(14, 18); sort2!(1, 14);  sort2!(2, 13);  sort2!(3, 7);   sort2!(5, 18);  
            sort2!(6, 17);  sort2!(12, 16); sort2!(0, 16);  sort2!(1, 2);   sort2!(3, 19);  
            sort2!(5, 13);  sort2!(6, 14);  sort2!(17, 18); sort2!(0, 4);   sort2!(5, 14);  
            sort2!(6, 10);  sort2!(9, 13);  sort2!(15, 19); sort2!(5, 8);   sort2!(6, 12);  
            sort2!(7, 13);  sort2!(11, 14); sort2!(2, 12);  sort2!(7, 17);  sort2!(8, 9);   
            sort2!(10, 11); sort2!(3, 9);   sort2!(7, 11);  sort2!(8, 12);  sort2!(10, 16);
            sort2!(3, 10);  sort2!(4, 12);  sort2!(7, 15);  sort2!(9, 16);  sort2!(7, 10); 
            sort2!(9, 12);  sort2!(7, 9);   sort2!(10, 12); sort2!(9, 10);  sort2!(10, 20); 
            sort2!(9, 10); 
        }
        _ => unimplemented!("median not implemented for N = {N}"),
    }
}

#[rustfmt::skip]
/// Sorts the elements at the positions in `pos` so that the smallest element becomes `pos[0]` and
/// each element `pos[i]` is less than or equal to the element at `pos[j]` if `i < j`.
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
            sort2!(0, 3); sort2!(1, 7); sort2!(2, 5); sort2!(4, 8); sort2!(0, 7); 
            sort2!(2, 4); sort2!(3, 8); sort2!(5, 6); sort2!(0, 2); sort2!(1, 3); 
            sort2!(4, 5); sort2!(7, 8); sort2!(1, 4); sort2!(3, 6); sort2!(5, 7); 
            sort2!(0, 1); sort2!(2, 4); sort2!(3, 5); sort2!(6, 8); sort2!(2, 3); 
            sort2!(4, 5); sort2!(6, 7); sort2!(1, 2); sort2!(3, 4); sort2!(5, 6); 
        }
        15 => {
            sort2!(1, 2);   sort2!(3, 10); sort2!(4, 14);  sort2!(5, 8);   sort2!(6, 13); 
            sort2!(7, 12);  sort2!(9, 11); sort2!(0, 14);  sort2!(1, 5);   sort2!(2, 8);
            sort2!(3, 7);   sort2!(6, 9);  sort2!(10, 12); sort2!(11, 13); sort2!(0, 7); 
            sort2!(1, 6);   sort2!(2, 9);  sort2!(4, 10);  sort2!(5, 11);  sort2!(8, 13); 
            sort2!(12, 14); sort2!(0, 6);  sort2!(2, 4);   sort2!(3, 5);   sort2!(7, 11); 
            sort2!(8, 10);  sort2!(9, 12); sort2!(13, 14); sort2!(0, 3);   sort2!(1, 2); 
            sort2!(4, 7);   sort2!(5, 9);  sort2!(6, 8);   sort2!(10, 11); sort2!(12, 13); 
            sort2!(0, 1);   sort2!(2, 3);  sort2!(4, 6);   sort2!(7, 9);   sort2!(10, 12); 
            sort2!(11, 13); sort2!(1, 2);  sort2!(3, 5);   sort2!(8, 10);  sort2!(11, 12); 
            sort2!(3, 4);   sort2!(5, 6);  sort2!(7, 8);   sort2!(9, 10);  sort2!(2, 3); 
            sort2!(4, 5);   sort2!(6, 7);  sort2!(8, 9);   sort2!(10, 11); sort2!(5, 6); 
            sort2!(7, 8);
        }
        21 => { 
            sort2!(0, 1);   sort2!(2, 3);   sort2!(4, 5);   sort2!(6, 7);   sort2!(8, 9);
            sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(16, 17); sort2!(18, 19);
            sort2!(0, 2);   sort2!(1, 3);   sort2!(4, 6);   sort2!(5, 7);   sort2!(8, 10);
            sort2!(9, 11);  sort2!(12, 14); sort2!(13, 15); sort2!(16, 18); sort2!(17, 19);
            sort2!(0, 8);   sort2!(1, 9);   sort2!(2, 10);  sort2!(3, 11);  sort2!(4, 12);
            sort2!(5, 13);  sort2!(6, 14);  sort2!(7, 15);  sort2!(0, 4);   sort2!(1, 5);
            sort2!(3, 7);   sort2!(6, 20);  sort2!(8, 12);  sort2!(9, 13);  sort2!(10, 14);
            sort2!(15, 19); sort2!(2, 6);   sort2!(3, 18);  sort2!(7, 20);  sort2!(2, 16);
            sort2!(3, 6);   sort2!(5, 18);  sort2!(7, 17);  sort2!(11, 20); sort2!(0, 2);
            sort2!(3, 8);   sort2!(6, 12);  sort2!(7, 10);  sort2!(9, 16);  sort2!(11, 15);
            sort2!(13, 17); sort2!(14, 18); sort2!(19, 20); sort2!(1, 7);   sort2!(2, 3);
            sort2!(4, 9);   sort2!(10, 11); sort2!(13, 16); sort2!(15, 18); sort2!(17, 19);
            sort2!(1, 4);   sort2!(5, 10);  sort2!(6, 13);  sort2!(7, 8);   sort2!(11, 14);
            sort2!(12, 16); sort2!(15, 17); sort2!(18, 19); sort2!(1, 2);   sort2!(3, 4);
            sort2!(5, 6);   sort2!(10, 12); sort2!(11, 13); sort2!(14, 16); sort2!(17, 18);
            sort2!(2, 3);   sort2!(4, 5);   sort2!(6, 9);   sort2!(10, 11); sort2!(12, 13);
            sort2!(14, 15); sort2!(16, 17); sort2!(6, 7);   sort2!(8, 9);   sort2!(15, 16);
            sort2!(4, 6);   sort2!(7, 8);   sort2!(9, 12);  sort2!(13, 15); sort2!(3, 4);
            sort2!(5, 7);   sort2!(8, 10);  sort2!(9, 11);  sort2!(12, 14); sort2!(5, 6);
            sort2!(7, 8);   sort2!(9, 10);  sort2!(11, 12); sort2!(13, 14);
        }
        63 => {
            sort2!(1, 2);   sort2!(3, 21);  sort2!(4, 6);   sort2!(5, 7);   sort2!(8, 10);
            sort2!(9, 11);  sort2!(12, 14); sort2!(13, 15); sort2!(16, 18); sort2!(17, 19);
            sort2!(20, 22); sort2!(23, 57); sort2!(24, 26); sort2!(25, 27); sort2!(28, 30);
            sort2!(29, 31); sort2!(32, 34); sort2!(33, 35); sort2!(36, 38); sort2!(37, 39);
            sort2!(40, 42); sort2!(41, 43); sort2!(44, 46); sort2!(45, 47); sort2!(48, 50);
            sort2!(49, 51); sort2!(52, 54); sort2!(53, 55); sort2!(56, 58); sort2!(59, 61);
            sort2!(60, 62); sort2!(0, 1);   sort2!(3, 20);  sort2!(4, 5);   sort2!(6, 7);
            sort2!(8, 9);   sort2!(10, 11); sort2!(12, 13); sort2!(14, 15); sort2!(16, 17);
            sort2!(18, 19); sort2!(21, 22); sort2!(23, 56); sort2!(24, 25); sort2!(26, 27);
            sort2!(28, 29); sort2!(30, 31); sort2!(32, 33); sort2!(34, 35); sort2!(36, 37);
            sort2!(38, 39); sort2!(40, 41); sort2!(42, 43); sort2!(44, 45); sort2!(46, 47);
            sort2!(48, 49); sort2!(50, 51); sort2!(52, 53); sort2!(54, 55); sort2!(57, 58);
            sort2!(59, 60); sort2!(61, 62); sort2!(0, 3);   sort2!(1, 2);   sort2!(4, 16);
            sort2!(5, 6);   sort2!(7, 19);  sort2!(8, 48);  sort2!(9, 10);  sort2!(11, 51);
            sort2!(12, 52); sort2!(13, 14); sort2!(15, 55); sort2!(17, 18); sort2!(20, 21);
            sort2!(23, 44); sort2!(24, 28); sort2!(25, 26); sort2!(27, 31); sort2!(29, 30);
            sort2!(32, 36); sort2!(33, 34); sort2!(35, 39); sort2!(37, 38); sort2!(40, 59);
            sort2!(41, 42); sort2!(43, 62); sort2!(45, 46); sort2!(47, 58); sort2!(49, 50);
            sort2!(53, 54); sort2!(56, 57); sort2!(60, 61); sort2!(0, 8);   sort2!(1, 20);
            sort2!(2, 21);  sort2!(3, 44);  sort2!(4, 40);  sort2!(5, 17);  sort2!(6, 18);
            sort2!(7, 43);  sort2!(9, 49);  sort2!(10, 50); sort2!(11, 22); sort2!(12, 24);
            sort2!(13, 53); sort2!(14, 54); sort2!(15, 27); sort2!(16, 28); sort2!(19, 31);
            sort2!(23, 32); sort2!(25, 29); sort2!(26, 30); sort2!(33, 37); sort2!(34, 38);
            sort2!(35, 47); sort2!(36, 48); sort2!(39, 51); sort2!(41, 60); sort2!(42, 61);
            sort2!(45, 56); sort2!(46, 57); sort2!(52, 59); sort2!(55, 62); sort2!(0, 23);
            sort2!(1, 9);   sort2!(2, 10);  sort2!(3, 36);  sort2!(4, 12);  sort2!(5, 41);
            sort2!(6, 42);  sort2!(7, 15);  sort2!(8, 32);  sort2!(11, 35); sort2!(13, 25);
            sort2!(14, 26); sort2!(16, 52); sort2!(17, 29); sort2!(18, 30); sort2!(19, 55);
            sort2!(20, 56); sort2!(21, 57); sort2!(22, 47); sort2!(24, 40); sort2!(27, 43);
            sort2!(28, 59); sort2!(31, 62); sort2!(33, 45); sort2!(34, 46); sort2!(37, 49);
            sort2!(38, 50); sort2!(39, 58); sort2!(44, 48); sort2!(53, 60); sort2!(54, 61);
            sort2!(0, 4);   sort2!(1, 33);  sort2!(2, 34);  sort2!(3, 24);  sort2!(5, 13);
            sort2!(6, 14);  sort2!(7, 11);  sort2!(8, 16);  sort2!(9, 45);  sort2!(10, 46);
            sort2!(12, 23); sort2!(15, 35); sort2!(17, 53); sort2!(18, 54); sort2!(19, 22);
            sort2!(20, 37); sort2!(21, 38); sort2!(25, 41); sort2!(26, 42); sort2!(27, 39);
            sort2!(28, 44); sort2!(29, 60); sort2!(30, 61); sort2!(31, 51); sort2!(32, 52);
            sort2!(36, 40); sort2!(43, 58); sort2!(47, 55); sort2!(48, 59); sort2!(49, 56);
            sort2!(50, 57); sort2!(1, 5);   sort2!(2, 6);   sort2!(3, 8);   sort2!(4, 12);
            sort2!(9, 17);  sort2!(10, 18); sort2!(11, 15); sort2!(13, 33); sort2!(14, 34);
            sort2!(16, 23); sort2!(19, 27); sort2!(20, 25); sort2!(21, 26); sort2!(22, 35);
            sort2!(24, 36); sort2!(28, 32); sort2!(29, 49); sort2!(30, 50); sort2!(31, 47);
            sort2!(37, 41); sort2!(38, 42); sort2!(39, 43); sort2!(40, 52); sort2!(44, 48);
            sort2!(45, 53); sort2!(46, 54); sort2!(51, 62); sort2!(55, 58); sort2!(56, 60);
            sort2!(57, 61); sort2!(3, 4);   sort2!(5, 13);  sort2!(6, 14);  sort2!(8, 12);
            sort2!(9, 20);  sort2!(10, 21); sort2!(11, 19); sort2!(15, 27); sort2!(16, 32);
            sort2!(17, 33); sort2!(18, 34); sort2!(22, 47); sort2!(23, 24); sort2!(25, 37);
            sort2!(26, 38); sort2!(28, 36); sort2!(29, 45); sort2!(30, 46); sort2!(31, 43);
            sort2!(35, 39); sort2!(40, 44); sort2!(41, 53); sort2!(42, 54); sort2!(48, 52);
            sort2!(49, 56); sort2!(50, 57); sort2!(51, 55); sort2!(58, 62); sort2!(4, 8);
            sort2!(5, 9);   sort2!(6, 10);  sort2!(12, 16); sort2!(13, 20); sort2!(14, 21);
            sort2!(15, 19); sort2!(17, 45); sort2!(18, 46); sort2!(22, 27); sort2!(23, 28);
            sort2!(24, 36); sort2!(25, 33); sort2!(26, 34); sort2!(29, 37); sort2!(30, 38);
            sort2!(31, 35); sort2!(32, 40); sort2!(39, 43); sort2!(41, 49); sort2!(42, 50);
            sort2!(44, 48); sort2!(47, 51); sort2!(53, 56); sort2!(54, 57); sort2!(55, 58);
            sort2!(9, 13);  sort2!(10, 14); sort2!(12, 23); sort2!(16, 28); sort2!(17, 20);
            sort2!(18, 21); sort2!(22, 31); sort2!(24, 32); sort2!(25, 29); sort2!(26, 30);
            sort2!(27, 35); sort2!(33, 37); sort2!(34, 38); sort2!(36, 40); sort2!(39, 47);
            sort2!(41, 45); sort2!(42, 46); sort2!(43, 51); sort2!(49, 53); sort2!(50, 54);
            sort2!(8, 12);  sort2!(16, 23); sort2!(17, 25); sort2!(18, 26); sort2!(19, 22);
            sort2!(20, 29); sort2!(21, 30); sort2!(24, 28); sort2!(27, 31); sort2!(32, 36);
            sort2!(33, 41); sort2!(34, 42); sort2!(35, 39); sort2!(37, 45); sort2!(38, 46);
            sort2!(40, 44); sort2!(43, 47); sort2!(51, 55); sort2!(1, 12);  sort2!(2, 28);
            sort2!(5, 16);  sort2!(6, 32);  sort2!(9, 23);  sort2!(10, 36); sort2!(13, 17);
            sort2!(14, 18); sort2!(20, 25); sort2!(21, 26); sort2!(27, 53); sort2!(29, 33);
            sort2!(30, 34); sort2!(31, 56); sort2!(35, 60); sort2!(37, 41); sort2!(38, 42);
            sort2!(43, 54); sort2!(45, 49); sort2!(46, 50); sort2!(47, 57); sort2!(51, 61);
            sort2!(1, 3);   sort2!(2, 4);   sort2!(6, 8);   sort2!(7, 33);  sort2!(10, 23);
            sort2!(11, 37); sort2!(13, 24); sort2!(14, 40); sort2!(15, 41); sort2!(18, 44);
            sort2!(19, 45); sort2!(20, 32); sort2!(21, 48); sort2!(22, 49); sort2!(26, 52);
            sort2!(30, 59); sort2!(31, 42); sort2!(39, 50); sort2!(43, 53); sort2!(55, 56);
            sort2!(58, 60); sort2!(61, 62); sort2!(2, 3);   sort2!(4, 12);  sort2!(5, 6);
            sort2!(7, 17);  sort2!(8, 16);  sort2!(11, 21); sort2!(14, 24); sort2!(15, 25);
            sort2!(18, 28); sort2!(19, 29); sort2!(22, 33); sort2!(26, 36); sort2!(27, 37);
            sort2!(30, 40); sort2!(34, 44); sort2!(35, 45); sort2!(38, 48); sort2!(39, 49);
            sort2!(41, 52); sort2!(46, 59); sort2!(47, 55); sort2!(51, 58); sort2!(56, 57);
            sort2!(60, 61); sort2!(7, 18);  sort2!(11, 20); sort2!(15, 28); sort2!(17, 26);
            sort2!(19, 30); sort2!(21, 32); sort2!(22, 25); sort2!(27, 29); sort2!(31, 41);
            sort2!(33, 44); sort2!(34, 36); sort2!(35, 48); sort2!(37, 46); sort2!(38, 40);
            sort2!(42, 52); sort2!(45, 59); sort2!(7, 12);  sort2!(11, 16); sort2!(15, 23);
            sort2!(17, 18); sort2!(19, 20); sort2!(21, 24); sort2!(22, 34); sort2!(25, 36);
            sort2!(26, 28); sort2!(27, 38); sort2!(29, 40); sort2!(30, 32); sort2!(31, 33);
            sort2!(35, 37); sort2!(39, 41); sort2!(42, 44); sort2!(43, 48); sort2!(45, 46);
            sort2!(47, 52); sort2!(51, 59); sort2!(7, 9);   sort2!(10, 12); sort2!(11, 13);
            sort2!(14, 16); sort2!(15, 17); sort2!(18, 23); sort2!(19, 21); sort2!(20, 24);
            sort2!(22, 26); sort2!(25, 28); sort2!(27, 30); sort2!(29, 32); sort2!(31, 34);
            sort2!(33, 36); sort2!(35, 38); sort2!(37, 40); sort2!(39, 42); sort2!(41, 44);
            sort2!(43, 45); sort2!(46, 48); sort2!(47, 49); sort2!(50, 52); sort2!(51, 53);
            sort2!(54, 59); sort2!(4, 7);   sort2!(8, 9);   sort2!(10, 11); sort2!(12, 15);
            sort2!(13, 14); sort2!(16, 18); sort2!(17, 19); sort2!(20, 23); sort2!(21, 22);
            sort2!(24, 25); sort2!(26, 27); sort2!(28, 30); sort2!(29, 31); sort2!(32, 34);
            sort2!(33, 35); sort2!(36, 37); sort2!(38, 39); sort2!(40, 41); sort2!(42, 43);
            sort2!(44, 46); sort2!(45, 47); sort2!(48, 51); sort2!(49, 50); sort2!(52, 53);
            sort2!(54, 55); sort2!(58, 59); sort2!(4, 5);   sort2!(6, 7);   sort2!(8, 10);
            sort2!(9, 11);  sort2!(12, 13); sort2!(14, 15); sort2!(16, 17); sort2!(18, 19);
            sort2!(20, 21); sort2!(22, 23); sort2!(24, 26); sort2!(25, 27); sort2!(28, 29);
            sort2!(30, 31); sort2!(32, 33); sort2!(34, 35); sort2!(36, 38); sort2!(37, 39);
            sort2!(40, 42); sort2!(41, 43); sort2!(44, 45); sort2!(46, 47); sort2!(48, 49);
            sort2!(50, 51); sort2!(52, 54); sort2!(53, 55); sort2!(56, 58); sort2!(57, 59);
            sort2!(3, 4);   sort2!(5, 6);   sort2!(7, 8);   sort2!(9, 10);  sort2!(11, 12);
            sort2!(13, 14); sort2!(15, 16); sort2!(17, 18); sort2!(19, 20); sort2!(21, 22);
            sort2!(23, 24); sort2!(25, 26); sort2!(27, 28); sort2!(29, 30); sort2!(31, 32);
            sort2!(33, 34); sort2!(35, 36); sort2!(37, 38); sort2!(39, 40); sort2!(41, 42);
            sort2!(43, 44); sort2!(45, 46); sort2!(47, 48); sort2!(49, 50); sort2!(51, 52);
            sort2!(53, 54); sort2!(55, 56); sort2!(57, 58); sort2!(59, 60); 
        } _ => unimplemented!("sort not implemented for N={N}"),
    } 
}
