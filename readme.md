# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm. 

This implementation demonstrates that the Floyd & Rivest algorithm can be 10%-100% faster than a well implemented quickselect implementation. The speed improvements are most noticeable for indices far from the median.

The code is currently not well tested and should not be used in production.

### Comparison with  `slice::select_nth_unstable` as the baseline using random `u32` data.

| slice length | index      | throughput, runs/s | baseline, runs/s | improvement, % |
| ------------ | ---------- | ------------------ | ---------------- | -------------- |
| 10 000       | 100        | 157066             | 92257            | 70 %           |
| 10 000       | 2 500      | 98948              | 82887            | 19 %           |
| 10 000       | 5 000      | 89875              | 79453            | 13 %           |
| 1 000 000    | 10 000     | 2322               | 1001             | 131 %          |
| 1 000 000    | 250 000    | 918                | 887              | 3 %            |
| 1 000 000    | 500 000    | 847                | 839              | 1 %            |
| 100 000 000  | 1 000 000  | ~20                | ~10              | 107 %          |
| 100 000 000  | 25 000 000 | ~12                | ~9               | 34 %           |
| 100 000 000  | 50 000 000 | ~10                | ~8               | 32 %           |

The benchmarks can be run with `cargo test -r perf_tests -- --nocapture --ignored`.

## Notes

The implementation is based on  [1]. The speed improvements are mostly due to pivot selection from a small randomized sample, combined with a dual pivot partitioning algorithm. For relatively small slices this implementation uses quickselect with custom biased pivot selection. 

The partitioning algorithms suggested in [1] turned out to be hard to implement in a cache-friendly way. A dual pivot partitioning was modified from `core::slice::sort::partition_at_index` method.

Instead of comparing elements from the left and right to a single pivot, the elements from the left are compared to the lower pivot and the elements from the right are compared to the upper pivot. The out-of-order elements are swapped as in the original implementation, but then tested for 
being between the pivots. The elements that are between the pivots are then moved to temporary partitions at the end or beginning of the slice, resulting in:
```text
 ┌───────────────────┬───────┬───┬────────┬───────────────────┐
 │ low <= .. <= high │ < low │ ? │ > high │ low <= .. <= high │
 └───────────────────┴───────┴───┴────────┴───────────────────┘
```
Finally, the elements from the beginning and the end are moved to the middle, resulting in the final partitioning:
```text
 ┌──────┬──────────────────┬────────┬───────────────────┐
 │< low │low <= .. <= high │ > high │ low <= .. <= high │
 └──────┴──────────────────┴────────┴───────────────────┘
```

[1] Kiwiel, Krzysztof C. (30 November 2005). "On Floyd and Rivest's SELECT algorithm" (PDF). Theoretical Computer Science. 347 (1–2): 214–238. [doi:10.1016/j.tcs.2005.06.032](https://doi.org/10.1016%2Fj.tcs.2005.06.032).