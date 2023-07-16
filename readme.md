# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random_u32         | 1000         | 1           | 1356.138             | 691.172            | 1.962 |
| random_u32         | 1000         | 10          | 1189.546             | 666.546            | 1.785 |
| random_u32         | 1000         | 50          | 942.821              | 645.015            | 1.462 |
| random_u32         | 1000         | 250         | 689.924              | 592.729            | 1.164 |
| random_u32         | 1000         | 500         | 645.860              | 565.269            | 1.143 |
| random_u32         | 10000        | 10          | 2217.536             | 868.776            | 2.552 |
| random_u32         | 10000        | 100         | 1782.012             | 867.059            | 2.055 |
| random_u32         | 10000        | 500         | 1401.961             | 855.480            | 1.639 |
| random_u32         | 10000        | 2500        | 1028.173             | 787.980            | 1.305 |
| random_u32         | 10000        | 5000        | 965.589              | 767.137            | 1.259 |
| random_u32         | 100000       | 100         | 2599.723             | 969.827            | 2.681 |
| random_u32         | 100000       | 1000        | 2179.713             | 936.992            | 2.326 |
| random_u32         | 100000       | 5000        | 1816.028             | 956.471            | 1.899 |
| random_u32         | 100000       | 25000       | 1328.183             | 875.181            | 1.518 |
| random_u32         | 100000       | 50000       | 1201.660             | 827.308            | 1.452 |
| random_u32         | 1000000      | 1000        | 2741.789             | 984.877            | 2.784 |
| random_u32         | 1000000      | 10000       | 2556.331             | 1004.291           | 2.545 |
| random_u32         | 1000000      | 50000       | 2095.718             | 972.289            | 2.155 |
| random_u32         | 1000000      | 250000      | 1484.217             | 876.235            | 1.694 |
| random_u32         | 1000000      | 500000      | 1322.755             | 833.113            | 1.588 |
| random_u32         | 10000000     | 10000       | 2489.206             | 963.841            | 2.583 |
| random_u32         | 10000000     | 100000      | 2372.327             | 948.773            | 2.500 |
| random_u32         | 10000000     | 500000      | 2052.002             | 949.918            | 2.160 |
| random_u32         | 10000000     | 2500000     | 1482.685             | 861.392            | 1.721 |
| random_u32         | 10000000     | 5000000     | 1300.760             | 808.629            | 1.609 |
| random_u32         | 100000000    | 100000      | 2578.814             | 948.453            | 2.719 |
| random_u32         | 100000000    | 1000000     | 2428.456             | 974.719            | 2.491 |
| random_u32         | 100000000    | 5000000     | 2103.741             | 930.448            | 2.261 |
| random_u32         | 100000000    | 25000000    | 1540.507             | 859.616            | 1.792 |
| random_u32         | 100000000    | 50000000    | 1328.281             | 778.609            | 1.706 |
| sawtooth_u32       | 1000         | 1           | 1162.435             | 1056.637           | 1.100 |
| sawtooth_u32       | 1000         | 10          | 1277.661             | 1025.713           | 1.246 |
| sawtooth_u32       | 1000         | 50          | 1301.726             | 1008.210           | 1.291 |
| sawtooth_u32       | 1000         | 250         | 995.681              | 964.804            | 1.032 |
| sawtooth_u32       | 1000         | 500         | 923.573              | 972.772            | 0.949 |
| sawtooth_u32       | 10000        | 10          | 1523.311             | 1055.294           | 1.443 |
| sawtooth_u32       | 10000        | 100         | 1692.967             | 1071.934           | 1.579 |
| sawtooth_u32       | 10000        | 500         | 1315.841             | 946.022            | 1.391 |
| sawtooth_u32       | 10000        | 2500        | 997.757              | 827.017            | 1.206 |
| sawtooth_u32       | 10000        | 5000        | 1030.741             | 790.664            | 1.304 |
| sawtooth_u32       | 100000       | 100         | 2445.496             | 1071.070           | 2.283 |
| sawtooth_u32       | 100000       | 1000        | 2248.850             | 1027.708           | 2.188 |
| sawtooth_u32       | 100000       | 5000        | 1771.710             | 985.333            | 1.798 |
| sawtooth_u32       | 100000       | 25000       | 1322.865             | 869.130            | 1.522 |
| sawtooth_u32       | 100000       | 50000       | 1187.634             | 808.046            | 1.470 |
| sawtooth_u32       | 1000000      | 1000        | 2711.537             | 1079.956           | 2.511 |
| sawtooth_u32       | 1000000      | 10000       | 2603.239             | 1053.432           | 2.471 |
| sawtooth_u32       | 1000000      | 50000       | 2126.615             | 1019.545           | 2.086 |
| sawtooth_u32       | 1000000      | 250000      | 1479.831             | 850.766            | 1.739 |
| sawtooth_u32       | 1000000      | 500000      | 1325.512             | 802.390            | 1.652 |
| sawtooth_u32       | 10000000     | 10000       | 2426.534             | 1032.208           | 2.351 |
| sawtooth_u32       | 10000000     | 100000      | 2309.193             | 1012.208           | 2.281 |
| sawtooth_u32       | 10000000     | 500000      | 2066.161             | 945.277            | 2.186 |
| sawtooth_u32       | 10000000     | 2500000     | 1522.825             | 835.298            | 1.823 |
| sawtooth_u32       | 10000000     | 5000000     | 1355.024             | 796.131            | 1.702 |
| sawtooth_u32       | 100000000    | 100000      | 2819.946             | 1005.855           | 2.804 |
| sawtooth_u32       | 100000000    | 1000000     | 2658.066             | 981.598            | 2.708 |
| sawtooth_u32       | 100000000    | 5000000     | 2341.606             | 947.790            | 2.471 |
| sawtooth_u32       | 100000000    | 25000000    | 1702.015             | 813.568            | 2.092 |
| sawtooth_u32       | 100000000    | 50000000    | 1437.747             | 811.348            | 1.772 |
| reversed_u32       | 1000         | 1           | 1697.399             | 973.466            | 1.744 |
| reversed_u32       | 1000         | 10          | 1455.928             | 940.540            | 1.548 |
| reversed_u32       | 1000         | 50          | 1574.803             | 938.739            | 1.678 |
| reversed_u32       | 1000         | 250         | 954.129              | 896.817            | 1.064 |
| reversed_u32       | 1000         | 500         | 864.093              | 838.825            | 1.030 |
| reversed_u32       | 10000        | 10          | 2740.389             | 977.616            | 2.803 |
| reversed_u32       | 10000        | 100         | 1435.110             | 991.478            | 1.447 |
| reversed_u32       | 10000        | 500         | 961.282              | 991.820            | 0.969 |
| reversed_u32       | 10000        | 2500        | 1215.445             | 995.304            | 1.221 |
| reversed_u32       | 10000        | 5000        | 1092.076             | 935.607            | 1.167 |
| reversed_u32       | 100000       | 100         | 3143.370             | 1042.256           | 3.016 |
| reversed_u32       | 100000       | 1000        | 2943.774             | 1037.488           | 2.837 |
| reversed_u32       | 100000       | 5000        | 1865.156             | 1038.605           | 1.796 |
| reversed_u32       | 100000       | 25000       | 1540.110             | 1042.258           | 1.478 |
| reversed_u32       | 100000       | 50000       | 1329.219             | 1075.913           | 1.235 |
| reversed_u32       | 1000000      | 1000        | 3203.856             | 1168.510           | 2.742 |
| reversed_u32       | 1000000      | 10000       | 3023.277             | 1171.899           | 2.580 |
| reversed_u32       | 1000000      | 50000       | 2469.674             | 1090.982           | 2.264 |
| reversed_u32       | 1000000      | 250000      | 1720.882             | 937.384            | 1.836 |
| reversed_u32       | 1000000      | 500000      | 1527.400             | 881.424            | 1.733 |
| reversed_u32       | 10000000     | 10000       | 1737.080             | 1031.033           | 1.685 |
| reversed_u32       | 10000000     | 100000      | 1678.297             | 1033.975           | 1.623 |
| reversed_u32       | 10000000     | 500000      | 1603.920             | 971.139            | 1.652 |
| reversed_u32       | 10000000     | 2500000     | 1252.101             | 823.735            | 1.520 |
| reversed_u32       | 10000000     | 5000000     | 1174.244             | 787.079            | 1.492 |
| reversed_u32       | 100000000    | 100000      | 1458.915             | 1046.386           | 1.394 |
| reversed_u32       | 100000000    | 1000000     | 1458.055             | 1039.844           | 1.402 |
| reversed_u32       | 100000000    | 5000000     | 1317.352             | 930.594            | 1.416 |
| reversed_u32       | 100000000    | 25000000    | 1170.696             | 800.218            | 1.463 |
| reversed_u32       | 100000000    | 50000000    | 1226.110             | 738.498            | 1.660 |
| randomdups_u32     | 1000         | 1           | 1222.644             | 782.891            | 1.562 |
| randomdups_u32     | 1000         | 10          | 1205.945             | 770.600            | 1.565 |
| randomdups_u32     | 1000         | 50          | 1017.558             | 731.247            | 1.392 |
| randomdups_u32     | 1000         | 250         | 768.272              | 648.856            | 1.184 |
| randomdups_u32     | 1000         | 500         | 692.130              | 575.626            | 1.202 |
| randomdups_u32     | 10000        | 10          | 1546.394             | 930.921            | 1.661 |
| randomdups_u32     | 10000        | 100         | 1638.714             | 915.553            | 1.790 |
| randomdups_u32     | 10000        | 500         | 1321.258             | 877.738            | 1.505 |
| randomdups_u32     | 10000        | 2500        | 1025.737             | 810.129            | 1.266 |
| randomdups_u32     | 10000        | 5000        | 977.134              | 783.823            | 1.247 |
| randomdups_u32     | 100000       | 100         | 2504.801             | 1007.846           | 2.485 |
| randomdups_u32     | 100000       | 1000        | 2060.977             | 985.913            | 2.090 |
| randomdups_u32     | 100000       | 5000        | 1735.904             | 969.247            | 1.791 |
| randomdups_u32     | 100000       | 25000       | 1305.192             | 888.112            | 1.470 |
| randomdups_u32     | 100000       | 50000       | 1195.743             | 837.648            | 1.428 |
| randomdups_u32     | 1000000      | 1000        | 2617.096             | 1015.483           | 2.577 |
| randomdups_u32     | 1000000      | 10000       | 2490.844             | 1002.728           | 2.484 |
| randomdups_u32     | 1000000      | 50000       | 2023.517             | 980.820            | 2.063 |
| randomdups_u32     | 1000000      | 250000      | 1451.645             | 908.911            | 1.597 |
| randomdups_u32     | 1000000      | 500000      | 1311.585             | 844.925            | 1.552 |
| randomdups_u32     | 10000000     | 10000       | 2338.516             | 966.586            | 2.419 |
| randomdups_u32     | 10000000     | 100000      | 2243.327             | 963.519            | 2.328 |
| randomdups_u32     | 10000000     | 500000      | 1982.992             | 947.991            | 2.092 |
| randomdups_u32     | 10000000     | 2500000     | 1447.334             | 882.233            | 1.641 |
| randomdups_u32     | 10000000     | 5000000     | 1275.303             | 806.566            | 1.581 |
| randomdups_u32     | 100000000    | 100000      | 2512.645             | 991.820            | 2.533 |
| randomdups_u32     | 100000000    | 1000000     | 2400.685             | 930.029            | 2.581 |
| randomdups_u32     | 100000000    | 5000000     | 2048.548             | 957.133            | 2.140 |
| randomdups_u32     | 100000000    | 25000000    | 1495.274             | 840.362            | 1.779 |
| randomdups_u32     | 100000000    | 50000000    | 1326.116             | 799.223            | 1.659 |
| random_bool        | 1000         | 1           | 621.945              | 477.082            | 1.304 |
| random_bool        | 1000         | 10          | 617.078              | 469.825            | 1.313 |
| random_bool        | 1000         | 50          | 611.709              | 470.260            | 1.301 |
| random_bool        | 1000         | 250         | 608.017              | 466.685            | 1.303 |
| random_bool        | 1000         | 500         | 635.144              | 478.932            | 1.326 |
| random_bool        | 10000        | 10          | 654.122              | 478.224            | 1.368 |
| random_bool        | 10000        | 100         | 678.225              | 496.103            | 1.367 |
| random_bool        | 10000        | 500         | 668.907              | 491.089            | 1.362 |
| random_bool        | 10000        | 2500        | 663.404              | 484.733            | 1.369 |
| random_bool        | 10000        | 5000        | 745.111              | 492.603            | 1.513 |
| random_bool        | 100000       | 100         | 674.817              | 496.657            | 1.359 |
| random_bool        | 100000       | 1000        | 669.421              | 490.309            | 1.365 |
| random_bool        | 100000       | 5000        | 680.517              | 498.825            | 1.364 |
| random_bool        | 100000       | 25000       | 674.718              | 494.203            | 1.365 |
| random_bool        | 100000       | 50000       | 764.582              | 501.186            | 1.526 |
| random_bool        | 1000000      | 1000        | 682.024              | 504.882            | 1.351 |
| random_bool        | 1000000      | 10000       | 681.064              | 487.811            | 1.396 |
| random_bool        | 1000000      | 50000       | 683.889              | 507.470            | 1.348 |
| random_bool        | 1000000      | 250000      | 675.720              | 492.663            | 1.372 |
| random_bool        | 1000000      | 500000      | 847.098              | 513.829            | 1.649 |
| random_bool        | 10000000     | 10000       | 679.976              | 519.920            | 1.308 |
| random_bool        | 10000000     | 100000      | 682.051              | 516.486            | 1.321 |
| random_bool        | 10000000     | 500000      | 675.722              | 519.079            | 1.302 |
| random_bool        | 10000000     | 2500000     | 678.541              | 489.749            | 1.385 |
| random_bool        | 10000000     | 5000000     | 891.367              | 477.957            | 1.865 |
| random_bool        | 100000000    | 100000      | 680.544              | 521.414            | 1.305 |
| random_bool        | 100000000    | 1000000     | 676.870              | 478.915            | 1.413 |
| random_bool        | 100000000    | 5000000     | 680.069              | 488.293            | 1.393 |
| random_bool        | 100000000    | 25000000    | 678.556              | 510.529            | 1.329 |
| random_bool        | 100000000    | 50000000    | 955.191              | 573.047            | 1.667 |

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