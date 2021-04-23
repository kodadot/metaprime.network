use crate::{Event, Runtime};
use frame_support::{parameter_types, PalletId};

parameter_types! {
    pub const NftPalletId: PalletId = PalletId(*b"kod/nfts");
}

impl kodadot_nft::Config for Runtime {
    type Event = Event;
    type PalletId = NftPalletId;
}

impl orml_nft::Config for Runtime {
    type ClassId = u32;
    type TokenId = u64;
    type ClassData = kodadot_nft::ClassData;
    type TokenData = kodadot_nft::TokenData;
}
