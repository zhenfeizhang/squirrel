Synchronized multi-signature scheme from lattice
------

This is a reference implementation for the paper: [Synchronized multi-signature scheme from lattice](tbd).

# Benchmark
```
cargo bench [--features=parallel]
```

| Tree height | KeyGen | Sign | Verification | Aggregate 100 signatures | Batch verify 100 signatures |
| ---:| ---:| ---:| ---:| ---:| ---:|
| 5  | 2.2 ms | 0.5 ms| 0.5 ms | 20 ms | 1.3 ms |
| 10 | 61 ms | 0.6 ms| 1.1 ms | 35 ms | 1.7 ms| 
| 15 | 1.9 s | 0.6 ms| 1.5 ms | 42 ms | 2.2 ms |
| 17 | 7.8 s | 0.6 ms| 1.6 ms | 58 ms | 2.1 ms |
| 19 | 31 s | 0.6 ms | 1.9 ms | 60 ms | 2.3 ms |
| 21 | 124 s | 0.6 ms | 1.7 ms | 66 ms | 2.4 ms |