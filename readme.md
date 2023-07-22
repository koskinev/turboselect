# Turboselect

An alternative to the `slice::select_nth_unstable` method, based on the Floyd & Rivest SELECT algorithm, demonstrating improved speed over the Quickselect implementation of the standard library. The improvements are most noticeable for indices far from the median. 

Just as  `slice::select_nth_unstable`, `turboselect::select_nth_unstable` takes a mutable slice and an index, and reorders the elements so that the element at the given index is at its final position. The elements before the index are less than or equal to the element at the index, and the elements after the index are greater than or equal to the element at the index:

```rust
    let nth = turboselect::select_nth_unstable(data, index);
  // ┌─────────────────┬─────────────────┬────────────────┐
  // │ &data[i] <= nth │ &data[i] == nth │ data[i] >= nth │
  // └─────────────────┴─────────────────┴────────────────┘
  //     i < index          i == index       i > index           
```

Note that the code relies heavily on unsafe code and is currently not thoroughly tested. To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| slice length | index  | throughput | baseline | ratio |
| ------------ | ------ | ---------- | -------- | ----- |
| 1000         | 1      | 1276.316   | 649.252  | 1.966 |
| 1000         | 10     | 1203.315   | 653.992  | 1.840 |
| 1000         | 50     | 953.050    | 630.743  | 1.511 |
| 1000         | 250    | 697.324    | 586.512  | 1.189 |
| 1000         | 500    | 652.465    | 562.908  | 1.159 |
| 10000        | 10     | 2284.529   | 935.908  | 2.441 |
| 10000        | 100    | 1843.303   | 934.396  | 1.973 |
| 10000        | 500    | 1388.795   | 892.037  | 1.557 |
| 10000        | 2500   | 1072.055   | 835.686  | 1.283 |
| 10000        | 5000   | 991.160    | 793.664  | 1.249 |
| 100000       | 100    | 2637.534   | 981.948  | 2.686 |
| 100000       | 1000   | 2292.932   | 977.132  | 2.347 |
| 100000       | 5000   | 1805.598   | 953.070  | 1.895 |
| 100000       | 25000  | 1299.534   | 865.152  | 1.502 |
| 100000       | 50000  | 1170.767   | 811.671  | 1.442 |
| 1000000      | 1000   | 2697.692   | 974.252  | 2.769 |
| 1000000      | 10000  | 2564.625   | 999.482  | 2.566 |
| 1000000      | 50000  | 2055.573   | 956.625  | 2.149 |
| 1000000      | 250000 | 1448.800   | 867.032  | 1.671 |
| 1000000      | 500000 | 1262.295   | 808.145  | 1.562 |

The comparison was run on a Ryzen 5800H. Throughput is calculated as millions of elements per second, i.e `data.len() * runs / seconds`. The data used in the comparison was random `u32`s.

You can run the benchmarks with `cargo test -r turboselect_perf -- --nocapture --ignored`.

## Notes

The implementation is partially based on K. Kiwiel's paper "On Floyd and Rivest's SELECT algorithm" [1]. The speed improvements are mostly due to pivot selection. Typical Quickselect implementation is optimized for sorting, and tries to find a pivot that is near the median of the slice, for example by taking the median of medians. This tends to about halve the size of the unordered part of the slice. Turboselect's pivot selection is biased towards the element at the given index, and tries to reducing the size of the unordered part of the slice as much as possible without overshooting. 

This is illustrated in the following diagram, where the levels of blocks represent the unordered part of the slice in each step of the iteration. The desired index is marked with a `":"` and the position of the pivot after partitioning with a `"|"`. Since Turboselect's pivot "hugs" the index, the unordered part of the slice shrinks more quickly, reducing both the number of iterations and the number of elements examined in each iteration.

```text 
     Quickselect:                     |  Turboselect:
                                      |
            index                     |         index
              :                       |           : 
     0123456789abcdefghijklmnopqrstu  |  0123456789abcdefghijklmnopqrstu  
    ┌───────────────────────────────┐ | ┌───────────────────────────────┐
  0 │         :     |               │ | │         : |                   │
    ├───────────────┬───────────────┘ | ├───────────┬───────────────────┘
  1 │     |   :     │                 | │       | : │
    └─────┬─────────┤                 | └───────┬───┤
  2       │   :|    │                 |         │|: │
          ├────┬────┘                 |         └┬──┤
  3       │ | :│                      |          │:|│
          └─┬──┤                      |          ├─┬┘
  4         │|:│                      |          │:│ done
            └┬─┤                      |          └─┘
  5          │:│ done                 |
             └─┘                      |
```

When the size of the slice to be partitioned is large, the pivot selection calls Turboselect recursively for the sample. For small slices, an extension of median-of-medians is used. See the method `kth_of_nths` for details.

The sampling allows the pivot selection to also detect when the pivot is likely to have many duplicates. In these cases, the slice is split into three parts:

```text
 Equal-partitioning:
┌─────────────┬──────────────┬─────────────┐
│ x < data[u] │ x == data[u] │ x > data[u] │
└─────────────┴──────────────┴─────────────┘
               u            v
```

Kiwiel suggests a dual-pivot partitioning scheme, but it is not implemented in Turboselect, since it turned out to be slower than successive calls to single pivot partitions. An implementation is available in the commit history.

[1] Kiwiel, Krzysztof C. (30 November 2005). "On Floyd and Rivest's SELECT algorithm". Theoretical Computer Science. 347 (1–2): 214–238. [doi:10.1016/j.tcs.2005.06.032](https://doi.org/10.1016%2Fj.tcs.2005.06.032).