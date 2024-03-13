use ecies::encrypt;
use ecies::utils::generate_keypair;
use methods::{MTCS_CHECK_ELF, MTCS_CHECK_ID};
use mtcs_core::SimpleSetOff;
use risc0_zkvm::{default_prover, ExecutorEnv};

fn main() {
    let (sk, pk) = generate_keypair();
    let (sk, pk) = (&sk.serialize(), &pk.serialize());

    let setoffs = vec![
        SimpleSetOff {
            id: None,
            debtor: 1,
            creditor: 2,
            amount: 100,
            set_off: 70,
            remainder: 30,
        },
        SimpleSetOff {
            id: None,
            debtor: 2,
            creditor: 3,
            amount: 100,
            set_off: 70,
            remainder: 30,
        },
        SimpleSetOff {
            id: None,
            debtor: 3,
            creditor: 1,
            amount: 70,
            set_off: 70,
            remainder: 0,
        },
    ]
    .into_iter()
    .map(|so| {
        let so = serde_json::to_string(&so).unwrap();
        let cipher = encrypt(pk, so.as_bytes()).unwrap();
        hex::encode(&cipher)
    })
    .collect::<Vec<String>>();

    let hex_sk: &str = &hex::encode(&sk.to_vec());

    // First, we construct an executor environment
    let env = ExecutorEnv::builder()
        .write(&hex_sk)
        .unwrap()
        .write(&setoffs)
        .unwrap()
        .build()
        .unwrap();

    let now = std::time::Instant::now();

    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove(env, MTCS_CHECK_ELF).unwrap();

    let proof_time = now.elapsed();
    println!("Proof generation time: {:?}", proof_time);

    // TODO: Implement code for transmitting or serializing the receipt for
    // other parties to verify here

    // Optional: Verify receipt to confirm that recipients will also be able to
    // verify your receipt
    receipt.verify(MTCS_CHECK_ID).unwrap();
    println!("Verification time: {:?}", now.elapsed() - proof_time);
}

#[cfg(test)]
mod tests {
    #[test]
    fn keygen() {
        use ecies::{decrypt, encrypt, utils::generate_keypair};

        const MSG: &str = "helloworld";
        let (sk, pk) = generate_keypair();
        let (sk, pk) = (&sk.serialize(), &pk.serialize());

        let msg = MSG.as_bytes();
        assert_eq!(
            msg,
            decrypt(sk, &encrypt(pk, msg).unwrap()).unwrap().as_slice()
        );
    }
}
