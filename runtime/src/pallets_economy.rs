use crate::{
    constants::UNITS,
    primitives::{AccountId, Balance},
    Balances, Event, Runtime, System, Treasury,
};
use frame_support::{
    parameter_types,
    traits::{Currency, OnUnbalanced},
    weights::IdentityFee,
};

parameter_types! {
    /// Same as Polkadot Relay Chain.
    pub const ExistentialDeposit: Balance = 1 * UNITS;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = Treasury;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1 ;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

/// Splits fees 20/80 between reserve and block author.
pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        Treasury::on_unbalanced(amount);
    }
}
