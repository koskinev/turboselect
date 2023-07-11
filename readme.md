# Turboselect

An alternative implementation of the `slice::select_nth_unstable` method based on the Floyd & Rivest SELECT algorithm, demonstrating that the Floyd & Rivest algorithm can be faster than well implemented quickselect. The speed improvements are most noticeable for indices far from the median. Note that the code relies heavily on unsafe code and currently not thoroughly tested. 

To run the tests, use `cargo test` and `cargo +nightly miri test`.

**Comparison with  `slice::select_nth_unstable` as the baseline**

| data type          | slice length | index       | throughput, M el/s   | baseline, M el /s  | ratio |
| ------------------ | ------------ | ----------- | -------------------- | ------------------ | ----- |
| random_u32         | 1000         | 1           | 1170.741             | 669.024            | 1.750 |
| random_u32         | 1000         | 10          | 1087.292             | 651.318            | 1.669 |
| random_u32         | 1000         | 50          | 946.288              | 638.258            | 1.483 |
| random_u32         | 1000         | 250         | 685.112              | 581.671            | 1.178 |
| random_u32         | 1000         | 500         | 635.920              | 559.891            | 1.136 |
| random_u32         | 10000        | 10          | 1882.503             | 930.960            | 2.022 |
| random_u32         | 10000        | 100         | 1787.663             | 935.919            | 1.910 |
| random_u32         | 10000        | 500         | 1401.802             | 916.245            | 1.530 |
| random_u32         | 10000        | 2500        | 1030.778             | 836.049            | 1.233 |
| random_u32         | 10000        | 5000        | 982.850              | 781.576            | 1.258 |
| random_u32         | 100000       | 100         | 2301.354             | 979.460            | 2.350 |
| random_u32         | 100000       | 1000        | 2243.391             | 978.314            | 2.293 |
| random_u32         | 100000       | 5000        | 1891.002             | 953.899            | 1.982 |
| random_u32         | 100000       | 25000       | 1288.567             | 859.835            | 1.499 |
| random_u32         | 100000       | 50000       | 1172.934             | 820.194            | 1.430 |
| random_u32         | 1000000      | 1000        | 2546.980             | 988.249            | 2.577 |
| random_u32         | 1000000      | 10000       | 2447.939             | 998.272            | 2.452 |
| random_u32         | 1000000      | 50000       | 2072.423             | 964.052            | 2.150 |
| random_u32         | 1000000      | 250000      | 1394.686             | 877.832            | 1.589 |
| random_u32         | 1000000      | 500000      | 1254.738             | 830.534            | 1.511 |
| random_u32         | 10000000     | 10000       | 2368.294             | 957.068            | 2.475 |
| random_u32         | 10000000     | 100000      | 2369.680             | 946.966            | 2.502 |
| random_u32         | 10000000     | 500000      | 1698.173             | 951.751            | 1.784 |
| random_u32         | 10000000     | 2500000     | 1210.360             | 850.953            | 1.422 |
| random_u32         | 10000000     | 5000000     | 1224.683             | 809.430            | 1.513 |
| random_u32         | 100000000    | 100000      | 2477.097             | 942.398            | 2.629 |
| random_u32         | 100000000    | 1000000     | 2439.013             | 967.716            | 2.520 |
| random_u32         | 100000000    | 5000000     | 1701.137             | 921.458            | 1.846 |
| random_u32         | 100000000    | 25000000    | 1141.586             | 775.108            | 1.473 |
| random_u32         | 100000000    | 50000000    | 1169.007             | 736.957            | 1.586 |
| sawtooth_u32       | 1000         | 1           | 1423.316             | 1062.113           | 1.340 |
| sawtooth_u32       | 1000         | 10          | 1179.327             | 860.816            | 1.370 |
| sawtooth_u32       | 1000         | 50          | 1271.205             | 1000.403           | 1.271 |
| sawtooth_u32       | 1000         | 250         | 893.764              | 899.432            | 0.994 |
| sawtooth_u32       | 1000         | 500         | 859.591              | 935.239            | 0.919 |
| sawtooth_u32       | 10000        | 10          | 2025.779             | 1109.243           | 1.826 |
| sawtooth_u32       | 10000        | 100         | 2018.144             | 1106.968           | 1.823 |
| sawtooth_u32       | 10000        | 500         | 1319.054             | 962.093            | 1.371 |
| sawtooth_u32       | 10000        | 2500        | 1001.597             | 821.639            | 1.219 |
| sawtooth_u32       | 10000        | 5000        | 1021.174             | 755.976            | 1.351 |
| sawtooth_u32       | 100000       | 100         | 2536.898             | 1013.211           | 2.504 |
| sawtooth_u32       | 100000       | 1000        | 2328.530             | 959.432            | 2.427 |
| sawtooth_u32       | 100000       | 5000        | 2037.397             | 944.414            | 2.157 |
| sawtooth_u32       | 100000       | 25000       | 1258.644             | 778.844            | 1.616 |
| sawtooth_u32       | 100000       | 50000       | 1154.468             | 756.198            | 1.527 |
| sawtooth_u32       | 1000000      | 1000        | 2798.754             | 1042.740           | 2.684 |
| sawtooth_u32       | 1000000      | 10000       | 2738.584             | 1015.337           | 2.697 |
| sawtooth_u32       | 1000000      | 50000       | 2271.288             | 975.092            | 2.329 |
| sawtooth_u32       | 1000000      | 250000      | 1409.171             | 800.622            | 1.760 |
| sawtooth_u32       | 1000000      | 500000      | 1232.586             | 744.853            | 1.655 |
| sawtooth_u32       | 10000000     | 10000       | 2542.632             | 958.881            | 2.652 |
| sawtooth_u32       | 10000000     | 100000      | 2492.513             | 948.904            | 2.627 |
| sawtooth_u32       | 10000000     | 500000      | 1794.039             | 894.928            | 2.005 |
| sawtooth_u32       | 10000000     | 2500000     | 1273.305             | 793.681            | 1.604 |
| sawtooth_u32       | 10000000     | 5000000     | 1223.535             | 742.817            | 1.647 |
| sawtooth_u32       | 100000000    | 100000      | 2616.889             | 935.489            | 2.797 |
| sawtooth_u32       | 100000000    | 1000000     | 2613.222             | 908.900            | 2.875 |
| sawtooth_u32       | 100000000    | 5000000     | 1885.759             | 883.974            | 2.133 |
| sawtooth_u32       | 100000000    | 25000000    | 1390.023             | 766.984            | 1.812 |
| sawtooth_u32       | 100000000    | 50000000    | 1307.412             | 755.551            | 1.730 |
| reversed_u32       | 1000         | 1           | 1434.349             | 1031.170           | 1.391 |
| reversed_u32       | 1000         | 10          | 1275.116             | 873.262            | 1.460 |
| reversed_u32       | 1000         | 50          | 1350.558             | 891.378            | 1.515 |
| reversed_u32       | 1000         | 250         | 882.933              | 951.450            | 0.928 |
| reversed_u32       | 1000         | 500         | 611.380              | 885.383            | 0.691 |
| reversed_u32       | 10000        | 10          | 2346.806             | 1172.860           | 2.001 |
| reversed_u32       | 10000        | 100         | 2199.026             | 1090.039           | 2.017 |
| reversed_u32       | 10000        | 500         | 1116.235             | 1093.471           | 1.021 |
| reversed_u32       | 10000        | 2500        | 1105.590             | 1092.506           | 1.012 |
| reversed_u32       | 10000        | 5000        | 1083.022             | 1021.559           | 1.060 |
| reversed_u32       | 100000       | 100         | 2555.293             | 1024.697           | 2.494 |
| reversed_u32       | 100000       | 1000        | 2346.914             | 977.098            | 2.402 |
| reversed_u32       | 100000       | 5000        | 1977.279             | 940.639            | 2.102 |
| reversed_u32       | 100000       | 25000       | 1446.283             | 1017.813           | 1.421 |
| reversed_u32       | 100000       | 50000       | 1196.090             | 1065.621           | 1.122 |
| reversed_u32       | 1000000      | 1000        | 2722.210             | 1041.276           | 2.614 |
| reversed_u32       | 1000000      | 10000       | 2664.015             | 1059.188           | 2.515 |
| reversed_u32       | 1000000      | 50000       | 2265.912             | 1000.152           | 2.266 |
| reversed_u32       | 1000000      | 250000      | 1523.716             | 854.344            | 1.783 |
| reversed_u32       | 1000000      | 500000      | 1323.129             | 809.371            | 1.635 |
| reversed_u32       | 10000000     | 10000       | 2478.212             | 956.140            | 2.592 |
| reversed_u32       | 10000000     | 100000      | 2485.877             | 974.217            | 2.552 |
| reversed_u32       | 10000000     | 500000      | 1729.661             | 893.714            | 1.935 |
| reversed_u32       | 10000000     | 2500000     | 1256.296             | 769.175            | 1.633 |
| reversed_u32       | 10000000     | 5000000     | 1214.674             | 732.207            | 1.659 |
| reversed_u32       | 100000000    | 100000      | 1680.592             | 996.001            | 1.687 |
| reversed_u32       | 100000000    | 1000000     | 1843.183             | 999.808            | 1.844 |
| reversed_u32       | 100000000    | 5000000     | 1173.561             | 878.261            | 1.336 |
| reversed_u32       | 100000000    | 25000000    | 1204.927             | 750.040            | 1.606 |
| reversed_u32       | 100000000    | 50000000    | 1180.235             | 703.481            | 1.678 |
| randomdups_u32     | 1000         | 1           | 1143.251             | 769.554            | 1.486 |
| randomdups_u32     | 1000         | 10          | 1076.996             | 717.668            | 1.501 |
| randomdups_u32     | 1000         | 50          | 1046.788             | 747.960            | 1.400 |
| randomdups_u32     | 1000         | 250         | 715.282              | 615.136            | 1.163 |
| randomdups_u32     | 1000         | 500         | 665.364              | 582.618            | 1.142 |
| randomdups_u32     | 10000        | 10          | 1925.136             | 994.552            | 1.936 |
| randomdups_u32     | 10000        | 100         | 1809.058             | 946.221            | 1.912 |
| randomdups_u32     | 10000        | 500         | 1340.732             | 910.848            | 1.472 |
| randomdups_u32     | 10000        | 2500        | 997.560              | 815.569            | 1.223 |
| randomdups_u32     | 10000        | 5000        | 924.526              | 730.742            | 1.265 |
| randomdups_u32     | 100000       | 100         | 2268.583             | 960.618            | 2.362 |
| randomdups_u32     | 100000       | 1000        | 2201.015             | 946.886            | 2.324 |
| randomdups_u32     | 100000       | 5000        | 1851.647             | 905.940            | 2.044 |
| randomdups_u32     | 100000       | 25000       | 1251.383             | 822.409            | 1.522 |
| randomdups_u32     | 100000       | 50000       | 1120.948             | 776.657            | 1.443 |
| randomdups_u32     | 1000000      | 1000        | 2442.389             | 961.943            | 2.539 |
| randomdups_u32     | 1000000      | 10000       | 2342.619             | 939.414            | 2.494 |
| randomdups_u32     | 1000000      | 50000       | 1961.052             | 914.657            | 2.144 |
| randomdups_u32     | 1000000      | 250000      | 1346.704             | 857.790            | 1.570 |
| randomdups_u32     | 1000000      | 500000      | 1176.697             | 775.767            | 1.517 |
| randomdups_u32     | 10000000     | 10000       | 2253.944             | 902.931            | 2.496 |
| randomdups_u32     | 10000000     | 100000      | 2234.778             | 897.912            | 2.489 |
| randomdups_u32     | 10000000     | 500000      | 1573.779             | 887.196            | 1.774 |
| randomdups_u32     | 10000000     | 2500000     | 1177.778             | 825.531            | 1.427 |
| randomdups_u32     | 10000000     | 5000000     | 1151.573             | 760.355            | 1.515 |
| randomdups_u32     | 100000000    | 100000      | 2378.725             | 920.614            | 2.584 |
| randomdups_u32     | 100000000    | 1000000     | 2315.962             | 880.192            | 2.631 |
| randomdups_u32     | 100000000    | 5000000     | 1650.687             | 886.411            | 1.862 |
| randomdups_u32     | 100000000    | 25000000    | 1162.197             | 788.834            | 1.473 |
| randomdups_u32     | 100000000    | 50000000    | 1187.095             | 747.984            | 1.587 |
| random_bool        | 1000         | 1           | 598.908              | 427.956            | 1.399 |
| random_bool        | 1000         | 10          | 646.471              | 448.509            | 1.441 |
| random_bool        | 1000         | 50          | 653.588              | 455.183            | 1.436 |
| random_bool        | 1000         | 250         | 512.573              | 350.678            | 1.462 |
| random_bool        | 1000         | 500         | 539.461              | 457.690            | 1.179 |
| random_bool        | 10000        | 10          | 647.548              | 450.198            | 1.438 |
| random_bool        | 10000        | 100         | 643.050              | 448.643            | 1.433 |
| random_bool        | 10000        | 500         | 637.747              | 442.631            | 1.441 |
| random_bool        | 10000        | 2500        | 640.778              | 443.966            | 1.443 |
| random_bool        | 10000        | 5000        | 560.040              | 444.325            | 1.260 |
| random_bool        | 100000       | 100         | 669.655              | 456.237            | 1.468 |
| random_bool        | 100000       | 1000        | 681.474              | 458.303            | 1.487 |
| random_bool        | 100000       | 5000        | 673.033              | 457.230            | 1.472 |
| random_bool        | 100000       | 25000       | 697.258              | 466.606            | 1.494 |
| random_bool        | 100000       | 50000       | 576.662              | 450.416            | 1.280 |
| random_bool        | 1000000      | 1000        | 702.179              | 467.856            | 1.501 |
| random_bool        | 1000000      | 10000       | 695.655              | 450.506            | 1.544 |
| random_bool        | 1000000      | 50000       | 700.946              | 469.107            | 1.494 |
| random_bool        | 1000000      | 250000      | 705.093              | 463.051            | 1.523 |
| random_bool        | 1000000      | 500000      | 610.948              | 467.862            | 1.306 |
| random_bool        | 10000000     | 10000       | 698.470              | 484.653            | 1.441 |
| random_bool        | 10000000     | 100000      | 703.715              | 478.158            | 1.472 |
| random_bool        | 10000000     | 500000      | 710.139              | 484.194            | 1.467 |
| random_bool        | 10000000     | 2500000     | 701.528              | 457.072            | 1.535 |
| random_bool        | 10000000     | 5000000     | 619.682              | 436.499            | 1.420 |
| random_bool        | 100000000    | 100000      | 702.174              | 483.036            | 1.454 |
| random_bool        | 100000000    | 1000000     | 703.139              | 451.052            | 1.559 |
| random_bool        | 100000000    | 5000000     | 690.415              | 454.255            | 1.520 |
| random_bool        | 100000000    | 25000000    | 695.979              | 464.674            | 1.498 |
| random_bool        | 100000000    | 50000000    | 600.317              | 517.781            | 1.159 |

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