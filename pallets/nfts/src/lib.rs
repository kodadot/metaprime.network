#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::NFT;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::StaticLookup, DispatchResult};
use sp_std::vec::Vec;

pub use pallet::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData {}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData {}

pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + orml_nft::Config<ClassData = ClassData, TokenData = TokenData>
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// ClassId not found
        ClassIdNotFound,
        /// The operator is not the owner of the token and has no permission
        NoPermission,
        /// Quantity is invalid. need >= 1
        InvalidQuantity,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Created NFT class. \[owner, class_id\]
        CreatedClass(T::AccountId, ClassIdOf<T>),
        /// Minted NFT token. \[from, to, class_id, quantity\]
        MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, u32),
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create NFT class, tokens belong to the class.
        ///
        /// - `metadata`: external metadata
        #[pallet::weight(1_000)]
        pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;
            let next_id = orml_nft::Pallet::<T>::next_class_id();

            orml_nft::Pallet::<T>::create_class(&owner, metadata, ClassData::default())?;

            Self::deposit_event(Event::CreatedClass(owner, next_id));
            Ok(().into())
        }

        /// Mint NFT token
        ///
        /// - `to`: the token owner's account
        /// - `class_id`: token belong to the class id
        /// - `metadata`: external metadata
        /// - `quantity`: token quantity
        #[pallet::weight(1_000)]
        #[transactional]
        pub fn mint(
            origin: OriginFor<T>,
            to: <T::Lookup as StaticLookup>::Source,
            class_id: ClassIdOf<T>,
            metadata: Vec<u8>,
            quantity: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
            let class_info =
                orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
            ensure!(who == class_info.owner, Error::<T>::NoPermission);

            for _ in 0..quantity {
                orml_nft::Pallet::<T>::mint(&to, class_id, metadata.clone(), TokenData::default())?;
            }

            Self::deposit_event(Event::MintedToken(who, to, class_id, quantity));
            Ok(().into())
        }

        /// Transfer NFT token to another account
        ///
        /// - `to`: the token owner's account
        /// - `token`: (class_id, token_id)
        #[pallet::weight(1_000)]
        #[transactional]
        pub fn transfer(
            origin: OriginFor<T>,
            to: <T::Lookup as StaticLookup>::Source,
            token: (ClassIdOf<T>, TokenIdOf<T>),
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            <Pallet<T> as NFT<T::AccountId>>::transfer(&who, &to, token)?;
            Ok(().into())
        }
    }
}

impl<T: Config> NFT<T::AccountId> for Pallet<T> {
    type ClassId = ClassIdOf<T>;
    type TokenId = TokenIdOf<T>;
    type Balance = u128;

    fn balance(who: &T::AccountId) -> Self::Balance {
        orml_nft::TokensByOwner::<T>::iter_prefix(who).count() as u128
    }

    fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<T::AccountId> {
        orml_nft::Pallet::<T>::tokens(token.0, token.1).map(|t| t.owner)
    }

    fn transfer(
        from: &T::AccountId,
        to: &T::AccountId,
        token: (Self::ClassId, Self::TokenId),
    ) -> DispatchResult {
        orml_nft::Pallet::<T>::transfer(from, to, token)
    }
}
