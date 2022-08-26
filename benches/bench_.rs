#[macro_use]
extern crate criterion;

use criterion::Criterion;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sync_multi_sig::{
    HOTSHash, HVCHash, LargeNTTPoly, LargePoly, MultiSig, Path, RandomizedPath, Randomizers,
    SMSigScheme, SignedPoly, SmallNTTPoly, SmallPoly, TerPolyCoeffEncoding, Tree, ALPHA, HEIGHT,
    SMALL_MODULUS_BITS, LARGE_MODULUS_BITS,
};

criterion_main!(bench);
criterion_group!(
    bench,
    // bench_smsig,
    // bench_smsig_agg,
    bench_hash,
    bench_ter_poly,
    bench_hvc_ntt,
    bench_hots_ntt,
    bench_randomization,
    bench_tree,
    bench_decompose,
);

fn bench_ter_poly(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let weight = 10;
    let num_tests = 1000;
    let bin_polys: Vec<SignedPoly> = (0..num_tests)
        .map(|_| SignedPoly::rand_binary(&mut rng))
        .collect();
    let ter_polys: Vec<TerPolyCoeffEncoding> = (0..num_tests)
        .map(|_| (&SignedPoly::rand_ternary(&mut rng, weight)).into())
        .collect();

    let mut bench_group = c.benchmark_group("poly");
    bench_group.sample_size(100);
    let bench_str = format!("{} binary poly mul by ternary poly", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = SignedPoly::ter_mul_bin(&ter_polys[i], &bin_polys[i]);
            }
        });
    });
}

fn bench_hvc_ntt(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let num_tests = 1000;
    let polys: Vec<SmallPoly> = (0..num_tests)
        .map(|_| SmallPoly::rand_poly(&mut rng))
        .collect();
    let another_polys: Vec<SmallPoly> = (0..num_tests)
        .map(|_| SmallPoly::rand_poly(&mut rng))
        .collect();
    let poly_ntts: Vec<SmallNTTPoly> = polys.iter().map(|x| x.into()).collect();
    let another_poly_ntts: Vec<SmallNTTPoly> = another_polys.iter().map(|x| x.into()).collect();

    let mut bench_group = c.benchmark_group("hvc poly");
    bench_group.sample_size(100);
    let bench_str = format!("{} of ntt transform", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _: SmallNTTPoly = (&polys[i]).into();
            }
        });
    });

    let poly_ntts_clone = poly_ntts.clone();
    let bench_str = format!("{} of inv_ntt transform", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _: SmallPoly = (&poly_ntts_clone[i]).into();
            }
        });
    });

    let poly_ntts_clone = poly_ntts.clone();
    let another_poly_ntts_clone = another_poly_ntts.clone();
    let bench_str = format!("{} of ntt additions", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = poly_ntts_clone[i].clone() + another_poly_ntts_clone[i].clone();
            }
        });
    });

    let bench_str = format!("{} of ntt multiplications", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = poly_ntts[i].clone() * another_poly_ntts[i].clone();
            }
        });
    });
}

fn bench_hots_ntt(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let num_tests = 1000;
    let polys: Vec<LargePoly> = (0..num_tests)
        .map(|_| LargePoly::rand_poly(&mut rng))
        .collect();
    let another_polys: Vec<LargePoly> = (0..num_tests)
        .map(|_| LargePoly::rand_poly(&mut rng))
        .collect();
    let poly_ntts: Vec<LargeNTTPoly> = polys.iter().map(|x| x.into()).collect();
    let another_poly_ntts: Vec<LargeNTTPoly> = another_polys.iter().map(|x| x.into()).collect();

    let mut bench_group = c.benchmark_group("hots poly");
    bench_group.sample_size(100);
    let bench_str = format!("{} of ntt transform", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _: LargeNTTPoly = (&polys[i]).into();
            }
        });
    });

    let poly_ntts_clone = poly_ntts.clone();
    let bench_str = format!("{} of inv_ntt transform", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _: LargePoly = (&poly_ntts_clone[i]).into();
            }
        });
    });

    let poly_ntts_clone = poly_ntts.clone();
    let another_poly_ntts_clone = another_poly_ntts.clone();
    let bench_str = format!("{} of ntt additions", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = poly_ntts_clone[i].clone() + another_poly_ntts_clone[i].clone();
            }
        });
    });

    let bench_str = format!("{} of ntt multiplications", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = poly_ntts[i].clone() * another_poly_ntts[i].clone();
            }
        });
    });
}

