# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random (bool)      | 1000         | 1           | 351161.00            | 450329.44          | 0.780 |
| random (bool)      | 1000         | 10          | 344536.69            | 443642.97          | 0.777 |
| random (bool)      | 1000         | 50          | 348487.91            | 448439.81          | 0.777 |
| random (bool)      | 1000         | 250         | 323589.28            | 436851.62          | 0.741 |
| random (bool)      | 1000         | 500         | 308129.50            | 432177.47          | 0.713 |
| random (bool)      | 10000        | 10          | 37597.48             | 46656.38           | 0.806 |
| random (bool)      | 10000        | 100         | 37714.86             | 46671.89           | 0.808 |
| random (bool)      | 10000        | 500         | 37629.91             | 46690.95           | 0.806 |
| random (bool)      | 10000        | 2500        | 37639.75             | 46822.26           | 0.804 |
| random (bool)      | 10000        | 5000        | 35471.70             | 47054.84           | 0.754 |
| random (u32)       | 1000         | 1           | 1035851.62           | 552794.56          | 1.874 |
| random (u32)       | 1000         | 10          | 973696.75            | 536602.69          | 1.815 |
| random (u32)       | 1000         | 50          | 837361.25            | 534815.44          | 1.566 |
| random (u32)       | 1000         | 250         | 630176.12            | 486302.94          | 1.296 |
| random (u32)       | 1000         | 500         | 587641.44            | 447263.53          | 1.314 |
| random (u32)       | 10000        | 10          | 187178.14            | 82465.52           | 2.270 |
| random (u32)       | 10000        | 100         | 175373.92            | 76224.13           | 2.301 |
| random (u32)       | 10000        | 500         | 146714.44            | 78403.15           | 1.871 |
| random (u32)       | 10000        | 2500        | 107000.57            | 77147.97           | 1.387 |
| random (u32)       | 10000        | 5000        | 97324.88             | 66421.08           | 1.465 |
| sawtooth (u32)     | 1000         | 1           | 1482782.62           | 1057207.25         | 1.403 |
| sawtooth (u32)     | 1000         | 10          | 1434139.62           | 1057002.50         | 1.357 |
| sawtooth (u32)     | 1000         | 50          | 1329133.38           | 1000647.56         | 1.328 |
| sawtooth (u32)     | 1000         | 250         | 746318.50            | 948673.00          | 0.787 |
| sawtooth (u32)     | 1000         | 500         | 747818.25            | 926899.94          | 0.807 |
| sawtooth (u32)     | 10000        | 10          | 192366.92            | 111597.29          | 1.724 |
| sawtooth (u32)     | 10000        | 100         | 193155.89            | 112237.98          | 1.721 |
| sawtooth (u32)     | 10000        | 500         | 144821.31            | 91941.93           | 1.575 |
| sawtooth (u32)     | 10000        | 2500        | 104289.42            | 78950.02           | 1.321 |
| sawtooth (u32)     | 10000        | 5000        | 94534.00             | 63013.41           | 1.500 |
| reversed (u32)     | 1000         | 1           | 1320953.00           | 1383295.00         | 0.955 |
| reversed (u32)     | 1000         | 10          | 1323207.50           | 1388233.00         | 0.953 |
| reversed (u32)     | 1000         | 50          | 1229490.38           | 1391286.12         | 0.884 |
| reversed (u32)     | 1000         | 250         | 747970.19            | 1336113.00         | 0.560 |
| reversed (u32)     | 1000         | 500         | 576025.56            | 1362019.12         | 0.423 |
| reversed (u32)     | 10000        | 10          | 219161.38            | 170405.80          | 1.286 |
| reversed (u32)     | 10000        | 100         | 211217.89            | 172063.42          | 1.228 |
| reversed (u32)     | 10000        | 500         | 166209.55            | 169508.41          | 0.981 |
| reversed (u32)     | 10000        | 2500        | 120780.64            | 172792.03          | 0.699 |
| reversed (u32)     | 10000        | 5000        | 97250.72             | 170075.52          | 0.572 |
| random dups (u32s) | 1000         | 1           | 994758.38            | 682628.25          | 1.457 |
| random dups (u32s) | 1000         | 10          | 981315.19            | 686130.06          | 1.430 |
| random dups (u32s) | 1000         | 50          | 801986.56            | 587709.25          | 1.365 |
| random dups (u32s) | 1000         | 250         | 607541.69            | 500355.75          | 1.214 |
| random dups (u32s) | 1000         | 500         | 542796.62            | 518564.81          | 1.047 |
| random dups (u32s) | 10000        | 10          | 182367.84            | 98909.95           | 1.844 |
| random dups (u32s) | 10000        | 100         | 171368.98            | 96450.92           | 1.777 |
| random dups (u32s) | 10000        | 500         | 136073.06            | 87684.55           | 1.552 |
| random dups (u32s) | 10000        | 2500        | 105420.16            | 73242.81           | 1.439 |
| random dups (u32s) | 10000        | 5000        | 94902.27             | 66247.57           | 1.433 |

The benchmarks were run on a Ryzen 5800H. You can run the benchmarks with `cargo test -r perf_tests -- --nocapture --ignored`.

## Notes

The implementation is based on [1]. The speed improvements are mostly due to pivot selection from a small randomized sample, combined with a dual pivot partitioning algorithm for large slices. The sample is used to recursively find two pivots that are relatively close to each other, with a high probability of the selected element being between them. Then, the pivots are used to partition the slice into three parts: elements less than the lower pivot, elements between the pivots, and elements greater than the higher pivot. 
```text
 ┌──────┬──────────────────┬────────┐
 │< low │low <= .. <= high │ > high │ 
 └──────┴──────────────────┴────────┘
```

This reduces the size of the unordered part of the slice more efficiently than a single pivot partitioning. For relatively small slices, quickselect with custom biased pivot selection is faster. 

The partitioning algorithms suggested in [1] turned out to be hard to implement in a cache-friendly way. A dual pivot partitioning was modified from `core::slice::sort::partition_at_index`.

Instead of comparing elements from the left and right to a single pivot, the elements from the left are compared to the lower pivot and the elements from the right are compared to the upper pivot. The out-of-order elements are swapped as in the original implementation, but then tested for being between the pivots. The elements that are between the pivots are then moved to temporary partitions at the end or beginning of the slice, resulting in:
```text
 ┌───────────────────┬───────┬───┬────────┬───────────────────┐
 │ low <= .. <= high │ < low │ ? │ > high │ low <= .. <= high │
 └───────────────────┴───────┴───┴────────┴───────────────────┘
```
Finally, the elements from the beginning and the end are moved to the middle, resulting in the final partitioning.


[1] Kiwiel, Krzysztof C. (30 November 2005). "On Floyd and Rivest's SELECT algorithm". Theoretical Computer Science. 347 (1–2): 214–238. [doi:10.1016/j.tcs.2005.06.032](https://doi.org/10.1016%2Fj.tcs.2005.06.032).