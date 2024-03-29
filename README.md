Squirrel: Efficient Synchronized Multi-Signatures from Lattices
------

This is a reference implementation for the paper: [Squirrel: Efficient Synchronized Multi-Signatures from Lattices](https://eprint.iacr.org/2022/694).

_NEWS_: this work is superseded by __Chipmunk: Better Synchronized Multi-Signatures from Lattices__: [paper](https://eprint.iacr.org/2023/1820) and [GitHub](https://github.com/GottfriedHerold/Chipmunk).

# Benchmark
```
cargo bench [--features=parallel]
```

![](bench.png)

# Citation

```bibtex
@misc{cryptoeprint:2022/694,
      author = {Nils Fleischhacker and Mark Simkin and Zhenfei Zhang},
      title = {Squirrel: Efficient Synchronized Multi-Signatures from Lattices},
      howpublished = {ACM CCS}
      year = {2022},
      url = {https://eprint.iacr.org/2022/694}
}
```
