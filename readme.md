# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random (u32)       | 1000         | 1           | 1088.852             | 670.329            | 1.624 |
| random (u32)       | 1000         | 10          | 1042.716             | 651.735            | 1.600 |
| random (u32)       | 1000         | 50          | 910.980              | 639.008            | 1.426 |
| random (u32)       | 1000         | 250         | 672.161              | 582.996            | 1.153 |
| random (u32)       | 1000         | 500         | 634.129              | 566.600            | 1.119 |
| random (u32)       | 10000        | 10          | 1892.638             | 952.656            | 1.987 |
| random (u32)       | 10000        | 100         | 1779.352             | 945.785            | 1.881 |
| random (u32)       | 10000        | 500         | 1476.345             | 919.251            | 1.606 |
| random (u32)       | 10000        | 2500        | 1086.923             | 842.990            | 1.289 |
| random (u32)       | 10000        | 5000        | 977.217              | 798.042            | 1.225 |
| random (u32)       | 1000000      | 1000        | 2472.393             | 996.049            | 2.482 |
| random (u32)       | 1000000      | 10000       | 2410.114             | 989.789            | 2.435 |
| random (u32)       | 1000000      | 50000       | 2050.817             | 970.749            | 2.113 |
| random (u32)       | 1000000      | 250000      | 1394.985             | 882.448            | 1.581 |
| random (u32)       | 1000000      | 500000      | 1244.337             | 833.239            | 1.493 |
| random (u32)       | 100000000    | 100000      | 2198.244             | 921.452            | 2.386 |
| random (u32)       | 100000000    | 1000000     | 2263.925             | 970.615            | 2.332 |
| random (u32)       | 100000000    | 5000000     | 1914.996             | 940.598            | 2.036 |
| random (u32)       | 100000000    | 25000000    | 1311.584             | 846.871            | 1.549 |
| random (u32)       | 100000000    | 50000000    | 1180.714             | 776.832            | 1.520 |
| sawtooth (u32)     | 1000         | 1           | 1347.199             | 1026.172           | 1.313 |
| sawtooth (u32)     | 1000         | 10          | 1417.828             | 1035.676           | 1.369 |
| sawtooth (u32)     | 1000         | 50          | 1354.556             | 976.324            | 1.387 |
| sawtooth (u32)     | 1000         | 250         | 777.635              | 924.797            | 0.841 |
| sawtooth (u32)     | 1000         | 500         | 741.467              | 871.437            | 0.851 |
| sawtooth (u32)     | 10000        | 10          | 2007.889             | 1126.943           | 1.782 |
| sawtooth (u32)     | 10000        | 100         | 1992.457             | 1118.339           | 1.782 |
| sawtooth (u32)     | 10000        | 500         | 1544.674             | 995.544            | 1.552 |
| sawtooth (u32)     | 10000        | 2500        | 1127.873             | 865.166            | 1.304 |
| sawtooth (u32)     | 10000        | 5000        | 1013.416             | 794.787            | 1.275 |
| sawtooth (u32)     | 1000000      | 1000        | 2843.456             | 1074.767           | 2.646 |
| sawtooth (u32)     | 1000000      | 10000       | 2774.208             | 1060.673           | 2.616 |
| sawtooth (u32)     | 1000000      | 50000       | 2295.359             | 988.367            | 2.322 |
| sawtooth (u32)     | 1000000      | 250000      | 1485.066             | 846.704            | 1.754 |
| sawtooth (u32)     | 1000000      | 500000      | 1310.387             | 807.845            | 1.622 |
| sawtooth (u32)     | 100000000    | 100000      | 2698.264             | 1041.799           | 2.590 |
| sawtooth (u32)     | 100000000    | 1000000     | 2638.947             | 1048.368           | 2.517 |
| sawtooth (u32)     | 100000000    | 5000000     | 2235.120             | 960.811            | 2.326 |
| sawtooth (u32)     | 100000000    | 25000000    | 1493.187             | 846.780            | 1.763 |
| sawtooth (u32)     | 100000000    | 50000000    | 1336.423             | 792.232            | 1.687 |
| reversed (u32)     | 1000         | 1           | 1287.218             | 955.299            | 1.347 |
| reversed (u32)     | 1000         | 10          | 1197.484             | 952.091            | 1.258 |
| reversed (u32)     | 1000         | 50          | 1136.641             | 912.910            | 1.245 |
| reversed (u32)     | 1000         | 250         | 777.824              | 908.182            | 0.856 |
| reversed (u32)     | 1000         | 500         | 659.801              | 840.451            | 0.785 |
| reversed (u32)     | 10000        | 10          | 2197.333             | 1070.247           | 2.053 |
| reversed (u32)     | 10000        | 100         | 2148.557             | 1074.212           | 2.000 |
| reversed (u32)     | 10000        | 500         | 1712.423             | 1070.415           | 1.600 |
| reversed (u32)     | 10000        | 2500        | 1218.075             | 1062.104           | 1.147 |
| reversed (u32)     | 10000        | 5000        | 997.120              | 999.207            | 0.998 |
| reversed (u32)     | 1000000      | 1000        | 2968.399             | 1173.523           | 2.529 |
| reversed (u32)     | 1000000      | 10000       | 2860.999             | 1146.814           | 2.495 |
| reversed (u32)     | 1000000      | 50000       | 2418.349             | 1086.332           | 2.226 |
| reversed (u32)     | 1000000      | 250000      | 1624.865             | 932.670            | 1.742 |
| reversed (u32)     | 1000000      | 500000      | 1443.076             | 878.060            | 1.643 |
| reversed (u32)     | 100000000    | 100000      | 2373.326             | 1046.592           | 2.268 |
| reversed (u32)     | 100000000    | 1000000     | 2382.272             | 1046.322           | 2.277 |
| reversed (u32)     | 100000000    | 5000000     | 1855.706             | 933.902            | 1.987 |
| reversed (u32)     | 100000000    | 25000000    | 1414.834             | 812.969            | 1.740 |
| reversed (u32)     | 100000000    | 50000000    | 1213.667             | 731.946            | 1.658 |
| random dups (u32s) | 1000         | 1           | 1168.302             | 783.811            | 1.491 |
| random dups (u32s) | 1000         | 10          | 1114.720             | 772.504            | 1.443 |
| random dups (u32s) | 1000         | 50          | 924.743              | 722.685            | 1.280 |
| random dups (u32s) | 1000         | 250         | 722.007              | 637.565            | 1.132 |
| random dups (u32s) | 1000         | 500         | 642.500              | 601.822            | 1.068 |
| random dups (u32s) | 10000        | 10          | 1911.581             | 993.582            | 1.924 |
| random dups (u32s) | 10000        | 100         | 1843.957             | 982.059            | 1.878 |
| random dups (u32s) | 10000        | 500         | 1487.763             | 941.845            | 1.580 |
| random dups (u32s) | 10000        | 2500        | 1113.197             | 856.216            | 1.300 |
| random dups (u32s) | 10000        | 5000        | 989.628              | 800.358            | 1.236 |
| random dups (u32s) | 1000000      | 1000        | 2505.324             | 1004.283           | 2.495 |
| random dups (u32s) | 1000000      | 10000       | 2441.864             | 1000.798           | 2.440 |
| random dups (u32s) | 1000000      | 50000       | 2078.664             | 981.684            | 2.117 |
| random dups (u32s) | 1000000      | 250000      | 1415.556             | 897.527            | 1.577 |
| random dups (u32s) | 1000000      | 500000      | 1256.982             | 841.371            | 1.494 |
| random dups (u32s) | 100000000    | 100000      | 2324.620             | 955.151            | 2.434 |
| random dups (u32s) | 100000000    | 1000000     | 2276.144             | 964.755            | 2.359 |
| random dups (u32s) | 100000000    | 5000000     | 1915.039             | 920.472            | 2.080 |
| random dups (u32s) | 100000000    | 25000000    | 1330.041             | 861.303            | 1.544 |
| random dups (u32s) | 100000000    | 50000000    | 1205.541             | 794.234            | 1.518 |
| random (bool)      | 1000         | 1           | 491.364              | 516.676            | 0.951 |
| random (bool)      | 1000         | 10          | 487.097              | 514.056            | 0.948 |
| random (bool)      | 1000         | 50          | 486.006              | 515.333            | 0.943 |
| random (bool)      | 1000         | 250         | 541.140              | 510.411            | 1.060 |
| random (bool)      | 1000         | 500         | 467.896              | 502.417            | 0.931 |
| random (bool)      | 10000        | 10          | 530.163              | 540.029            | 0.982 |
| random (bool)      | 10000        | 100         | 529.402              | 539.221            | 0.982 |
| random (bool)      | 10000        | 500         | 531.947              | 543.269            | 0.979 |
| random (bool)      | 10000        | 2500        | 530.367              | 540.616            | 0.981 |
| random (bool)      | 10000        | 5000        | 479.848              | 537.789            | 0.892 |
| random (bool)      | 1000000      | 1000        | 544.935              | 546.024            | 0.998 |
| random (bool)      | 1000000      | 10000       | 545.920              | 548.443            | 0.995 |
| random (bool)      | 1000000      | 50000       | 546.870              | 541.734            | 1.009 |
| random (bool)      | 1000000      | 250000      | 545.949              | 549.877            | 0.993 |
| random (bool)      | 1000000      | 500000      | 486.506              | 557.719            | 0.872 |
| random (bool)      | 100000000    | 100000      | 546.995              | 570.964            | 0.958 |
| random (bool)      | 100000000    | 1000000     | 546.472              | 562.693            | 0.971 |
| random (bool)      | 100000000    | 5000000     | 547.280              | 574.089            | 0.953 |
| random (bool)      | 100000000    | 25000000    | 547.182              | 577.042            | 0.948 |
| random (bool)      | 100000000    | 50000000    | 496.940              | 571.197            | 0.870 |

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