fn bench_hash(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let num_tests = 1000;
    let hasher = HVCHash::init(&mut rng);
    let inputs: Vec<Vec<SmallPoly>> = (0..num_tests)
        .map(|_| (0..SMALL_MODULUS_BITS<<1).map(|_| SmallPoly::rand_poly(&mut rng)).collect())
        .collect();

    let mut bench_group = c.benchmark_group("hash");
    bench_group.sample_size(100);
    let bench_str = format!("{} hvc_hash digests", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = hasher.hash(&inputs[i]);
            }
        });
    });

    let hasher = HOTSHash::init(&mut rng);
    let inputs: Vec<Vec<SignedPoly>> = (0..num_tests)
        .map(|_| (0..LARGE_MODULUS_BITS<<1).map(|_| SignedPoly::rand_binary(&mut rng)).collect())
        .collect();
    let bench_str = format!("{} hots_hash digests", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = hasher.hash(&inputs[i]);
            }
        });
    });
}

fn bench_decompose(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let num_tests = 1000;

    let inputs: Vec<SmallPoly> = (0..num_tests)
        .map(|_| SmallPoly::rand_poly(&mut rng))
        .collect();

    let decomposed: Vec<[SignedPoly; SMALL_MODULUS_BITS]> =
        inputs.iter().map(|x| x.decompose()).collect();

    let mut bench_group = c.benchmark_group("decompose");
    bench_group.sample_size(100);
    let bench_str = format!("{} decompositions", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = inputs[i].decompose();
            }
        });
    });

    let bench_str = format!("{} projections", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = SmallPoly::projection(&decomposed[i]);
            }
        });
    });
}

fn bench_tree(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let num_tests = 1;
    let hasher = HVCHash::init(&mut rng);
    let leaves: Vec<Vec<SmallPoly>> = (0..num_tests)
        .map(|_| {
            (0..(1 << (HEIGHT - 1)))
                .map(|_| SmallPoly::rand_poly(&mut rng))
                .collect()
        })
        .collect();
    let trees: Vec<Tree> = leaves
        .iter()
        .map(|x| Tree::new_with_leaf_nodes(x, &hasher))
        .collect();

    let proofs: Vec<Path> = trees.iter().map(|x| x.gen_proof(0)).collect();

    let hasher_clone = hasher.clone();
    let mut bench_group = c.benchmark_group("tree");
    bench_group.sample_size(10);

    let bench_str = format!("build {} trees of height {}", num_tests, HEIGHT);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = Tree::new_with_leaf_nodes(&leaves[i], &hasher_clone);
            }
        });
    });

    let tree_clone = trees.clone();
    let bench_str = format!("generate {} proofs", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = tree_clone[i].gen_proof(0);
            }
        });
    });

    let bench_str = format!("verify {} proofs", num_tests);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                assert!(proofs[i].verify(&trees[i].root(), &hasher))
            }
        });
    });
}

