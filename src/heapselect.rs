use std::mem::ManuallyDrop;

use crate::sort::sort_at;

const fn ilog2(n: usize) -> usize {
    core::mem::size_of::<usize>() * 8 - n.leading_zeros() as usize
}

fn parent(index: usize) -> usize {
    (index - 1) / 2
}

fn left_child(index: usize) -> usize {
    2 * index + 1
}

fn right_child(index: usize) -> usize {
    2 * index + 2
}

fn max_leaf<T, F>(heap: &mut [T], index: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let mut p = index;
    let end = heap.len() - 1;
    let mut r = right_child(p);
    while r <= end {
        let gt = is_less(&heap[r - 1], &heap[r]) as usize;
        p = r * gt + (r - 1) * (1 - gt);
        r = right_child(p);
    }
    let has_left = (r - 1 <= end) as usize;
    has_left * (r - 1) + (1 - has_left) * p
}

fn min_leaf<T, F>(heap: &mut [T], index: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let mut p = index;
    let end = heap.len() - 1;
    let mut r = right_child(p);
    while r <= end {
        let lt = is_less(&heap[r], &heap[r - 1]) as usize;
        p = r * lt + (r - 1) * (1 - lt);
        r = right_child(p);
    }
    let has_left = (r - 1 <= end) as usize;
    has_left * (r - 1) + (1 - has_left) * p
}

fn push_down_max<T, F>(heap: &mut [T], index: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut n = max_leaf(heap, index, is_less);
    let l = heap.as_mut_ptr();
    unsafe {
        let mut x = ManuallyDrop::new(core::ptr::read(l.add(index)));
        while is_less(&*l.add(n), &*x) {
            n = parent(n);
        }
        core::ptr::swap_nonoverlapping(&mut *l.add(n), &mut *x, 1);
        while n > index {
            let p = parent(n);
            core::ptr::swap_nonoverlapping(&mut *l.add(p), &mut *x, 1);
            n = parent(n);
        }
    }
}

fn push_down_min<T, F>(heap: &mut [T], index: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut n = min_leaf(heap, index, is_less);
    let l = heap.as_mut_ptr();
    unsafe {
        let mut x = ManuallyDrop::new(core::ptr::read(l.add(index)));
        while is_less(&*x, &*l.add(n)) {
            n = parent(n);
        }
        core::ptr::swap_nonoverlapping(&mut *l.add(n), &mut *x, 1);
        while n > index {
            let p = parent(n);
            core::ptr::swap_nonoverlapping(&mut *l.add(p), &mut *x, 1);
            n = parent(n);
        }
    }
}

fn max_heap<T, F>(data: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut index = data.len() / 2;
    while index > 0 {
        index -= 1;
        push_down_max(data, index, is_less);
    }
}

fn min_heap<T, F>(data: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut index = data.len() / 2;
    while index > 0 {
        index -= 1;
        push_down_min(data, index, is_less);
    }
}

pub(crate) fn heapselect<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    match data.len() {
        0 | 1 => {}
        2 => sort_at(data, [0, 1], is_less),
        3 => sort_at(data, [0, 1, 2], is_less),
        4 => sort_at(data, [0, 1, 2, 3], is_less),
        _ => {
            let (low, high) = data.split_at_mut(index);
            max_heap(low, is_less);
            min_heap(high, is_less);
            match index {
                0 => {}
                _ => {
                    while is_less(&high[0], &low[0]) {
                        core::mem::swap(&mut high[0], &mut low[0]);
                        push_down_max(low, 0, is_less);
                        push_down_min(high, 0, is_less);
                    }
                }
            }
        }
    }
    (index, index)
}

