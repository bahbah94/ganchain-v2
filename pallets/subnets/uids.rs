#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Hash;
use sp_std::collections::btree_set::BTreeSet;

#[frame_support::pallet]
pub mod uids {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    // Storage for managing UIDs
    #[pallet::storage]
    #[pallet::getter(fn get_uid)]
    pub type Uids<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    #[pallet::storage]
    #[pallet::getter(fn get_last_uid)]
    pub type LastUid<T> = StorageValue<_, u64, ValueQuery>;

    // A set of active UIDs to avoid reuse and ensure uniqueness
    #[pallet::storage]
    #[pallet::getter(fn active_uids)]
    pub type ActiveUids<T> = StorageValue<_, BTreeSet<u64>, ValueQuery>;

    // Event definitions
    #[pallet::event]
    #[pallet::generate_store(pub(super) trait Store)]
    pub enum Event<T: Config> {
        UidGenerated(T::AccountId, u64),
        UidRevoked(T::AccountId, u64),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn generate_uid(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Generate a new UID
            let new_uid = Self::generate_unique_uid()?;

            // Store the new UID for the account
            Uids::<T>::insert(&who, new_uid);
            ActiveUids::<T>::mutate(|uids| uids.insert(new_uid));

            // Emit event
            Self::deposit_event(Event::UidGenerated(who, new_uid));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn revoke_uid(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if the account has a UID
            let uid = Uids::<T>::get(&who).ok_or(Error::<T>::UidNotFound)?;

            // Remove the UID from storage
            Uids::<T>::remove(&who);
            ActiveUids::<T>::mutate(|uids| uids.remove(&uid));

            // Emit event
            Self::deposit_event(Event::UidRevoked(who, uid));
            Ok(())
        }
    }

    // Helper functions for UID management
    impl<T: Config> Pallet<T> {
        // Generate a new unique UID
        fn generate_unique_uid() -> Result<u64, DispatchError> {
            // Increment the last UID value
            let last_uid = LastUid::<T>::get();
            let new_uid = last_uid.checked_add(1).ok_or(Error::<T>::UidOverflow)?;

            // Ensure the new UID is unique and not currently active
            ensure!(!ActiveUids::<T>::get().contains(&new_uid), Error::<T>::UidAlreadyExists);

            // Update the last UID value
            LastUid::<T>::put(new_uid);
            Ok(new_uid)
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        UidNotFound,
        UidAlreadyExists,
        UidOverflow,
        // Other error variants...
    }
}
