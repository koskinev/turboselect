# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, runs/sec | baseline, runs/sec | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random (u32)       | 1000         | 1           | 1041346.92           | 662192.28          | 1.573 |
| random (u32)       | 1000         | 10          | 1039622.75           | 657846.07          | 1.580 |
| random (u32)       | 1000         | 50          | 917399.30            | 650984.89          | 1.409 |
| random (u32)       | 1000         | 250         | 671890.42            | 586884.82          | 1.145 |
| random (u32)       | 1000         | 500         | 614049.78            | 552253.09          | 1.112 |
| random (u32)       | 10000        | 10          | 189019.38            | 94794.44           | 1.994 |
| random (u32)       | 10000        | 100         | 177207.76            | 93384.22           | 1.898 |
| random (u32)       | 10000        | 500         | 148430.26            | 92210.24           | 1.610 |
| random (u32)       | 10000        | 2500        | 109408.58            | 84359.64           | 1.297 |
| random (u32)       | 10000        | 5000        | 98207.77             | 79291.49           | 1.239 |
| random (u32)       | 1000000      | 1000        | 2521.63              | 1005.74            | 2.507 |
| random (u32)       | 1000000      | 10000       | 2445.98              | 1008.42            | 2.426 |
| random (u32)       | 1000000      | 50000       | 2075.75              | 987.22             | 2.103 |
| random (u32)       | 1000000      | 250000      | 1411.24              | 887.60             | 1.590 |
| random (u32)       | 1000000      | 500000      | 1256.80              | 841.23             | 1.494 |
| random (u32)       | 100000000    | 100000      | 24.70                | 9.82               | 2.515 |
| random (u32)       | 100000000    | 1000000     | 23.87                | 9.86               | 2.422 |
| random (u32)       | 100000000    | 5000000     | 19.96                | 9.50               | 2.100 |
| random (u32)       | 100000000    | 25000000    | 14.14                | 8.80               | 1.606 |
| random (u32)       | 100000000    | 50000000    | 12.74                | 8.29               | 1.536 |
| sawtooth (u32)     | 1000         | 1           | 1317293.34           | 1098525.44         | 1.199 |
| sawtooth (u32)     | 1000         | 10          | 1126853.26           | 1071993.06         | 1.051 |
| sawtooth (u32)     | 1000         | 50          | 971603.42            | 1004372.67         | 0.967 |
| sawtooth (u32)     | 1000         | 250         | 892922.99            | 1006543.16         | 0.887 |
| sawtooth (u32)     | 1000         | 500         | 749996.53            | 960851.87          | 0.781 |
| sawtooth (u32)     | 10000        | 10          | 191360.46            | 115744.60          | 1.653 |
| sawtooth (u32)     | 10000        | 100         | 192175.12            | 116734.48          | 1.646 |
| sawtooth (u32)     | 10000        | 500         | 150770.30            | 102830.06          | 1.466 |
| sawtooth (u32)     | 10000        | 2500        | 111760.10            | 87502.65           | 1.277 |
| sawtooth (u32)     | 10000        | 5000        | 98483.89             | 78832.66           | 1.249 |
| sawtooth (u32)     | 1000000      | 1000        | 2898.07              | 1105.33            | 2.622 |
| sawtooth (u32)     | 1000000      | 10000       | 2805.16              | 1079.42            | 2.599 |
| sawtooth (u32)     | 1000000      | 50000       | 2333.83              | 1011.19            | 2.308 |
| sawtooth (u32)     | 1000000      | 250000      | 1497.97              | 865.07             | 1.732 |
| sawtooth (u32)     | 1000000      | 500000      | 1312.36              | 821.09             | 1.598 |
| sawtooth (u32)     | 100000000    | 100000      | 29.37                | 10.85              | 2.708 |
| sawtooth (u32)     | 100000000    | 1000000     | 28.37                | 10.75              | 2.639 |
| sawtooth (u32)     | 100000000    | 5000000     | 23.94                | 10.01              | 2.391 |
| sawtooth (u32)     | 100000000    | 25000000    | 15.95                | 8.51               | 1.875 |
| sawtooth (u32)     | 100000000    | 50000000    | 14.05                | 8.55               | 1.643 |
| reversed (u32)     | 1000         | 1           | 1330406.10           | 1085135.83         | 1.226 |
| reversed (u32)     | 1000         | 10          | 1429616.74           | 1041442.57         | 1.373 |
| reversed (u32)     | 1000         | 50          | 1442845.54           | 1036133.81         | 1.393 |
| reversed (u32)     | 1000         | 250         | 946370.22            | 1031645.34         | 0.917 |
| reversed (u32)     | 1000         | 500         | 1012589.83           | 961250.69          | 1.053 |
| reversed (u32)     | 10000        | 10          | 229033.62            | 121328.00          | 1.888 |
| reversed (u32)     | 10000        | 100         | 213177.57            | 120398.81          | 1.771 |
| reversed (u32)     | 10000        | 500         | 178202.97            | 119200.69          | 1.495 |
| reversed (u32)     | 10000        | 2500        | 122669.46            | 120236.04          | 1.020 |
| reversed (u32)     | 10000        | 5000        | 95678.10             | 109379.34          | 0.875 |
| reversed (u32)     | 1000000      | 1000        | 3070.05              | 1191.04            | 2.578 |
| reversed (u32)     | 1000000      | 10000       | 2960.79              | 1166.45            | 2.538 |
| reversed (u32)     | 1000000      | 50000       | 2476.71              | 1100.13            | 2.251 |
| reversed (u32)     | 1000000      | 250000      | 1668.26              | 953.36             | 1.750 |
| reversed (u32)     | 1000000      | 500000      | 1460.15              | 892.01             | 1.637 |
| reversed (u32)     | 100000000    | 100000      | 24.05                | 11.21              | 2.145 |
| reversed (u32)     | 100000000    | 1000000     | 23.25                | 10.63              | 2.187 |
| reversed (u32)     | 100000000    | 5000000     | 18.45                | 9.85               | 1.874 |
| reversed (u32)     | 100000000    | 25000000    | 12.35                | 8.39               | 1.472 |
| reversed (u32)     | 100000000    | 50000000    | 7.40                 | 8.01               | 0.923 |
| random dups (u32s) | 1000         | 1           | 1199628.38           | 810190.58          | 1.481 |
| random dups (u32s) | 1000         | 10          | 1087315.11           | 806958.13          | 1.347 |
| random dups (u32s) | 1000         | 50          | 879903.42            | 751157.97          | 1.171 |
| random dups (u32s) | 1000         | 250         | 708605.73            | 657681.05          | 1.077 |
| random dups (u32s) | 1000         | 500         | 654881.49            | 624062.88          | 1.049 |
| random dups (u32s) | 10000        | 10          | 190439.51            | 103884.61          | 1.833 |
| random dups (u32s) | 10000        | 100         | 182647.78            | 101864.93          | 1.793 |
| random dups (u32s) | 10000        | 500         | 149179.12            | 98101.19           | 1.521 |
| random dups (u32s) | 10000        | 2500        | 112361.47            | 88753.57           | 1.266 |
| random dups (u32s) | 10000        | 5000        | 100740.54            | 83830.67           | 1.202 |
| random dups (u32s) | 1000000      | 1000        | 2570.28              | 1034.58            | 2.484 |
| random dups (u32s) | 1000000      | 10000       | 2489.79              | 1025.82            | 2.427 |
| random dups (u32s) | 1000000      | 50000       | 2107.01              | 999.27             | 2.109 |
| random dups (u32s) | 1000000      | 250000      | 1435.76              | 910.84             | 1.576 |
| random dups (u32s) | 1000000      | 500000      | 1272.28              | 860.95             | 1.478 |
| random dups (u32s) | 100000000    | 100000      | 24.64                | 10.18              | 2.421 |
| random dups (u32s) | 100000000    | 1000000     | 24.20                | 9.94               | 2.435 |
| random dups (u32s) | 100000000    | 5000000     | 20.55                | 9.83               | 2.091 |
| random dups (u32s) | 100000000    | 25000000    | 14.12                | 8.95               | 1.577 |
| random dups (u32s) | 100000000    | 50000000    | 12.55                | 8.48               | 1.479 |
| random (bool)      | 1000         | 1           | 468959.13            | 466334.00          | 1.006 |
| random (bool)      | 1000         | 10          | 484459.87            | 478881.70          | 1.012 |
| random (bool)      | 1000         | 50          | 486224.91            | 478156.37          | 1.017 |
| random (bool)      | 1000         | 250         | 462676.41            | 461538.83          | 1.002 |
| random (bool)      | 1000         | 500         | 442178.21            | 475777.14          | 0.929 |
| random (bool)      | 10000        | 10          | 53748.13             | 49641.28           | 1.083 |
| random (bool)      | 10000        | 100         | 53964.44             | 49942.15           | 1.081 |
| random (bool)      | 10000        | 500         | 53766.38             | 49705.47           | 1.082 |
| random (bool)      | 10000        | 2500        | 54124.28             | 49881.11           | 1.085 |
| random (bool)      | 10000        | 5000        | 51814.13             | 49860.99           | 1.039 |
| random (bool)      | 1000000      | 1000        | 551.46               | 503.64             | 1.095 |
| random (bool)      | 1000000      | 10000       | 553.84               | 513.86             | 1.078 |
| random (bool)      | 1000000      | 50000       | 553.98               | 502.41             | 1.103 |
| random (bool)      | 1000000      | 250000      | 579.52               | 509.46             | 1.138 |
| random (bool)      | 1000000      | 500000      | 562.76               | 509.06             | 1.105 |
| random (bool)      | 100000000    | 100000      | 5.54                 | 5.08               | 1.090 |
| random (bool)      | 100000000    | 1000000     | 5.54                 | 5.25               | 1.056 |
| random (bool)      | 100000000    | 5000000     | 5.54                 | 4.91               | 1.128 |
| random (bool)      | 100000000    | 25000000    | 5.86                 | 4.94               | 1.186 |
| random (bool)      | 100000000    | 50000000    | 5.73                 | 5.16               | 1.112 |

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