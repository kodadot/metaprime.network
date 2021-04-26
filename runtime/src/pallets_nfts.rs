use crate::{primitives::Balance, AuctionManager, Balances, Event, Runtime};
use frame_support::{parameter_types, PalletId};

parameter_types! {
    pub const AuctionPalletId: PalletId = PalletId(*b"kod/auct");
}

impl kodadot_auction::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type PalletId = AuctionPalletId;
}

impl orml_auction::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type AuctionId = u32;
    type Handler = AuctionManager;
    type WeightInfo = ();
}

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
