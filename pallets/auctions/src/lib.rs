#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    storage::types::ValueQuery,
    traits::{Currency, ReservableCurrency},
    transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{Auction, AuctionHandler, Change, OnNewBidResult};
use sp_runtime::{traits::AccountIdConversion, DispatchResult};

pub use pallet::*;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + orml_auction::Config<Balance = BalanceOf<Self>> + orml_nft::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The pallet in charge of currency transfers
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The pallet's module id - used to keep NFTs which are on auction
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// This can only be done by the owner of the nft
        NotOwnerOfNft,
        /// Nft transfer has failed for an unknown reason
        FailedNftTransfer,
        /// Failed to register auction
        AuctionNotRegistered,
    }

    /// Store auction info (owner, start price, class id, token id).
    ///
    /// Returns the default value if auction info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn auctions)]
    pub type Auctions<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AuctionId,
        (T::AccountId, BalanceOf<T>, T::ClassId, T::TokenId),
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Listed a NFT on sale. \[auction id, class_id, token id, start price\]
        ListedNFT(T::AuctionId, T::ClassId, T::TokenId, BalanceOf<T>),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// List a NFT for auction.
        /// - `nft` should be the class and token ids of the nft
        /// - `start_price` is the minimum price for any bid
        /// - `start` is when the auction starts
        /// - `end` is when the auction ends
        #[pallet::weight(1_000)]
        #[transactional]
        fn list(
            origin: OriginFor<T>,
            nft: (T::ClassId, T::TokenId),
            start_price: BalanceOf<T>,
            start: T::BlockNumber,
            end: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                orml_nft::Pallet::<T>::is_owner(&who, nft),
                Error::<T>::NotOwnerOfNft
            );
            ensure!(
                orml_nft::Pallet::<T>::transfer(&who, &T::PalletId::get().into_account(), nft)
                    .is_ok(),
                Error::<T>::FailedNftTransfer
            );

            let auction_id = orml_auction::Pallet::<T>::new_auction(start, Some(end));
            ensure!(auction_id.is_ok(), Error::<T>::AuctionNotRegistered);

            Auctions::<T>::insert(
                auction_id.expect("we did a is_ok check; qed"),
                (who, start_price, nft.0, nft.1),
            );

            Self::deposit_event(Event::ListedNFT(
                auction_id.expect("we did a is_ok check; qed"),
                nft.0,
                nft.1,
                start_price,
            ));
            Ok(())
        }
    }
}

impl<T: Config> AuctionHandler<T::AccountId, BalanceOf<T>, T::BlockNumber, T::AuctionId>
    for Pallet<T>
{
    fn on_new_bid(
        _now: T::BlockNumber,
        id: T::AuctionId,
        new_bid: (T::AccountId, BalanceOf<T>),
        last_bid: Option<(T::AccountId, BalanceOf<T>)>,
    ) -> OnNewBidResult<T::BlockNumber> {
        let minimum_bid = Auctions::<T>::get(id).1;
        if new_bid.1 < minimum_bid {
            return OnNewBidResult {
                accept_bid: false,
                auction_end_change: Change::NoChange,
            };
        }

        if T::Currency::reserve(&new_bid.0, new_bid.1).is_err() {
            // failed to reserve new coins
            return OnNewBidResult {
                accept_bid: false,
                auction_end_change: Change::NoChange,
            };
        }
        if let Some(bid) = last_bid {
            T::Currency::unreserve(&bid.0, bid.1);
        }

        OnNewBidResult {
            accept_bid: false,
            auction_end_change: Change::NoChange,
        }
    }

    fn on_auction_ended(id: T::AuctionId, winner: Option<(T::AccountId, BalanceOf<T>)>) {
        let meta = Auctions::<T>::take(id);
        let dest = match winner.clone() {
            Some((acc, _)) => acc,
            None => meta.clone().0,
        };

        if let Some(details) = winner {
            T::Currency::unreserve(&details.0, details.1);
            T::Currency::transfer(
                &details.0,
                &meta.0,
                details.1,
                frame_support::traits::ExistenceRequirement::AllowDeath,
            );
        }

        orml_nft::Pallet::<T>::transfer(
            &T::PalletId::get().into_account(),
            &dest,
            (meta.2, meta.3),
        );
    }
}
