pub use crate::crypto::ed25519::verify_signature;

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::fixed_bytes;

    #[test]
    fn test_sui_ed25519() {
        let address =
            fixed_bytes!("a1938c88ee613aa15b0ae05c1036519052fd469ae0f221ee64e477ac52ecc4cb");
        let message = b"hey";
        let signature = fixed_bytes!("42a69fe6e00d23b805cc9520a69c8cd56fc21b4f736bebffdda3abdcd97c3101020b09f1d0514e0a6d2773000c0db788bcf18db7d8074a1010b4cbf568289e0b");

        assert_eq!(
            verify_signature(&address.as_slice(), message, &signature.as_slice()).unwrap(),
            ()
        );
    }
}
