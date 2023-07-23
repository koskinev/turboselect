# Turboselect

Turboselect is an alternative implementation of the `slice::select_nth_unstable` method, which is used to find the nth smallest element in a slice. It is based on the Floyd & Rivest SELECT algorithm and demonstrates speed improvements over the Quickselect implementation in the standard library, particulary for indices far from the median. The algorithm works by recursively selecting a pivot element to compare against and partitioning the slice into smaller parts. The implementation is partially based on K. Kiwiel's paper "On Floyd and Rivest's SELECT algorithm" [2],[3].

Like `slice::select_nth_unstable`, `turboselect::select_nth_unstable` takes a mutable slice and an index, and reorders the elements so that the element at the given index is at its final position. After the call, the elements before the index are less than or equal to the element at the index, and the elements after the index are greater than or equal to the element at the index.

```rust
    let (_, nth, _) = turboselect::select_nth_unstable(data, index);
  // ┌─────────────────┬─────────────────┬────────────────┐
  // │ &data[i] <= nth │ &data[i] == nth │ data[i] >= nth │
  // └─────────────────┴─────────────────┴────────────────┘
  //     i < index          i == index       i > index           
```

In addition to the `select_nth_unstable`, the following methods are provided:
- `select_nth_unstable_by_key`, which takes a key extraction function as an argument.
- `select_nth_unstable_by_cached_key`, which is similar to `select_nth_unstable_by_key`, but caches the keys in a temporary buffer. This is useful if the key extraction is expensive. 
  
The implementation relies heavily on unsafe code and is currently not thoroughly tested. To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**
 
| slice length | index   | throughput | baseline | ratio |
| ------------ | ------- | ---------- | -------- | ----- |
| 1 000        | 1       | 1255.193   | 658.895  | 1.905 |
| 1 000        | 50      | 924.130    | 629.538  | 1.468 |
| 1 000        | 500     | 633.066    | 552.940  | 1.145 |
| 10 000       | 10      | 2131.869   | 892.944  | 2.387 |
| 10 000       | 500     | 1400.510   | 893.016  | 1.568 |
| 10 000       | 5 000   | 943.022    | 740.596  | 1.273 |
| 100 000      | 100     | 2633.922   | 972.657  | 2.708 |
| 100 000      | 5 000   | 1815.562   | 939.621  | 1.932 |
| 100 000      | 50 000  | 1204.720   | 804.198  | 1.498 |
| 1 000 000    | 1 000   | 2737.727   | 972.119  | 2.816 |
| 1 000 000    | 50 000  | 2093.268   | 955.093  | 2.192 |
| 1 000 000    | 500 000 | 1336.267   | 818.857  | 1.632 |

Throughput is calculated as millions of elements per second, i.e `data.len() * runs / seconds`. 

This comparison was run on a Ryzen 5800H with pseudorandom `u32`s used as input. Similar tests were run with sawtooth, sorted and reverse sorted inputs, as well as pseudorandom `u32`s with many duplicates, and pseudorandom booleans. Turboselect outperformed Quickselect in all except one case, median of 1000-element sawtooth pattern where the throughput ratio was approximately 0.97.

You can run the benchmarks with `cargo test -r turboselect_perf -- --nocapture --ignored`.

## Notes

The speed improvements are mostly due to pivot selection. In Quickselect, median of medians is usually used, which tends to put the pivot near the middle of the slice. This about halves the size of the unordered part of the slice. Turboselect biases the selection towards the desired index, in order to reduce the size of the unordered part of the slice as much as possible without overshooting. 

The difference is illustrated in the following diagram, where the levels of blocks represent the unordered part of the slice in each step of the iteration. The index is marked with a `":"`, the position of the pivot after partitioning with a `"|"`, and the ordered part of the slice with `░░`. Turboselect's pivot "hugs" the index, and the unordered part of the slice shrinks more quickly, reducing both the number of iterations and the number of elements examined in each iteration.

```text 
    Quickselect:                     |  Turboselect:
                                     |
      index                          |    index
        :                            |      :
    0123456789abcdefghijklmnopqrstu  |  0123456789abcdefghijklmnopqrstu
   ┌───────────────────────────────┐ | ┌───────────────────────────────┐
 0 │    :           |░░░░░░░░░░░░░░│ | │   :  |░░░░░░░░░░░░░░░░░░░░░░░░│
   ├────────────────┬──────────────┘ | ├──────┬────────────────────────┘
 1 │    :   |░░░░░░░│                | │░░|:  │
   ├────────┬───────┘                | └──┬───┤
 2 │░░░|:   │                        |    │:|░│
   └───┬────┤                        |    ├─┬─┘
 3     │:|░░│                        |    │:│
       ├─┬──┘                        |    └─┘
 4     │ │                           |
       └─┘                           |
```

Pivot selection starts with putting a random sample of elements to the beginning of the slice. When the slice is large, this is followed with a recursive call to Turboselect. For small slices, an extension of median-of-medians inspired by A. Alexandrescu's paper "Fast Deterministic Selection" [1], is used. See the method `kth_of_nths` for details.

The sampling allows the pivot selection to detect when the pivot is likely to have many duplicates. In these cases, the slice is split into three parts:

```text
 Equal-partitioning:
┌─────────────┬──────────────┬─────────────┐
│ x < data[u] │ x == data[u] │ x > data[u] │
└─────────────┴──────────────┴─────────────┘
               u            v
```

Kiwiel suggests a dual-pivot partitioning scheme, but it is not used, since it turned out to be slower than successive calls to single pivot partitions. An implementation is available in the commit history.

## About the name

"Turbo" in the name refers to the complexity added in order to gain some speed. It is not a *guarantee* of speed, as there are plenty of slow turbo things out there.

## References

[1] Alexandrescu, Andrei. (2017). Fast Deterministic Selection. [10.4230/LIPIcs.SEA.2017.24](https://dx.doi.org/10.4230/LIPIcs.SEA.2017.24). 
[2] Floyd, Robert & Rivest, Ronald. (1975). Algorithm 489: The algorithm SELECT—for finding the ith smallest of n elements. Communications of the ACM. 18. [10.1145/360680.360694](https://dx.doi.org/10.1145/360680.360694). 
[3] Kiwiel, Krzysztof. (2005). On Floyd and Rivest's SELECT algorithm. Theoretical Computer Science. 347. 214-238. [10.1016/j.tcs.2005.06.032](https://dx.doi.org/10.1016/j.tcs.2005.06.032).  