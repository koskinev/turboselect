use core::convert::identity;

#[inline]
/// Compares the elements at `a` and `b` and swaps them if `a` is greater than `b`. Returns `true`
/// if the elements were swapped. Panics if `a` or `b` is out of bounds or `a == b`.
fn sort2<T, F>(data: &mut [T], a: usize, b: usize, lt: &mut F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    if b < data.len() && a < b {
        let ptr = data.as_mut_ptr();
        unsafe {
            let swap = lt(&*ptr.add(b), &*ptr.add(a));
            let (min, max) = if swap {
                (ptr.add(b), ptr.add(a).read())
            } else {
                (ptr.add(a), ptr.add(b).read())
            };
            ptr.add(a).copy_from(min, 1);
            ptr.add(b).write(max);
            return swap;
        }
    }
    false
}

#[rustfmt::skip]
pub(crate) fn sort_at<T, M, F>(data: &mut [T], map: &M, n: usize, lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
    M: Fn(usize) -> usize,
{
    let mut pos: [usize; 16] = [0; 16];
    for (index, pos) in pos.iter_mut().take(n).enumerate() {
        let mapped = map(index);
        assert!(mapped < data.len());
        *pos = mapped;
    }

    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            sort2(data, pos[$a], pos[$b], lt);
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
            let mut size = 16;
            for chunk in data.chunks_mut(size) {
                tinysort(chunk, lt);
            }
            while size < len {
                size *= 2;
                for chunk in data.chunks_mut(size) {
                    let (low, high) = (0, size - 1);
                    for d in 0..size / 2 {
                        sort2(chunk, low + d, high - d, lt);
                    }
                    let mut part_size = size / 2;
                    while part_size > 1 {
                        for part in chunk.chunks_mut(part_size) {
                            let d = part.len().next_power_of_two() / 2;
                            for (low, high) in (0..d).map(|i| (i, i + d)) {
                                sort2(part, low, high, lt);
                            }
                        }
                        part_size /= 2;
                    }
                }
            }
        }
    }
}