fn bench_randomization(c: &mut Criterion) {
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let num_tests = 1000;
    let hasher = HVCHash::init(&mut rng);
    let mut proofs = Vec::new();
    let mut roots = Vec::new();
    for _ in 0..num_tests {
        let (proof, root) = Path::random_for_testing(&mut rng, &hasher);
        roots.push(root);
        proofs.push(proof)
    }

    let mut decomposed_proofs: Vec<RandomizedPath> = proofs.iter().map(|x| x.into()).collect();
    let randomizers: Vec<SignedPoly> = (0..num_tests)
        .map(|_| SignedPoly::rand_ternary(&mut rng, ALPHA >> 1))
        .collect();

    let agg_path = Path::aggregation(&proofs, &roots);

    let mut bench_group = c.benchmark_group("randomization");
    bench_group.sample_size(10);

    let bench_str = format!("decomposing {} paths", num_tests);
    let proofs_clone = proofs.clone();
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                let _ = RandomizedPath::from(&proofs_clone[i]);
            }
        });
    });

    let bench_str = format!("randomize {} paths", num_tests);
    let decomposed_proofs_clone = decomposed_proofs.clone();
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            for i in 0..num_tests {
                decomposed_proofs[i].randomize_with(&randomizers[i]);
                decomposed_proofs[i] = decomposed_proofs_clone[i].clone();
            }
        });
    });

    let bench_str = format!("generate {} randomizers", num_tests);
    let roots_clone = roots.clone();
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            let _ = Randomizers::from_pks(&roots_clone);
        });
    });

    let bench_str = format!("aggregate {} paths", num_tests);
    let roots_clone = roots.clone();
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            let _ = Path::aggregation(&proofs, &roots_clone);
        });
    });

    let bench_str = format!("verify aggregated paths");
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| assert!(agg_path.verify(&roots, &hasher)));
    });
}
#[allow(dead_code)]
fn bench_smsig(c: &mut Criterion) {
    let message = "this is the message to sign";
    let mut seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let pp = SMSigScheme::setup(&mut rng);

    let mut bench_group = c.benchmark_group("smsig");
    bench_group.sample_size(10);

    // ===============================
    // key gen
    // ===============================
    let bench_str = format!("key gen");
    let pp_clone = pp.clone();
    rng.fill_bytes(&mut seed);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| SMSigScheme::key_gen(&seed, &pp_clone));
    });
    let (pk, sk) = SMSigScheme::key_gen(&seed, &pp);

    // ===============================
    // sign
    // ===============================
    let bench_str = format!("Sign");
    let pp_clone = pp.clone();

    let sig = SMSigScheme::sign(&sk, 0, message.as_ref(), &pp);
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| SMSigScheme::sign(&sk, 0, message.as_ref(), &pp_clone));
    });
    // ===============================
    // verify
    // ===============================
    let bench_str = format!("verify");
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| assert!(SMSigScheme::verify(&pk, message.as_ref(), &sig, &pp)));
    });
}

#[allow(dead_code)]
fn bench_smsig_agg(c: &mut Criterion) {
    let message = "this is the message to sign";
    let mut seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let pp = SMSigScheme::setup(&mut rng);

    let mut bench_group = c.benchmark_group("smsig aggregation");
    bench_group.sample_size(10);

    let mut sigs = Vec::new();
    let mut pks = Vec::new();
    for _ in 0..100 {
        rng.fill_bytes(&mut seed);
        let (pk, sk) = SMSigScheme::key_gen(&seed, &pp);

        let sig = SMSigScheme::sign(&sk, 0, message.as_ref(), &pp);
        assert!(SMSigScheme::verify(&pk, message.as_ref(), &sig, &pp));
        pks.push(pk);
        sigs.push(sig);
    }
    // ===============================
    // aggregation
    // ===============================
    let bench_str = format!("aggregation");
    let agg_sig = SMSigScheme::aggregate(&sigs, &pks);
    let pks_clone = pks.clone();
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| SMSigScheme::aggregate(&sigs, &pks_clone));
    });
    // ===============================
    // batch verification
    // ===============================
    let bench_str = format!("batch verification");
    bench_group.bench_function(bench_str, move |b| {
        b.iter(|| {
            assert!(SMSigScheme::batch_verify(
                &pks,
                message.as_ref(),
                &agg_sig,
                &pp
            ))
        });
    });
}
