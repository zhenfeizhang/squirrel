use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::time::Instant;
use sync_multi_sig::{MultiSig, SMSigScheme, HEIGHT};

const NUM_REPETITIONS: usize = 1;

fn main() {
    smsig();
}

fn smsig() {
    let message = "this is the message to sign";
    let mut seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let pp = SMSigScheme::setup(&mut rng);

    println!("tree height: {}", HEIGHT);

    // ===============================
    // key gen
    // ===============================
    rng.fill_bytes(&mut seed);

    let start = Instant::now();
    // for _ in 0..NUM_REPETITIONS {
    //     let _ = SMSigScheme::key_gen(&seed, &pp);
    // }

    let (pk, sk) = SMSigScheme::key_gen(&seed, &pp);
    println!(
        "ken gen time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );

    // ===============================
    // sign
    // ===============================
    let start = Instant::now();
    // for _ in 0..NUM_REPETITIONS {
    //     let _ = SMSigScheme::sign(&sk, 0, message.as_ref(), &pp);
    // }
    let sig = SMSigScheme::sign(&sk, 0, message.as_ref(), &pp);
    println!(
        "signing time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
    

    // ===============================
    // verify
    // ===============================
    let start = Instant::now();
    for _ in 0..NUM_REPETITIONS {
        assert!(SMSigScheme::verify(&pk, message.as_ref(), &sig, &pp));
    }
    println!(
        "verification time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );

    let mut sigs = Vec::new();
    let mut pks = Vec::new();
    for _ in 0..4096 {
        pks.push(pk);
        sigs.push(sig.clone());
    }
    // ===============================
    // aggregation
    // ===============================
    let start = Instant::now();
    // for _ in 0..NUM_REPETITIONS {
    //     SMSigScheme::aggregate(&sigs, &pks);
    // }

    let agg_sig = SMSigScheme::aggregate(&sigs, &pks);
    println!(
        "aggregating time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
    // ===============================
    // batch verification
    // ===============================
    let start = Instant::now();
    for _ in 0..NUM_REPETITIONS {
        SMSigScheme::batch_verify(
            &pks,
            message.as_ref(),
            &agg_sig,
            &pp
        );
    }
    println!(
        "batch verification {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
}
