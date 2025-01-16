use anyhow::bail;
use sov_modules_api::{Spec, StateAccessor};

use crate::{state::wallet::WalletType, Capsule};

impl<S: Spec> Capsule<S> {
    pub fn check_duplicate_wallet(
        &self,
        wallet_type: &WalletType,
        state: &mut impl StateAccessor,
    ) -> bool {
        match self.wallets.get(&wallet_type, state).unwrap() {
            Some(wallet) => true,
            None => false,
        }
    }

    pub fn throw_duplicate_wallet_error(
        &self,
        wallet_type: &WalletType,
        state: &mut impl StateAccessor,
    ) -> anyhow::Result<()> {
        if self.check_duplicate_wallet(wallet_type, state) {
            bail!("Wallet type {:?} already exists", wallet_type);
        }

        Ok(())
    }
}
