# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random_u32         | 1000         | 1           | 1253.577             | 647.359            | 1.936 |
| random_u32         | 1000         | 10          | 1151.491             | 648.058            | 1.777 |
| random_u32         | 1000         | 50          | 918.661              | 631.941            | 1.454 |
| random_u32         | 1000         | 250         | 673.768              | 572.215            | 1.177 |
| random_u32         | 1000         | 500         | 644.904              | 564.557            | 1.142 |
| random_u32         | 10000        | 10          | 1989.283             | 951.430            | 2.091 |
| random_u32         | 10000        | 100         | 1843.052             | 940.353            | 1.960 |
| random_u32         | 10000        | 500         | 1390.027             | 922.338            | 1.507 |
| random_u32         | 10000        | 2500        | 1072.460             | 852.243            | 1.258 |
| random_u32         | 10000        | 5000        | 965.942              | 791.517            | 1.220 |
| random_u32         | 100000       | 100         | 2513.135             | 1008.955           | 2.491 |
| random_u32         | 100000       | 1000        | 2288.340             | 992.430            | 2.306 |
| random_u32         | 100000       | 5000        | 1624.656             | 971.458            | 1.672 |
| random_u32         | 100000       | 25000       | 1215.702             | 885.226            | 1.373 |
| random_u32         | 100000       | 50000       | 1193.759             | 825.991            | 1.445 |
| random_u32         | 1000000      | 1000        | 2738.696             | 993.061            | 2.758 |
| random_u32         | 1000000      | 10000       | 2511.273             | 1002.882           | 2.504 |
| random_u32         | 1000000      | 50000       | 1753.258             | 970.319            | 1.807 |
| random_u32         | 1000000      | 250000      | 1322.535             | 879.001            | 1.505 |
| random_u32         | 1000000      | 500000      | 1270.567             | 834.255            | 1.523 |
| random_u32         | 10000000     | 10000       | 2650.901             | 978.075            | 2.710 |
| random_u32         | 10000000     | 100000      | 2437.597             | 965.822            | 2.524 |
| random_u32         | 10000000     | 500000      | 1914.905             | 969.381            | 1.975 |
| random_u32         | 10000000     | 2500000     | 1448.555             | 872.765            | 1.660 |
| random_u32         | 10000000     | 5000000     | 1283.337             | 814.848            | 1.575 |
| random_u32         | 100000000    | 100000      | 2724.915             | 963.882            | 2.827 |
| random_u32         | 100000000    | 1000000     | 2538.871             | 997.175            | 2.546 |
| random_u32         | 100000000    | 5000000     | 1849.711             | 949.800            | 1.947 |
| random_u32         | 100000000    | 25000000    | 1460.367             | 877.086            | 1.665 |
| random_u32         | 100000000    | 50000000    | 1299.199             | 796.320            | 1.632 |
| sawtooth_u32       | 1000         | 1           | 1294.494             | 1050.295           | 1.233 |
| sawtooth_u32       | 1000         | 10          | 1278.230             | 1017.520           | 1.256 |
| sawtooth_u32       | 1000         | 50          | 1324.750             | 989.643            | 1.339 |
| sawtooth_u32       | 1000         | 250         | 954.089              | 903.245            | 1.056 |
| sawtooth_u32       | 1000         | 500         | 882.003              | 901.025            | 0.979 |
| sawtooth_u32       | 10000        | 10          | 2022.846             | 1152.421           | 1.755 |
| sawtooth_u32       | 10000        | 100         | 1953.114             | 1105.525           | 1.767 |
| sawtooth_u32       | 10000        | 500         | 1307.492             | 1015.601           | 1.287 |
| sawtooth_u32       | 10000        | 2500        | 1115.545             | 885.738            | 1.259 |
| sawtooth_u32       | 10000        | 5000        | 1015.936             | 801.664            | 1.267 |
| sawtooth_u32       | 100000       | 100         | 2662.623             | 1043.317           | 2.552 |
| sawtooth_u32       | 100000       | 1000        | 2394.424             | 1020.180           | 2.347 |
| sawtooth_u32       | 100000       | 5000        | 1728.735             | 982.351            | 1.760 |
| sawtooth_u32       | 100000       | 25000       | 1271.914             | 853.346            | 1.491 |
| sawtooth_u32       | 100000       | 50000       | 1059.973             | 701.597            | 1.511 |
| sawtooth_u32       | 1000000      | 1000        | 3026.871             | 1067.550           | 2.835 |
| sawtooth_u32       | 1000000      | 10000       | 2840.434             | 1044.986           | 2.718 |
| sawtooth_u32       | 1000000      | 50000       | 1972.379             | 991.371            | 1.990 |
| sawtooth_u32       | 1000000      | 250000      | 1389.757             | 833.552            | 1.667 |
| sawtooth_u32       | 1000000      | 500000      | 1311.206             | 790.450            | 1.659 |
| sawtooth_u32       | 10000000     | 10000       | 2976.921             | 1042.696           | 2.855 |
| sawtooth_u32       | 10000000     | 100000      | 2712.886             | 1025.191           | 2.646 |
| sawtooth_u32       | 10000000     | 500000      | 2202.369             | 960.758            | 2.292 |
| sawtooth_u32       | 10000000     | 2500000     | 1464.090             | 789.419            | 1.855 |
| sawtooth_u32       | 10000000     | 5000000     | 1158.411             | 676.281            | 1.713 |
| sawtooth_u32       | 100000000    | 100000      | 2792.575             | 927.387            | 3.011 |
| sawtooth_u32       | 100000000    | 1000000     | 2757.103             | 955.536            | 2.885 |
| sawtooth_u32       | 100000000    | 5000000     | 2032.952             | 927.371            | 2.192 |
| sawtooth_u32       | 100000000    | 25000000    | 1589.259             | 792.907            | 2.004 |
| sawtooth_u32       | 100000000    | 50000000    | 1440.598             | 831.382            | 1.733 |
| reversed_u32       | 1000         | 1           | 1607.069             | 910.190            | 1.766 |
| reversed_u32       | 1000         | 10          | 1404.640             | 912.683            | 1.539 |
| reversed_u32       | 1000         | 50          | 1343.447             | 894.221            | 1.502 |
| reversed_u32       | 1000         | 250         | 980.100              | 883.916            | 1.109 |
| reversed_u32       | 1000         | 500         | 689.519              | 805.906            | 0.856 |
| reversed_u32       | 10000        | 10          | 2499.484             | 1056.237           | 2.366 |
| reversed_u32       | 10000        | 100         | 2510.194             | 1062.055           | 2.364 |
| reversed_u32       | 10000        | 500         | 2218.052             | 1060.002           | 2.092 |
| reversed_u32       | 10000        | 2500        | 1039.905             | 1052.405           | 0.988 |
| reversed_u32       | 10000        | 5000        | 1123.471             | 982.649            | 1.143 |
| reversed_u32       | 100000       | 100         | 2902.355             | 1057.418           | 2.745 |
| reversed_u32       | 100000       | 1000        | 2711.417             | 1054.036           | 2.572 |
| reversed_u32       | 100000       | 5000        | 2434.465             | 1045.064           | 2.329 |
| reversed_u32       | 100000       | 25000       | 1291.537             | 1046.924           | 1.234 |
| reversed_u32       | 100000       | 50000       | 1284.977             | 1094.521           | 1.174 |
| reversed_u32       | 1000000      | 1000        | 3134.903             | 1145.964           | 2.736 |
| reversed_u32       | 1000000      | 10000       | 2899.682             | 1143.315           | 2.536 |
| reversed_u32       | 1000000      | 50000       | 2061.458             | 1068.637           | 1.929 |
| reversed_u32       | 1000000      | 250000      | 1521.206             | 916.388            | 1.660 |
| reversed_u32       | 1000000      | 500000      | 1434.118             | 857.105            | 1.673 |
| reversed_u32       | 10000000     | 10000       | 2947.074             | 1037.099           | 2.842 |
| reversed_u32       | 10000000     | 100000      | 2612.971             | 1046.643           | 2.497 |
| reversed_u32       | 10000000     | 500000      | 2068.848             | 971.658            | 2.129 |
| reversed_u32       | 10000000     | 2500000     | 1565.486             | 830.525            | 1.885 |
| reversed_u32       | 10000000     | 5000000     | 1342.752             | 790.764            | 1.698 |
| reversed_u32       | 100000000    | 100000      | 1540.089             | 1061.720           | 1.451 |
| reversed_u32       | 100000000    | 1000000     | 1627.282             | 1052.422           | 1.546 |
| reversed_u32       | 100000000    | 5000000     | 1389.367             | 945.496            | 1.469 |
| reversed_u32       | 100000000    | 25000000    | 1507.879             | 790.119            | 1.908 |
| reversed_u32       | 100000000    | 50000000    | 1253.091             | 722.577            | 1.734 |
| randomdups_u32     | 1000         | 1           | 1188.371             | 764.981            | 1.553 |
| randomdups_u32     | 1000         | 10          | 1175.154             | 760.294            | 1.546 |
| randomdups_u32     | 1000         | 50          | 886.279              | 672.728            | 1.317 |
| randomdups_u32     | 1000         | 250         | 666.709              | 586.487            | 1.137 |
| randomdups_u32     | 1000         | 500         | 659.860              | 589.284            | 1.120 |
| randomdups_u32     | 10000        | 10          | 1944.440             | 989.558            | 1.965 |
| randomdups_u32     | 10000        | 100         | 1759.871             | 950.666            | 1.851 |
| randomdups_u32     | 10000        | 500         | 1334.589             | 938.987            | 1.421 |
| randomdups_u32     | 10000        | 2500        | 1000.033             | 789.430            | 1.267 |
| randomdups_u32     | 10000        | 5000        | 916.754              | 742.639            | 1.234 |
| randomdups_u32     | 100000       | 100         | 2471.029             | 992.148            | 2.491 |
| randomdups_u32     | 100000       | 1000        | 2080.860             | 945.895            | 2.200 |
| randomdups_u32     | 100000       | 5000        | 1553.899             | 942.882            | 1.648 |
| randomdups_u32     | 100000       | 25000       | 1161.737             | 855.837            | 1.357 |
| randomdups_u32     | 100000       | 50000       | 1164.366             | 813.601            | 1.431 |
| randomdups_u32     | 1000000      | 1000        | 2636.352             | 984.608            | 2.678 |
| randomdups_u32     | 1000000      | 10000       | 2423.303             | 967.397            | 2.505 |
| randomdups_u32     | 1000000      | 50000       | 1677.773             | 936.949            | 1.791 |
| randomdups_u32     | 1000000      | 250000      | 1261.044             | 860.825            | 1.465 |
| randomdups_u32     | 1000000      | 500000      | 1236.168             | 814.102            | 1.518 |
| randomdups_u32     | 10000000     | 10000       | 2538.293             | 936.187            | 2.711 |
| randomdups_u32     | 10000000     | 100000      | 2397.320             | 939.075            | 2.553 |
| randomdups_u32     | 10000000     | 500000      | 1810.048             | 919.373            | 1.969 |
| randomdups_u32     | 10000000     | 2500000     | 1388.513             | 853.209            | 1.627 |
| randomdups_u32     | 10000000     | 5000000     | 1231.129             | 779.235            | 1.580 |
| randomdups_u32     | 100000000    | 100000      | 2652.137             | 982.244            | 2.700 |
| randomdups_u32     | 100000000    | 1000000     | 2500.534             | 954.267            | 2.620 |
| randomdups_u32     | 100000000    | 5000000     | 1866.520             | 982.239            | 1.900 |
| randomdups_u32     | 100000000    | 25000000    | 1468.493             | 870.491            | 1.687 |
| randomdups_u32     | 100000000    | 50000000    | 1312.953             | 822.921            | 1.595 |
| random_bool        | 1000         | 1           | 638.983              | 512.364            | 1.247 |
| random_bool        | 1000         | 10          | 648.076              | 512.776            | 1.264 |
| random_bool        | 1000         | 50          | 644.527              | 510.418            | 1.263 |
| random_bool        | 1000         | 250         | 616.277              | 490.698            | 1.256 |
| random_bool        | 1000         | 500         | 591.078              | 511.826            | 1.155 |
| random_bool        | 10000        | 10          | 686.903              | 539.021            | 1.274 |
| random_bool        | 10000        | 100         | 688.148              | 538.097            | 1.279 |
| random_bool        | 10000        | 500         | 688.961              | 540.536            | 1.275 |
| random_bool        | 10000        | 2500        | 679.280              | 528.696            | 1.285 |
| random_bool        | 10000        | 5000        | 666.180              | 531.567            | 1.253 |
| random_bool        | 100000       | 100         | 697.307              | 541.190            | 1.288 |
| random_bool        | 100000       | 1000        | 699.881              | 537.067            | 1.303 |
| random_bool        | 100000       | 5000        | 697.196              | 541.559            | 1.287 |
| random_bool        | 100000       | 25000       | 699.342              | 537.141            | 1.302 |
| random_bool        | 100000       | 50000       | 694.291              | 545.621            | 1.272 |
| random_bool        | 1000000      | 1000        | 711.222              | 550.326            | 1.292 |
| random_bool        | 1000000      | 10000       | 706.749              | 528.336            | 1.338 |
| random_bool        | 1000000      | 50000       | 714.595              | 549.069            | 1.301 |
| random_bool        | 1000000      | 250000      | 711.332              | 542.939            | 1.310 |
| random_bool        | 1000000      | 500000      | 715.003              | 549.637            | 1.301 |
| random_bool        | 10000000     | 10000       | 702.283              | 572.824            | 1.226 |
| random_bool        | 10000000     | 100000      | 707.390              | 568.057            | 1.245 |
| random_bool        | 10000000     | 500000      | 711.667              | 572.258            | 1.244 |
| random_bool        | 10000000     | 2500000     | 710.189              | 541.026            | 1.313 |
| random_bool        | 10000000     | 5000000     | 726.114              | 521.828            | 1.391 |
| random_bool        | 100000000    | 100000      | 711.113              | 577.394            | 1.232 |
| random_bool        | 100000000    | 1000000     | 713.642              | 527.293            | 1.353 |
| random_bool        | 100000000    | 5000000     | 712.665              | 538.907            | 1.322 |
| random_bool        | 100000000    | 25000000    | 711.496              | 562.835            | 1.264 |
| random_bool        | 100000000    | 50000000    | 747.640              | 628.396            | 1.190 |

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