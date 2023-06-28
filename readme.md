# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random (u32)       | 1000         | 1           | 1054760.41           | 569952.94          | 1.851 |
| random (u32)       | 1000         | 10          | 1073098.09           | 644500.42          | 1.665 |
| random (u32)       | 1000         | 50          | 921560.76            | 626331.34          | 1.471 |
| random (u32)       | 1000         | 250         | 684964.54            | 592354.41          | 1.156 |
| random (u32)       | 1000         | 500         | 651462.50            | 582876.83          | 1.118 |
| random (u32)       | 10000        | 10          | 190545.73            | 92251.80           | 2.065 |
| random (u32)       | 10000        | 100         | 181737.27            | 92935.86           | 1.956 |
| random (u32)       | 10000        | 500         | 151483.47            | 91004.14           | 1.665 |
| random (u32)       | 10000        | 2500        | 108950.11            | 83674.29           | 1.302 |
| random (u32)       | 10000        | 5000        | 98782.80             | 80212.00           | 1.232 |
| random (u32)       | 1000000      | 1000        | 2570.34              | 1021.90            | 2.515 |
| random (u32)       | 1000000      | 10000       | 2503.30              | 1025.19            | 2.442 |
| random (u32)       | 1000000      | 50000       | 2117.62              | 1006.75            | 2.103 |
| random (u32)       | 1000000      | 250000      | 1428.87              | 907.81             | 1.574 |
| random (u32)       | 1000000      | 500000      | 1261.23              | 852.71             | 1.479 |
| random (u32)       | 100000000    | 100000      | 24.89                | 10.11              | 2.462 |
| random (u32)       | 100000000    | 1000000     | 24.70                | 9.86               | 2.504 |
| random (u32)       | 100000000    | 5000000     | 20.55                | 9.78               | 2.101 |
| random (u32)       | 100000000    | 25000000    | 13.95                | 8.87               | 1.573 |
| random (u32)       | 100000000    | 50000000    | 12.53                | 8.30               | 1.509 |
| sawtooth (u32)     | 1000         | 1           | 1275280.86           | 1085429.84         | 1.175 |
| sawtooth (u32)     | 1000         | 10          | 1323128.68           | 1088731.37         | 1.215 |
| sawtooth (u32)     | 1000         | 50          | 1252954.45           | 943426.70          | 1.328 |
| sawtooth (u32)     | 1000         | 250         | 670616.95            | 963424.48          | 0.696 |
| sawtooth (u32)     | 1000         | 500         | 581278.41            | 943328.79          | 0.616 |
| sawtooth (u32)     | 10000        | 10          | 191006.17            | 118384.02          | 1.613 |
| sawtooth (u32)     | 10000        | 100         | 190083.60            | 118704.56          | 1.601 |
| sawtooth (u32)     | 10000        | 500         | 150936.45            | 104502.92          | 1.444 |
| sawtooth (u32)     | 10000        | 2500        | 112746.57            | 89225.88           | 1.264 |
| sawtooth (u32)     | 10000        | 5000        | 99360.64             | 81885.37           | 1.213 |
| sawtooth (u32)     | 1000000      | 1000        | 2965.51              | 1114.67            | 2.660 |
| sawtooth (u32)     | 1000000      | 10000       | 2862.20              | 1084.15            | 2.640 |
| sawtooth (u32)     | 1000000      | 50000       | 2365.26              | 1009.18            | 2.344 |
| sawtooth (u32)     | 1000000      | 250000      | 1516.02              | 865.59             | 1.751 |
| sawtooth (u32)     | 1000000      | 500000      | 1313.27              | 822.65             | 1.596 |
| sawtooth (u32)     | 100000000    | 100000      | 29.49                | 10.87              | 2.714 |
| sawtooth (u32)     | 100000000    | 1000000     | 28.91                | 10.82              | 2.671 |
| sawtooth (u32)     | 100000000    | 5000000     | 24.09                | 10.32              | 2.334 |
| sawtooth (u32)     | 100000000    | 25000000    | 16.27                | 9.06               | 1.796 |
| sawtooth (u32)     | 100000000    | 50000000    | 14.03                | 8.44               | 1.663 |
| reversed (u32)     | 1000         | 1           | 1245162.39           | 1055963.58         | 1.179 |
| reversed (u32)     | 1000         | 10          | 1072614.32           | 1075986.79         | 0.997 |
| reversed (u32)     | 1000         | 50          | 1032681.61           | 1017972.78         | 1.014 |
| reversed (u32)     | 1000         | 250         | 681472.16            | 1009907.77         | 0.675 |
| reversed (u32)     | 1000         | 500         | 654138.97            | 949958.99          | 0.689 |
| reversed (u32)     | 10000        | 10          | 223026.64            | 126075.75          | 1.769 |
| reversed (u32)     | 10000        | 100         | 215494.98            | 125368.85          | 1.719 |
| reversed (u32)     | 10000        | 500         | 168898.62            | 123221.65          | 1.371 |
| reversed (u32)     | 10000        | 2500        | 121728.11            | 124735.58          | 0.976 |
| reversed (u32)     | 10000        | 5000        | 98605.07             | 115266.71          | 0.855 |
| reversed (u32)     | 1000000      | 1000        | 3071.22              | 1187.41            | 2.586 |
| reversed (u32)     | 1000000      | 10000       | 2972.08              | 1168.82            | 2.543 |
| reversed (u32)     | 1000000      | 50000       | 2473.83              | 1103.38            | 2.242 |
| reversed (u32)     | 1000000      | 250000      | 1654.60              | 951.51             | 1.739 |
| reversed (u32)     | 1000000      | 500000      | 1443.97              | 894.20             | 1.615 |
| reversed (u32)     | 100000000    | 100000      | 21.51                | 11.29              | 1.906 |
| reversed (u32)     | 100000000    | 1000000     | 21.81                | 11.03              | 1.978 |
| reversed (u32)     | 100000000    | 5000000     | 17.98                | 9.76               | 1.843 |
| reversed (u32)     | 100000000    | 25000000    | 13.43                | 8.26               | 1.625 |
| reversed (u32)     | 100000000    | 50000000    | 11.89                | 7.83               | 1.519 |
| random dups (u32s) | 1000         | 1           | 1147739.80           | 795927.05          | 1.442 |
| random dups (u32s) | 1000         | 10          | 1018647.04           | 792115.27          | 1.286 |
| random dups (u32s) | 1000         | 50          | 887429.84            | 751408.75          | 1.181 |
| random dups (u32s) | 1000         | 250         | 691360.97            | 643526.79          | 1.074 |
| random dups (u32s) | 1000         | 500         | 631088.24            | 603229.76          | 1.046 |
| random dups (u32s) | 10000        | 10          | 183094.09            | 100868.58          | 1.815 |
| random dups (u32s) | 10000        | 100         | 176918.51            | 100606.39          | 1.759 |
| random dups (u32s) | 10000        | 500         | 147387.35            | 97035.30           | 1.519 |
| random dups (u32s) | 10000        | 2500        | 110700.18            | 87478.58           | 1.265 |
| random dups (u32s) | 10000        | 5000        | 98031.49             | 81464.61           | 1.203 |
| random dups (u32s) | 1000000      | 1000        | 2571.98              | 1026.95            | 2.504 |
| random dups (u32s) | 1000000      | 10000       | 2494.26              | 1018.74            | 2.448 |
| random dups (u32s) | 1000000      | 50000       | 2115.28              | 999.34             | 2.117 |
| random dups (u32s) | 1000000      | 250000      | 1432.97              | 910.24             | 1.574 |
| random dups (u32s) | 1000000      | 500000      | 1255.26              | 850.33             | 1.476 |
| random dups (u32s) | 100000000    | 100000      | 25.16                | 9.89               | 2.543 |
| random dups (u32s) | 100000000    | 1000000     | 24.56                | 10.09              | 2.434 |
| random dups (u32s) | 100000000    | 5000000     | 20.63                | 9.90               | 2.083 |
| random dups (u32s) | 100000000    | 25000000    | 14.27                | 8.78               | 1.625 |
| random dups (u32s) | 100000000    | 50000000    | 12.72                | 8.45               | 1.505 |
| random (bool)      | 1000         | 1           | 483191.38            | 469547.19          | 1.029 |
| random (bool)      | 1000         | 10          | 474447.80            | 472437.96          | 1.004 |
| random (bool)      | 1000         | 50          | 480899.28            | 477015.35          | 1.008 |
| random (bool)      | 1000         | 250         | 382475.31            | 471198.19          | 0.812 |
| random (bool)      | 1000         | 500         | 434675.44            | 478354.12          | 0.909 |
| random (bool)      | 10000        | 10          | 52704.45             | 49021.79           | 1.075 |
| random (bool)      | 10000        | 100         | 52577.50             | 48905.48           | 1.075 |
| random (bool)      | 10000        | 500         | 52560.69             | 48862.74           | 1.076 |
| random (bool)      | 10000        | 2500        | 53120.58             | 49052.25           | 1.083 |
| random (bool)      | 10000        | 5000        | 48736.60             | 48924.32           | 0.996 |
| random (bool)      | 1000000      | 1000        | 545.53               | 500.26             | 1.091 |
| random (bool)      | 1000000      | 10000       | 543.23               | 503.38             | 1.079 |
| random (bool)      | 1000000      | 50000       | 544.19               | 494.70             | 1.100 |
| random (bool)      | 1000000      | 250000      | 540.26               | 501.41             | 1.077 |
| random (bool)      | 1000000      | 500000      | 483.77               | 496.38             | 0.975 |
| random (bool)      | 100000000    | 100000      | 5.43                 | 4.84               | 1.123 |
| random (bool)      | 100000000    | 1000000     | 5.44                 | 5.20               | 1.046 |
| random (bool)      | 100000000    | 5000000     | 5.44                 | 4.88               | 1.116 |
| random (bool)      | 100000000    | 25000000    | 5.51                 | 4.97               | 1.108 |
| random (bool)      | 100000000    | 50000000    | 4.79                 | 5.17               | 0.926 |

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