use std::mem::ManuallyDrop;

use crate::sort::sort_at;

fn parent(index: usize) -> usize {
    (index - 1) / 2
}

fn left_child(index: usize) -> usize {
    2 * index + 1
}

fn right_child(index: usize) -> usize {
    2 * index + 2
}

fn leaf_search<T, F>(heap: &mut [T], index: usize, is_less: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let mut j = index;
    let end = heap.len() - 1;
    while right_child(j) <= end {
        if is_less(&heap[left_child(j)], &heap[right_child(j)]) {
            j = right_child(j);
        } else {
            j = left_child(j);
        }
    }
    if left_child(j) <= end {
        j = left_child(j);
    }
    j
}

fn sift_down<T, F>(heap: &mut [T], index: usize, is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let l = heap.as_mut_ptr();
    let mut j = leaf_search(heap, index, is_less);
    unsafe {
        while is_less(&*l.add(j), &*l.add(index)) {
            j = parent(j);
        }
        let mut x = ManuallyDrop::new(core::ptr::read(l.add(j)));
        l.add(j).copy_from(l.add(index), 1);
        while j > index {
            let p = parent(j);
            core::ptr::swap_nonoverlapping(&mut heap[p], &mut *x, 1);
            j = parent(j);
        }
    }
}

// fn sift_up<T, F>(heap: &mut [T], start: usize, end: usize, is_less: &mut F)
// where
//     F: FnMut(&T, &T) -> bool,
// {
//     let mut child = end;
//     while child > start {
//         let parent = parent(child);
//         if is_less(&heap[parent], &heap[child]) {
//             heap.swap(child, parent);
//             child = parent;
//         } else {
//             return;
//         }
//     }
// }

fn max_heap<T, F>(data: &mut [T], is_less: &mut F)
where
    F: FnMut(&T, &T) -> bool,
{
    let mut index = data.len() / 2;
    while index > 0 {
        index -= 1;
        sift_down(data, index, is_less);
    }
}

fn pop<'a, T, F>(heap: &'a mut [T], is_less: &mut F) -> (&'a mut T, &'a mut [T])
where
    F: FnMut(&T, &T) -> bool,
{
    heap.swap(0, heap.len() - 1);
    let (elem, data) = heap.split_last_mut().unwrap();
    sift_down(data, 0, is_less);
    (elem, data)
}

pub(crate) fn heapselect<T, F>(data: &mut [T], index: usize, is_less: &mut F) -> (usize, usize)
where
    F: FnMut(&T, &T) -> bool,
{
    let last = data.len() - 1;
    let (low, high) = data.split_at_mut(index);
    max_heap(low, is_less);
    max_heap(high, &mut |a, b| is_less(b, a));
    match index {
        0 => {}
        index if index == last => data.swap(0, index),
        _ => {
            while is_less(&high[0], &low[0]) {
                core::mem::swap(&mut high[0], &mut low[0]);
                sift_down(low, 0, is_less);
                sift_down(high, 0, &mut |a, b| is_less(b, a));
            }
        }
    }
    (index, index)
}

fn check_heap<T, F>(data: &[T], is_less: &mut F)
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

#[test]
fn build() {
    let mut data = [2, 4, 5, 7, 1, 3, 6, 8, 2, 1, 9];
    max_heap(&mut data, &mut i32::lt);
    check_heap(&data, &mut i32::lt);

    let (elem, data) = pop(&mut data, &mut i32::lt);
    eprintln!("{elem}");
    check_heap(data, &mut i32::lt);
}
