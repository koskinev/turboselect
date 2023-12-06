#[inline]
/// Compares the elements at `a` and `b` and swaps them if `a` is greater than `b`. Returns `true`
/// if the elements were swapped. Panics if `a` or `b` is out of bounds or `a == b`.
fn sort2<T, F>(data: &mut [T], a: usize, b: usize, lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    if b < data.len() && a < b {
        let ptr = data.as_mut_ptr();
        unsafe {
            let (min, max) = if lt(&*ptr.add(b), &*ptr.add(a)) {
                (ptr.add(b), ptr.add(a).read())
            } else {
                (ptr.add(a), ptr.add(b).read())
            };
            ptr.add(a).copy_from(min, 1);
            ptr.add(b).write(max);
        }
    }
}

#[rustfmt::skip]
fn sort<T, F, const N: usize>(data: &mut [T], lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    macro_rules! sort2 {
        ($a:expr, $b:expr) => {
            sort2(data, $a, $b, lt);
        };
    }

    match N {
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

/// Sorts the slice `data` using the given comparison function `lt`. For slice lengths of 16 or
/// less, a sorting network is used. For larger slices, a bitonic sorter is used.
pub(crate) fn tinysort<T, F>(data: &mut [T], lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    match data.len() {
        0 | 1 => {}
        2 => sort2(data, 0, 1, lt),
        3 => sort::<_, _, 3>(data, lt),
        4 => sort::<_, _, 4>(data, lt),
        5 => sort::<_, _, 5>(data, lt),
        6 => sort::<_, _, 6>(data, lt),
        7 => sort::<_, _, 7>(data, lt),
        8 => sort::<_, _, 8>(data, lt),
        9 => sort::<_, _, 9>(data, lt),
        10 => sort::<_, _, 10>(data, lt),
        11 => sort::<_, _, 11>(data, lt),
        12 => sort::<_, _, 12>(data, lt),
        13 => sort::<_, _, 13>(data, lt),
        14 => sort::<_, _, 14>(data, lt),
        15 => sort::<_, _, 15>(data, lt),
        16 => sort::<_, _, 16>(data, lt),
        len => {
            let mut size = 16;
            data.chunks_mut(size).for_each(|chunk| tinysort(chunk, lt));
            while size < len {
                size *= 2;
                for chunk in data.chunks_mut(size) {
                    merge(chunk, lt);
                }
            }
        }
    }
}

fn merge<T, F>(chunk: &mut [T], lt: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let size = chunk.len().next_power_of_two();
    let half = size / 2;
    for delta in 0..half {
        sort2(chunk, delta, size - delta - 1, lt);
    }
    let mut part = half;
    while part > 16 {
        for inner in chunk.chunks_mut(part) {
            for index in 0..(part / 2) {
                sort2(inner, index, index + (part / 2), lt);
            }
        }
        part /= 2;
    }
    for inner in chunk.chunks_mut(16) {
        sort2(inner, 0, 8, lt);
        sort2(inner, 1, 9, lt);
        sort2(inner, 2, 10, lt);
        sort2(inner, 3, 11, lt);
        sort2(inner, 4, 12, lt);
        sort2(inner, 5, 13, lt);
        sort2(inner, 6, 14, lt);
        sort2(inner, 7, 15, lt);

        sort2(inner, 0, 4, lt);
        sort2(inner, 1, 5, lt);
        sort2(inner, 2, 6, lt);
        sort2(inner, 3, 7, lt);
        sort2(inner, 8, 12, lt);
        sort2(inner, 9, 13, lt);
        sort2(inner, 10, 14, lt);
        sort2(inner, 11, 15, lt);

        sort2(inner, 0, 2, lt);
        sort2(inner, 1, 3, lt);
        sort2(inner, 4, 6, lt);
        sort2(inner, 5, 7, lt);
        sort2(inner, 8, 10, lt);
        sort2(inner, 9, 11, lt);
        sort2(inner, 12, 14, lt);
        sort2(inner, 13, 15, lt);

        sort2(inner, 0, 1, lt);
        sort2(inner, 2, 3, lt);
        sort2(inner, 4, 5, lt);
        sort2(inner, 6, 7, lt);
        sort2(inner, 8, 9, lt);
        sort2(inner, 10, 11, lt);
        sort2(inner, 12, 13, lt);
        sort2(inner, 14, 15, lt);
    }
}
