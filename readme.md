# Turboselect

Turboselect is an alternative implementation of the `slice::select_nth_unstable` method. Like `slice::select_nth_unstable`, `turboselect::select_nth_unstable` takes a mutable slice and an index, and reorders the elements so that the element at the given index is at its final position. After the call, the elements before the index are less than or equal to the element at the index, and the elements after the index are greater than or equal to the element at the index.

```rust
    let (_, nth, _) = turboselect::select_nth_unstable(data, index);
  // ┌─────────────────┬─────────────────┬────────────────┐
  // │ &data[i] <= nth │ &data[i] == nth │ data[i] >= nth │
  // └─────────────────┴─────────────────┴────────────────┘
  //     i < index          i == index       i > index           
```

Turboselect demonstrates better speed over the Quickselect implementation in the standard library, particulary for finding elements far from the median. The algorithm works by recursively selecting a pivot element to compare against and partitioning the slice into smaller parts. The implementation is partially based on K. Kiwiel's paper "On Floyd and Rivest's SELECT algorithm" [[2]](https://dx.doi.org/10.1145/360680.360694), [[3]](https://dx.doi.org/10.1016/j.tcs.2005.06.032).

In addition to `select_nth_unstable`, the following methods are provided:
- `select_nth_unstable_by_key`, which takes a key extraction function as an argument.
- `select_nth_unstable_by_cached_key`, which is similar to `select_nth_unstable_by_key`, but caches the keys in a temporary buffer. This is useful if the key extraction is expensive. 
  
The implementation relies heavily on unsafe code and is currently not thoroughly tested. To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**
 
| slice length | index      | throughput | baseline | ratio |
| ------------ | ---------- | ---------- | -------- | ----- |
| 1 000        | 1          | 1249.153   | 661.467  | 1.888 |
| 1 000        | 50         | 899.107    | 608.198  | 1.478 |
| 1 000        | 500        | 628.416    | 546.771  | 1.149 |
| 10 000       | 10         | 2175.853   | 919.347  | 2.367 |
| 10 000       | 500        | 1396.770   | 882.031  | 1.584 |
| 10 000       | 5 000      | 979.494    | 754.819  | 1.298 |
| 100 000      | 100        | 2617.018   | 960.814  | 2.724 |
| 100 000      | 5 000      | 1805.564   | 931.358  | 1.939 |
| 100 000      | 50 000     | 1205.500   | 793.645  | 1.519 |
| 1 000 000    | 1 000      | 2713.383   | 953.882  | 2.845 |
| 1 000 000    | 50 000     | 2060.831   | 932.977  | 2.209 |
| 1 000 000    | 500 000    | 1322.733   | 797.267  | 1.659 |
| 10 000 000   | 10 000     | 2397.961   | 924.765  | 2.593 |
| 10 000 000   | 500 000    | 1997.528   | 905.618  | 2.206 |
| 10 000 000   | 5 000 000  | 1289.671   | 769.359  | 1.676 |
| 100 000 000  | 100 000    | 2753.677   | 903.986  | 3.046 |
| 100 000 000  | 5 000 000  | 1664.166   | 833.201  | 1.997 |
| 100 000 000  | 50 000 000 | 1315.275   | 720.552  | 1.825 |
  
The comparison was run on a Ryzen 5800H with pseudorandom `u32`s used as input. Throughput is calculated as millions of elements per second, i.e `data.len() * runs / seconds`. Similar tests were run with sawtooth, sorted and reverse sorted inputs, as well as pseudorandom `u32`s with many duplicates, and pseudorandom booleans. Turboselect outperformed Quickselect in all cases where the input had at least 100 000 elements. Turboselect may be slower than Quickselect for smaller presorted, reversed and sawtooth inputs.

See [this table](bench_results.md) for full results.

You can run the benchmarks with `cargo test -r turboselect_perf -- --nocapture --ignored`.

## Notes

The speed improvements are mostly due to pivot selection. In Quickselect, median of medians is usually used, which tends to put the pivot near the middle of the slice. This about halves the size of the unordered part of the slice. Turboselect biases the selection towards the desired index to reduce the size of the unordered part of the slice as much as possible without overshooting. 

The difference is illustrated in the following diagram, where the levels of blocks represent the unordered part of the slice in each step of the iteration. The index is marked with a `":"`, the position of the pivot after partitioning with a `"|"`, and the ordered part of the slice with `░░`. Since Turboselect's pivot "hugs" the index, the unordered part of the slice shrinks more quickly, reducing both the number of iterations and the number of elements to be processed in subsequent iterations.

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

Pivot selection starts with putting a random sample of elements to the beginning of the slice. When the slice is large, this is followed with a recursive call to Turboselect. For small slices, an extension of median-of-medians inspired by A. Alexandrescu's paper "Fast Deterministic Selection" [[1]](https://dx.doi.org/10.4230/LIPIcs.SEA.2017.24), is used. See the function `kth_of_nths` for details.

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

"Turbo" refers to the complexity added in order to get the gains over the core library counterpart. Some other approach might be faster. 

## References

[1] Alexandrescu, Andrei. (2017). Fast Deterministic Selection. [10.4230/LIPIcs.SEA.2017.24](https://dx.doi.org/10.4230/LIPIcs.SEA.2017.24). 

[2] Floyd, Robert & Rivest, Ronald. (1975). Algorithm 489: The algorithm SELECT — for finding the $i$th smallest of $n$ elements. Communications of the ACM. 18. [10.1145/360680.360694](https://dx.doi.org/10.1145/360680.360694). 

[3] Kiwiel, Krzysztof. (2005). On Floyd and Rivest's SELECT algorithm. Theoretical Computer Science. 347. 214-238. [10.1016/j.tcs.2005.06.032](https://dx.doi.org/10.1016/j.tcs.2005.06.032).  