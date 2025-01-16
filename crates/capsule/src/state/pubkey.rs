// // This is a truncated version of https://github.com/solana-labs/solana/blob/master/sdk/program/src/pubkey.rs lice

// use std::{convert::Infallible, fmt, mem, str::FromStr};

// use num_derive::{FromPrimitive, ToPrimitive};
// use serde::Serialize;
// use thiserror::Error;

// const MAX_BASE58_LEN: usize = 44;

// #[cfg_attr(feature = "native", derive(serde::Serialize, serde::Deserialize))]
// #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Clone)]
// pub struct Pubkey(pub(crate) [u8; 32]);

// #[derive(Error, Debug, Serialize, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
// pub enum ParsePubkeyError {
//     #[error("String is the wrong size")]
//     WrongSize,
//     #[error("Invalid Base58 string")]
//     Invalid,
// }

// impl From<Infallible> for ParsePubkeyError {
//     fn from(_: Infallible) -> Self {
//         unreachable!("Infallible uninhabited");
//     }
// }

// impl FromStr for Pubkey {
//     type Err = ParsePubkeyError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if s.len() > MAX_BASE58_LEN {
//             return Err(ParsePubkeyError::WrongSize);
//         }
//         let pubkey_vec = bs58::decode(s)
//             .into_vec()
//             .map_err(|_| ParsePubkeyError::Invalid)?;
//         if pubkey_vec.len() != mem::size_of::<Pubkey>() {
//             Err(ParsePubkeyError::WrongSize)
//         } else {
//             Pubkey::try_from(pubkey_vec).map_err(|_| ParsePubkeyError::Invalid)
//         }
//     }
// }

// impl From<[u8; 32]> for Pubkey {
//     #[inline]
//     fn from(from: [u8; 32]) -> Self {
//         Self(from)
//     }
// }

// impl TryFrom<&[u8]> for Pubkey {
//     type Error = std::array::TryFromSliceError;

//     #[inline]
//     fn try_from(pubkey: &[u8]) -> Result<Self, Self::Error> {
//         <[u8; 32]>::try_from(pubkey).map(Self::from)
//     }
// }

// impl TryFrom<Vec<u8>> for Pubkey {
//     type Error = Vec<u8>;

//     #[inline]
//     fn try_from(pubkey: Vec<u8>) -> Result<Self, Self::Error> {
//         <[u8; 32]>::try_from(pubkey).map(Self::from)
//     }
// }

// impl TryFrom<&str> for Pubkey {
//     type Error = ParsePubkeyError;
//     fn try_from(s: &str) -> Result<Self, Self::Error> {
//         Pubkey::from_str(s)
//     }
// }

// impl AsRef<[u8]> for Pubkey {
//     fn as_ref(&self) -> &[u8] {
//         &self.0[..]
//     }
// }

// impl AsMut<[u8]> for Pubkey {
//     fn as_mut(&mut self) -> &mut [u8] {
//         &mut self.0[..]
//     }
// }

// impl fmt::Debug for Pubkey {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", bs58::encode(self.0).into_string())
//     }
// }

// impl fmt::Display for Pubkey {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", bs58::encode(self.0).into_string())
//     }
// }
