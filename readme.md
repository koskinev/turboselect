# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random (u32)       | 1000         | 1           | 1026424.31           | 565371.12          | 1.815 |
| random (u32)       | 1000         | 10          | 1034543.00           | 664620.12          | 1.557 |
| random (u32)       | 1000         | 50          | 905390.12            | 648775.62          | 1.396 |
| random (u32)       | 1000         | 250         | 665083.81            | 589728.19          | 1.128 |
| random (u32)       | 1000         | 500         | 618740.50            | 565609.31          | 1.094 |
| random (u32)       | 10000        | 10          | 186595.03            | 93007.84           | 2.006 |
| random (u32)       | 10000        | 100         | 176565.38            | 92461.07           | 1.910 |
| random (u32)       | 10000        | 500         | 146229.06            | 89918.27           | 1.626 |
| random (u32)       | 10000        | 2500        | 105696.04            | 82206.09           | 1.286 |
| random (u32)       | 10000        | 5000        | 94337.48             | 79641.80           | 1.185 |
| random (u32)       | 1000000      | 1000        | 2485.69              | 997.58             | 2.492 |
| random (u32)       | 1000000      | 10000       | 2430.01              | 996.42             | 2.439 |
| random (u32)       | 1000000      | 50000       | 2039.54              | 972.44             | 2.097 |
| random (u32)       | 1000000      | 250000      | 1380.12              | 890.14             | 1.550 |
| random (u32)       | 1000000      | 500000      | 1230.11              | 842.87             | 1.459 |
| random (u32)       | 100000000    | 100000      | 22.99                | 9.07               | 2.535 |
| random (u32)       | 100000000    | 1000000     | 23.24                | 9.68               | 2.401 |
| random (u32)       | 100000000    | 5000000     | 19.97                | 9.76               | 2.046 |
| random (u32)       | 100000000    | 25000000    | 13.59                | 8.74               | 1.555 |
| random (u32)       | 100000000    | 50000000    | 12.04                | 8.25               | 1.459 |
| sawtooth (u32)     | 1000         | 1           | 1167643.88           | 1002128.94         | 1.165 |
| sawtooth (u32)     | 1000         | 10          | 1281429.25           | 1054734.25         | 1.215 |
| sawtooth (u32)     | 1000         | 50          | 1194506.12           | 1027580.00         | 1.162 |
| sawtooth (u32)     | 1000         | 250         | 644100.88            | 959077.81          | 0.672 |
| sawtooth (u32)     | 1000         | 500         | 559463.88            | 966970.06          | 0.579 |
| sawtooth (u32)     | 10000        | 10          | 184051.16            | 115611.13          | 1.592 |
| sawtooth (u32)     | 10000        | 100         | 179883.27            | 113363.98          | 1.587 |
| sawtooth (u32)     | 10000        | 500         | 144996.25            | 100962.66          | 1.436 |
| sawtooth (u32)     | 10000        | 2500        | 107674.61            | 86324.10           | 1.247 |
| sawtooth (u32)     | 10000        | 5000        | 96475.20             | 80127.78           | 1.204 |
| sawtooth (u32)     | 1000000      | 1000        | 2838.03              | 1080.63            | 2.626 |
| sawtooth (u32)     | 1000000      | 10000       | 2763.28              | 1065.40            | 2.594 |
| sawtooth (u32)     | 1000000      | 50000       | 2275.98              | 996.27             | 2.285 |
| sawtooth (u32)     | 1000000      | 250000      | 1460.03              | 844.48             | 1.729 |
| sawtooth (u32)     | 1000000      | 500000      | 1280.25              | 810.99             | 1.579 |
| sawtooth (u32)     | 100000000    | 100000      | 28.61                | 10.93              | 2.616 |
| sawtooth (u32)     | 100000000    | 1000000     | 28.15                | 10.63              | 2.649 |
| sawtooth (u32)     | 100000000    | 5000000     | 23.76                | 9.81               | 2.423 |
| sawtooth (u32)     | 100000000    | 25000000    | 15.48                | 8.98               | 1.723 |
| sawtooth (u32)     | 100000000    | 50000000    | 13.59                | 8.30               | 1.637 |
| reversed (u32)     | 1000         | 1           | 1219170.38           | 1090124.38         | 1.118 |
| reversed (u32)     | 1000         | 10          | 943962.62            | 1062603.25         | 0.888 |
| reversed (u32)     | 1000         | 50          | 909745.69            | 1037053.56         | 0.877 |
| reversed (u32)     | 1000         | 250         | 675640.81            | 1012245.56         | 0.667 |
| reversed (u32)     | 1000         | 500         | 599277.25            | 928218.62          | 0.646 |
| reversed (u32)     | 10000        | 10          | 211113.78            | 121343.44          | 1.740 |
| reversed (u32)     | 10000        | 100         | 204628.69            | 118476.53          | 1.727 |
| reversed (u32)     | 10000        | 500         | 161253.36            | 119579.80          | 1.349 |
| reversed (u32)     | 10000        | 2500        | 114484.89            | 118529.15          | 0.966 |
| reversed (u32)     | 10000        | 5000        | 94702.70             | 110577.34          | 0.856 |
| reversed (u32)     | 1000000      | 1000        | 2967.11              | 1182.94            | 2.508 |
| reversed (u32)     | 1000000      | 10000       | 2865.72              | 1164.27            | 2.461 |
| reversed (u32)     | 1000000      | 50000       | 2402.63              | 1097.02            | 2.190 |
| reversed (u32)     | 1000000      | 250000      | 1597.97              | 936.08             | 1.707 |
| reversed (u32)     | 1000000      | 500000      | 1404.08              | 889.17             | 1.579 |
| reversed (u32)     | 100000000    | 100000      | 20.67                | 10.49              | 1.970 |
| reversed (u32)     | 100000000    | 1000000     | 20.31                | 10.81              | 1.879 |
| reversed (u32)     | 100000000    | 5000000     | 17.11                | 9.36               | 1.827 |
| reversed (u32)     | 100000000    | 25000000    | 13.06                | 8.43               | 1.550 |
| reversed (u32)     | 100000000    | 50000000    | 11.39                | 7.87               | 1.447 |
| random dups (u32s) | 1000         | 1           | 1130046.38           | 788923.31          | 1.432 |
| random dups (u32s) | 1000         | 10          | 1001546.00           | 785422.69          | 1.275 |
| random dups (u32s) | 1000         | 50          | 853725.69            | 726696.06          | 1.175 |
| random dups (u32s) | 1000         | 250         | 674697.94            | 633559.44          | 1.065 |
| random dups (u32s) | 1000         | 500         | 632809.62            | 612641.00          | 1.033 |
| random dups (u32s) | 10000        | 10          | 179584.09            | 100487.23          | 1.787 |
| random dups (u32s) | 10000        | 100         | 172021.17            | 99347.76           | 1.732 |
| random dups (u32s) | 10000        | 500         | 143242.80            | 95220.48           | 1.504 |
| random dups (u32s) | 10000        | 2500        | 106865.33            | 86042.68           | 1.242 |
| random dups (u32s) | 10000        | 5000        | 94500.23             | 80215.73           | 1.178 |
| random dups (u32s) | 1000000      | 1000        | 2511.68              | 1014.90            | 2.475 |
| random dups (u32s) | 1000000      | 10000       | 2442.71              | 1008.91            | 2.421 |
| random dups (u32s) | 1000000      | 50000       | 2060.51              | 990.73             | 2.080 |
| random dups (u32s) | 1000000      | 250000      | 1392.67              | 896.93             | 1.553 |
| random dups (u32s) | 1000000      | 500000      | 1238.61              | 843.53             | 1.468 |
| random dups (u32s) | 100000000    | 100000      | 24.39                | 9.98               | 2.443 |
| random dups (u32s) | 100000000    | 1000000     | 23.72                | 9.99               | 2.374 |
| random dups (u32s) | 100000000    | 5000000     | 19.62                | 9.69               | 2.025 |
| random dups (u32s) | 100000000    | 25000000    | 13.82                | 8.68               | 1.592 |
| random dups (u32s) | 100000000    | 50000000    | 12.20                | 8.23               | 1.482 |
| random (bool)      | 1000         | 1           | 482852.25            | 465742.97          | 1.037 |
| random (bool)      | 1000         | 10          | 465158.94            | 457503.66          | 1.017 |
| random (bool)      | 1000         | 50          | 463785.97            | 457734.28          | 1.013 |
| random (bool)      | 1000         | 250         | 380660.94            | 469693.31          | 0.810 |
| random (bool)      | 1000         | 500         | 422815.91            | 467393.22          | 0.905 |
| random (bool)      | 10000        | 10          | 52334.75             | 48608.34           | 1.077 |
| random (bool)      | 10000        | 100         | 52444.34             | 48699.67           | 1.077 |
| random (bool)      | 10000        | 500         | 52410.92             | 48707.71           | 1.076 |
| random (bool)      | 10000        | 2500        | 52846.16             | 48701.08           | 1.085 |
| random (bool)      | 10000        | 5000        | 48252.70             | 48648.50           | 0.992 |
| random (bool)      | 1000000      | 1000        | 539.29               | 495.63             | 1.088 |
| random (bool)      | 1000000      | 10000       | 541.11               | 496.92             | 1.089 |
| random (bool)      | 1000000      | 50000       | 540.65               | 495.59             | 1.091 |
| random (bool)      | 1000000      | 250000      | 539.71               | 495.25             | 1.090 |
| random (bool)      | 1000000      | 500000      | 480.39               | 489.80             | 0.981 |
| random (bool)      | 100000000    | 100000      | 5.40                 | 4.96               | 1.089 |
| random (bool)      | 100000000    | 1000000     | 5.41                 | 5.08               | 1.065 |
| random (bool)      | 100000000    | 5000000     | 5.41                 | 4.79               | 1.130 |
| random (bool)      | 100000000    | 25000000    | 5.41                 | 5.17               | 1.046 |
| random (bool)      | 100000000    | 50000000    | 4.76                 | 4.68               | 1.017 |

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