fn _check_heap<T, F>(data: &[T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let len = data.len();
    for i in 0..len / 2 {
        let l = left_child(i);
        let r = right_child(i);
        if l < len {
            assert!(!is_less(&data[i], &data[l]));
        }
        if r < len {
            assert!(!is_less(&data[i], &data[r]));
        }
    }
}

mod minmax {
    fn is_new_item_min(length: usize) -> bool {
        (length.ilog2() & 1) == 0
    }

    fn is_min_item(index: usize) -> bool {
        is_new_item_min(index + 1)
    }

    fn grandparent_index(index: usize) -> usize {
        (index - 3) / 4
    }

    fn parent_index(index: usize) -> usize {
        (index - 1) / 2
    }

    fn first_child_index(index: usize) -> usize {
        (index * 2) + 1
    }

    fn last_grandchild_index(index: usize) -> usize {
        (index * 4) + 6
    }

    fn smallest_descendant<T, F>(
        begin: &[T],
        first_child: usize,
        first_grandchild: usize,
        is_less: &mut F,
    ) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        let length = begin.len();
        let second_child = first_child + 1;
        if first_grandchild >= length {
            return first_child
                + (second_child != length && is_less(&begin[second_child], &begin[first_child]))
                    as usize;
        }
        let second_grandchild = first_grandchild + 1;
        if second_grandchild == length {
            return if is_less(&begin[first_grandchild], &begin[second_child]) {
                first_grandchild
            } else {
                second_child
            };
        }
        let min_grandchild = first_grandchild
            + (is_less(&begin[second_grandchild], &begin[first_grandchild]) as usize);
        let third_grandchild = second_grandchild + 1;
        if third_grandchild == length {
            if is_less(&begin[min_grandchild], &begin[second_child]) {
                min_grandchild
            } else {
                second_child
            }
        } else if is_less(&begin[min_grandchild], &begin[third_grandchild]) {
            min_grandchild
        } else {
            third_grandchild
        }
    }

    fn largest_descendant<T, F>(
        begin: &[T],
        first_child: usize,
        first_grandchild: usize,
        is_less: &mut F,
    ) -> usize
    where
        F: FnMut(&T, &T) -> bool,
    {
        let length = begin.len();
        let second_child = first_child + 1;
        if first_grandchild >= length {
            return first_child
                + (second_child != length && is_less(&begin[first_child], &begin[second_child]))
                    as usize;
        }
        let second_grandchild = first_grandchild + 1;
        if second_grandchild == length {
            return if is_less(&begin[second_child], &begin[first_grandchild]) {
                first_grandchild
            } else {
                second_child
            };
        }
        let max_grandchild = first_grandchild
            + (is_less(&begin[first_grandchild], &begin[second_grandchild]) as usize);
        let third_grandchild = second_grandchild + 1;
        if third_grandchild == length {
            if is_less(&begin[second_child], &begin[max_grandchild]) {
                max_grandchild
            } else {
                second_child
            }
        } else if is_less(&begin[max_grandchild], &begin[third_grandchild]) {
            third_grandchild
        } else {
            max_grandchild
        }
    }

    fn push_down_min<T, F>(begin: &mut [T], value: *mut T, mut index: usize, mut is_less: F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let len = begin.len();
        let ptr = begin.as_mut_ptr();
        loop {
            let last_grandchild = last_grandchild_index(index);
            if last_grandchild < len {
                unsafe {
                    let it = ptr.add(last_grandchild);
                    let min_first_half = last_grandchild - 2 - is_less(&*it, &*it.add(1)) as usize;
                    let min_second_half =
                        last_grandchild - is_less(&*it.add(2), &*it.add(3)) as usize;
                    let smallest = if is_less(&*ptr.add(min_second_half), &*ptr.add(min_first_half))
                    {
                        min_second_half
                    } else {
                        min_first_half
                    };
                    if !is_less(&*ptr.add(smallest), &*value) {
                        break;
                    }
                    ptr.add(index)
                        .copy_from_nonoverlapping(ptr.add(smallest), 1);
                    index = smallest;
                    let parent = parent_index(index);
                    if is_less(&*ptr.add(parent), &*value) {
                        core::ptr::swap_nonoverlapping(ptr.add(parent), value, 1);
                    }
                }
            } else {
                let first_child = first_child_index(index);
                if first_child >= len {
                    break;
                }
                let first_grandchild = last_grandchild - 3;
                let smallest =
                    smallest_descendant(begin, first_child, first_grandchild, &mut is_less);
                unsafe {
                    if !is_less(&*ptr.add(smallest), &*value) {
                        break;
                    }
                    ptr.add(index)
                        .copy_from_nonoverlapping(ptr.add(smallest), 1);
                    index = smallest;
                    if smallest < first_grandchild {
                        break;
                    }
                    let parent = parent_index(index);
                    if is_less(&*ptr.add(parent), &*value) {
                        ptr.add(index).copy_from_nonoverlapping(ptr.add(parent), 1);
                        index = parent;
                    }
                    break;
                }
            }
        }
        unsafe {
            ptr.add(index).write(value.read());
        }
    }

    fn push_down_min_one_child_only<T, F>(begin: &mut [T], index: usize, mut is_less: F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let child = first_child_index(index);
        if is_less(&begin[child], &begin[index]) {
            begin.swap(index, child);
        }
    }

    fn push_down_min_one_level_only<T, F>(begin: &mut [T], index: usize, mut is_less: F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let first_child = first_child_index(index);
        let smaller_child =
            first_child + (is_less(&begin[first_child + 1], &begin[first_child]) as usize);
        if is_less(&begin[smaller_child], &begin[index]) {
            begin.swap(index, smaller_child);
        }
    }

    fn push_down_max<T, F>(begin: &mut [T], value: *mut T, mut index: usize, mut compare: F)
    where
        F: FnMut(&T, &T) -> bool,
    {
        let length = begin.len();
        let ptr = begin.as_mut_ptr();
        loop {
            let last_grandchild = last_grandchild_index(index);
            if last_grandchild < length {
                unsafe {
                    let it = ptr.add(last_grandchild);
                    let max_first_half = last_grandchild - 2 - compare(&*it, &*it.add(1)) as usize;
                    let max_second_half =
                        last_grandchild - compare(&*it.add(2), &*it.add(3)) as usize;
                    let largest = if compare(&*ptr.add(max_first_half), &*ptr.add(max_second_half))
                    {
                        max_second_half
                    } else {
                        max_first_half
                    };
                    if !compare(&*value, &*ptr.add(largest)) {
                        break;
                    }
                    ptr.add(index).copy_from_nonoverlapping(ptr.add(largest), 1);
                    index = largest;
                    let parent = parent_index(index);
                    if compare(&*value, &*ptr.add(parent)) {
                        core::ptr::swap_nonoverlapping(ptr.add(parent), value, 1);
                    }
                }
            } else {
                let first_child = first_child_index(index);
                if first_child >= length {
                    break;
                }
                let first_grandchild = last_grandchild - 3;
                let largest =
                    largest_descendant(begin, first_child, first_grandchild, &mut compare);
                unsafe {
                    if !compare(&*value, &*ptr.add(largest)) {
                        break;
                    }
                    ptr.add(index).copy_from_nonoverlapping(ptr.add(largest), 1);
                    index = largest;
                    if largest < first_grandchild {
                        break;
                    }
                    let parent = parent_index(index);
                    if compare(&*value, &*ptr.add(parent)) {
                        ptr.add(index).copy_from_nonoverlapping(ptr.add(parent), 1);
                        index = parent;
                    }
                }
                break;
            }
        }
        unsafe {
            ptr.add(index).write(value.read());
        }
    }
}
