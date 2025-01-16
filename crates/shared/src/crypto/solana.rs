pub use crate::crypto::ed25519::verify_signature;

#[cfg(test)]
mod test {
    use super::*;
    use base58::FromBase58;

    #[test]
    fn test_solana() {
        let address = "CyrR839U5L1a4yRtnHwVDxkQcv7w93ecDLohv1SPebDv"
            .from_base58()
            .expect("could not parse public key");
        let message: &[u8; 3] = b"hey";
        let signature = "5J6Q44KuXUZndcTVPopEF9pdgrmHeQQ1NG3pHSvutYNVP2upisRUeNDqbQeiDcv2LwiP45xXCG6fogrm5WakzLki"
            .from_base58()
            .expect("could not parse signature");

        assert_eq!(verify_signature(&address, message, &signature).unwrap(), ());
    }
}
