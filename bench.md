Parameters and benchmark results
----

# rho = 1024; rho = 21

- q_hvc = 61441
- q_hots = 6694913
- alpha = 20
- beta_s = 44
- gamma = 41

```
hash/1000 hvc_hash digests                        
                        time:   [68.457 ms 68.745 ms 69.054 ms]

hash/1000 hots_hash digests                        
                        time:   [110.57 ms 111.20 ms 111.94 ms]

poly/1000 binary poly mul by ternary poly                        
                        time:   [1.4965 ms 1.4990 ms 1.5014 ms]

hvc poly/1000 of ntt transform                        
                        time:   [4.0885 ms 4.0925 ms 4.0970 ms]

hvc poly/1000 of inv_ntt transform                        
                        time:   [4.3778 ms 4.3859 ms 4.3957 ms]

hvc poly/1000 of ntt additions                        
                        time:   [76.922 us 77.239 us 77.595 us]
                        
hvc poly/1000 of ntt multiplications                        
                        time:   [197.29 us 197.59 us 197.93 us]

hots poly/1000 of ntt transform                        
                        time:   [5.8247 ms 5.8308 ms 5.8370 ms]
                        
hots poly/1000 of inv_ntt transform                        
                        time:   [4.2450 ms 4.2516 ms 4.2597 ms]

hots poly/1000 of ntt additions                        
                        time:   [216.68 us 217.32 us 218.03 us]

hots poly/1000 of ntt multiplications                        
                        time:   [507.94 us 508.44 us 509.04 us]

randomization/decomposing 1000 paths                        
                        time:   [376.54 ms 377.80 ms 378.95 ms]

randomization/randomize 1000 paths                        
                        time:   [265.16 ms 273.67 ms 283.65 ms]

randomization/generate 1000 randomizers                        
                        time:   [1.8035 ms 1.8101 ms 1.8176 ms]

randomization/aggregate 1000 paths                        
                        time:   [676.34 ms 679.63 ms 684.12 ms]

randomization/verify aggregated paths                        
                        time:   [19.378 ms 20.050 ms 20.393 ms]
```

ken gen time 219 sec
signing time 2.1 ms
verification time 3.2 ms
aggregating time 1000 signature 1.2 sec
batch verification 1000 signature 19.5 ms
