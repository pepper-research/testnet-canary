use sov_modules_api::Address;
use sov_modules_api::Spec;

/// Slot time in milliseconds
pub const SLOT_TIME: u64 = 5;
/// Sequencer authority address
// pub const SEQUENCER_AUTHORITY: Address<S: Spec> = Address::new([0; 32]); // TODO: update

pub const fn get_sequencer_authority<S: Spec>() -> Address<S> {
    Address::new([0; 32])
}
