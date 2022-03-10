use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::time::Instant;
use sync_multi_sig::{MultiSig, SMSigScheme, HEIGHT};

const NUM_REPETITIONS: usize = 10;

fn main() {
    smsig();
    smsig_agg();
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
    for _ in 0..NUM_REPETITIONS {
        let _ = SMSigScheme::key_gen(&seed, &pp);
    }
    println!(
        "ken gen time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
    let (pk, sk) = SMSigScheme::key_gen(&seed, &pp);

    // ===============================
    // sign
    // ===============================
    let start = Instant::now();
    for _ in 0..NUM_REPETITIONS {
        let _ = SMSigScheme::sign(&sk, 0, message.as_ref(), &pp);
    }

    println!(
        "signing time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
    let sig = SMSigScheme::sign(&sk, 0, message.as_ref(), &pp);

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
}

fn smsig_agg() {
    let message = "this is the message to sign";
    let mut seed = [0u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let pp = SMSigScheme::setup(&mut rng);

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
    let start = Instant::now();
    for _ in 0..NUM_REPETITIONS {
        SMSigScheme::aggregate(&sigs, &pks);
    }
    println!(
        "aggregating time {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
    let agg_sig = SMSigScheme::aggregate(&sigs, &pks);
    // ===============================
    // batch verification
    // ===============================
    let start = Instant::now();
    for _ in 0..NUM_REPETITIONS {
        assert!(SMSigScheme::batch_verify(
            &pks,
            message.as_ref(),
            &agg_sig,
            &pp
        ));
    }
    println!(
        "batch verification {}",
        start.elapsed().as_nanos() / NUM_REPETITIONS as u128
    );
}
