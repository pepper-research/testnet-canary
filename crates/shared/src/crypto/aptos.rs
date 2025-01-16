pub use crate::crypto::ed25519::verify_signature;

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::fixed_bytes;

    #[test]
    fn test_aptos() {
        let address =
            fixed_bytes!("ca1d16d870c0a272c699ec768d9252207f5a03fd2741c6f4e7a51f31113fa17d");
        let message = b"hey";
        let signature = fixed_bytes!("4b6b4687283684e1598b6d28cc5d721fcbfe1cadfb7eb247d0ed918c22744c598d4b0e15f2263fe508ba013cba91bfe9fffab0723d7f243eeed9774da7502601");

        assert_eq!(
            verify_signature(&address.as_slice(), message, &signature.as_slice()).unwrap(),
            ()
        );
    }
}
