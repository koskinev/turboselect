# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm. 

This repository demonstrates that the Floyd & Rivest algorithm can be more efficient than a well implemented quickselect implementation. The speed improvements are most noticeable for indices far from the median.

The code is currently not well tested and should not be used in production.

**Comparison with  `slice::select_nth_unstable` as the baseline using random `u32` data**

| slice length | index      | throughput, runs/s | baseline, runs/s | ratio |
| ------------ | ---------- | ------------------ | ---------------- | ----- |
| 10 000       | 100        | 145925.42          | 87067.06         | 1.676 |
| 10 000       | 2 500      | 94152.766          | 79057.164        | 1.191 |
| 10 000       | 5 000      | 86107.51           | 74921.46         | 1.149 |
| 1 000 000    | 10 000     | 2282.4504          | 950.0334         | 2.402 |
| 1 000 000    | 250 000    | 914.019            | 840.3464         | 1.088 |
| 1 000 000    | 500 000    | 843.78204          | 784.85913        | 1.075 |
| 100 000 000  | 1 000 000  | 19.821             | 9.417646         | 2.105 |
| 100 000 000  | 25 000 000 | 12.269             | 8.637            | 1.421 |
| 100 000 000  | 50 000 000 | 11.014             | 8.098            | 1.360 |

The benchmarks can be run with `cargo test -r perf_tests -- --nocapture --ignored`.

## Notes

The implementation is based on  [1]. The speed improvements are mostly due to pivot selection from a small randomized sample, combined with a dual pivot partitioning algorithm for large slices. The sample is used to recursively find two pivots that are relatively close to each other, with a high probability of the selected element being between them. Then, the pivots are used to partition the slice into three parts: elements less than the lower pivot, elements between the pivots, and elements greater than the higher pivot. 
```text
 ┌──────┬──────────────────┬────────┐
 │< low │low <= .. <= high │ > high │ 
 └──────┴──────────────────┴────────┘
```

Since the target element is usually between the pivots, and the pivots are close, 
the partitioning reduces the size of the unordered part of the slice more efficiently than a single pivot partitioning. For relatively small slices, quickselect with custom biased pivot selection is faster. 

The partitioning algorithms suggested in [1] turned out to be hard to implement in a cache-friendly way. A dual pivot partitioning was modified from `core::slice::sort::partition_at_index` method.

Instead of comparing elements from the left and right to a single pivot, the elements from the left are compared to the lower pivot and the elements from the right are compared to the upper pivot. The out-of-order elements are swapped as in the original implementation, but then tested for 
being between the pivots. The elements that are between the pivots are then moved to temporary partitions at the end or beginning of the slice, resulting in:
```text
 ┌───────────────────┬───────┬───┬────────┬───────────────────┐
 │ low <= .. <= high │ < low │ ? │ > high │ low <= .. <= high │
 └───────────────────┴───────┴───┴────────┴───────────────────┘
```
Finally, the elements from the beginning and the end are moved to the middle, resulting in the final partitioning.


[1] Kiwiel, Krzysztof C. (30 November 2005). "On Floyd and Rivest's SELECT algorithm" (PDF). Theoretical Computer Science. 347 (1–2): 214–238. [doi:10.1016/j.tcs.2005.06.032](https://doi.org/10.1016%2Fj.tcs.2005.06.032).