use alloy_primitives::{Address, Signature};
use anyhow::{anyhow, bail, Result};

pub fn verify_signature(
    address_bytes: &[u8],
    message: &[u8],
    signature_bytes: &[u8],
) -> Result<()> {
    let signature = Signature::try_from(signature_bytes)
        .map_err(|e| anyhow!(e).context("could not parse signature"))?;

    let recovered_address = signature
        .recover_address_from_msg(&message)
        .map_err(|e| anyhow!(e).context("could not recover public key from signature"))?;
    if recovered_address != address_bytes {
        let actual_address = Address::from_slice(address_bytes);
        bail!("public key mismatch: expected {actual_address}, got {recovered_address}");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::fixed_bytes;

    #[test]
    fn test_ethereum() {
        let address = fixed_bytes!("22e2862a76eb7030d49521e9e9d6179f8ff4e4b5");
        let message = b"hey";
        let signature = fixed_bytes!("205b79f9a71b6788b4d8ba2e87c27bd87e7d78cee51cbedb2ba4a24d2cf8d0f1765cd8e3be1f918f8ede87c906f88b5b03e602f7725b72a963a66a9e7796f6f61c");

        assert_eq!(
            verify_signature(address.as_slice(), message, signature.as_slice()).unwrap(),
            ()
        );
    }
}
