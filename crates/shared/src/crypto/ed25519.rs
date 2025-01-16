use anyhow::{anyhow, Result};
use ed25519_dalek::{ed25519::SignatureBytes, Signer, SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH};

pub fn get_public_key(keypair_bytes: &[u8]) -> Result<[u8; PUBLIC_KEY_LENGTH]> {
    let keypair_slice = keypair_bytes.try_into()?;
    SigningKey::from_keypair_bytes(keypair_slice)
        .map(|key| key.verifying_key().to_bytes())
        .map_err(|e| anyhow!(e).context("could not parse keypair"))
}

pub fn verify_signature(pubkey_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> Result<()> {
    let publickey = VerifyingKey::from_bytes(&pubkey_bytes.try_into()?)
        .map_err(|e| anyhow!(e).context("could not parse public key"))?;
    let signature = signature_bytes
        .try_into()
        .map_err(|e: ed25519_dalek::ed25519::Error| {
            anyhow!(e).context("could not parse signature")
        })?;

    publickey
        .verify_strict(message, &signature)
        .map_err(|e| anyhow!(e).context("could not verify signature"))
}

pub fn sign_message(keypair_bytes: &[u8], message: &[u8]) -> Result<SignatureBytes> {
    let keypair_slice = keypair_bytes.try_into()?;
    SigningKey::from_keypair_bytes(keypair_slice)
        .map_err(|e| anyhow!(e).context("could not parse keypair"))?
        .try_sign(message)
        .map(|signature| signature.to_bytes())
        .map_err(|e| anyhow!(e).context("could not sign message"))
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::fixed_bytes;

    #[test]
    fn test_signing_ed25519() -> Result<()> {
        let keypair = fixed_bytes!("9227a3817dd9ba732cfea45508fe8f85f2cbcebd5da0f657e2a97c01156eed0e9577af0f37005b5b8102e6104e63331addd469e3684b91b42a5152aecebcda74");
        let message = b"hey";

        let signature = sign_message(keypair.as_slice(), message)?;
        let actual_signature: SignatureBytes = fixed_bytes!("74ffe827f833b957fb95c20e58e1ba9f96330ea66e17fec6298535a26f387607ee84d0bae7d086444e73b51cc21ded4c373f87a05fd2c69e94cfbdb3064e5804").into();
        assert_eq!(signature, actual_signature);

        let public_key = get_public_key(keypair.as_slice())?;
        let actual_public_key: [u8; 32] =
            fixed_bytes!("9577af0f37005b5b8102e6104e63331addd469e3684b91b42a5152aecebcda74").into();
        assert_eq!(public_key, actual_public_key);

        verify_signature(&public_key, message, &signature)
    }
}
