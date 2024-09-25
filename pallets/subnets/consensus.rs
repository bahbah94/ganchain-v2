#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Hash, Zero};
use sp_std::collections::btree_map::BTreeMap;
use sp_io::hashing::blake2_128;

#[frame_support::pallet]
pub mod enhanced_consensus {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
        type ValidationReward: Get<BalanceOf<Self>>;
        type PenaltyAmount: Get<BalanceOf<Self>>;
        type ValidationWindow: Get<u64>; // Time window for Validators to vote
    }

    #[pallet::storage]
    #[pallet::getter(fn drill_test_results)]
    pub type DrillTestResults<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, DrillTestResult>;

    #[pallet::storage]
    #[pallet::getter(fn validation_votes)]
    pub type ValidationVotes<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, ValidationVote<T>>;

    // Storage for tracking Validator performance reputation scores
    #[pallet::storage]
    #[pallet::getter(fn validator_reputation)]
    pub type ValidatorReputation<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32>;

    #[pallet::event]
    #[pallet::generate_store(pub(super) trait Store)]
    pub enum Event<T: Config> {
        DrillTestSubmitted(T::AccountId, u64),
        ValidationVoteCast(T::Hash, T::AccountId, bool),
        DrillTestValidated(T::Hash, bool),
        RewardDistributed(T::AccountId, BalanceOf<T>),
        ValidatorPenalized(T::AccountId, BalanceOf<T>),
    }

    #[derive(Clone, Encode, Decode, Default, RuntimeDebug, PartialEq)]
    pub struct DrillTestResult {
        pub task_id: u64,
        pub success: bool,
        pub performance_metrics: Vec<(String, f64)>,
    }

    #[derive(Clone, Encode, Decode, Default, RuntimeDebug, PartialEq)]
    pub struct ValidationVote<T: Config> {
        pub validators: BTreeMap<T::AccountId, bool>, // Validators' votes on the test result
        pub approval_count: u32,
        pub rejection_count: u32,
        pub final_decision: Option<bool>, // Final decision on the drill test result
        pub time_started: u64, // Timestamp when validation started
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn submit_drill_test(
            origin: OriginFor<T>,
            task_id: u64,
            performance_metrics: Vec<(String, f64)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let result = DrillTestResult {
                task_id,
                success: true, // Placeholder; validation will determine actual success
                performance_metrics,
            };

            DrillTestResults::<T>::insert(&who, result.clone());
            let validation_id = T::Hash::from(<T::Hash as Hash>::hash(&who.encode()));

            // Initiate validation voting with a randomized set of Validators
            let validators = Self::select_random_validators();
            let vote = ValidationVote::<T> {
                validators,
                approval_count: 0,
                rejection_count: 0,
                final_decision: None,
                time_started: <frame_system::Pallet<T>>::block_number().saturated_into::<u64>(),
            };

            ValidationVotes::<T>::insert(validation_id, vote);
            Self::deposit_event(Event::DrillTestSubmitted(who, task_id));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn cast_validation_vote(
            origin: OriginFor<T>,
            validation_id: T::Hash,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure the validation task exists
            let mut vote = ValidationVotes::<T>::get(validation_id).ok_or(Error::<T>::ValidationNotFound)?;

            // Check if the voting window is still open
            let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
            ensure!(
                current_block - vote.time_started <= T::ValidationWindow::get(),
                "Validation window has closed"
            );

            // Record the vote and adjust Validator reputation based on accuracy
            vote.validators.insert(who.clone(), approve);
            if approve {
                vote.approval_count += 1;
                Self::adjust_reputation(&who, true);
            } else {
                vote.rejection_count += 1;
                Self::adjust_reputation(&who, false);
            }

            // Check if a decision can be reached
            Self::check_validation_status(&mut vote, validation_id);

            ValidationVotes::<T>::insert(validation_id, vote);
            Self::deposit_event(Event::ValidationVoteCast(validation_id, who, approve));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn check_validation_status(vote: &mut ValidationVote<T>, validation_id: T::Hash) {
            let total_votes = vote.approval_count + vote.rejection_count;
            let supermajority_threshold = (total_votes * 2) / 3;

            if vote.approval_count > supermajority_threshold {
                vote.final_decision = Some(true);
                Self::finalize_validation(validation_id, true);
            } else if vote.rejection_count > supermajority_threshold {
                vote.final_decision = Some(false);
                Self::finalize_validation(validation_id, false);
            }
        }

        fn finalize_validation(validation_id: T::Hash, approved: bool) {
            if approved {
                // Reward Validators
                for (validator, _) in ValidationVotes::<T>::get(validation_id).unwrap().validators {
                    let reward = T::ValidationReward::get();
                    T::Currency::deposit_creating(&validator, reward);
                    Self::deposit_event(Event::RewardDistributed(validator, reward));
                }
            } else {
                // Penalize Validators who voted incorrectly
                for (validator, vote) in ValidationVotes::<T>::get(validation_id).unwrap().validators {
                    if !vote {
                        let penalty = T::PenaltyAmount::get();
                        T::Currency::withdraw(
                            &validator,
                            penalty,
                            WithdrawReasons::RESERVE,
                            ExistenceRequirement::KeepAlive,
                        )
                        .ok();
                        Self::deposit_event(Event::ValidatorPenalized(validator, penalty));
                    }
                }
            }

            // Finalize and log the validation decision
            Self::deposit_event(Event::DrillTestValidated(validation_id, approved));
        }

        // Random selection of Validators
        fn select_random_validators() -> BTreeMap<T::AccountId, bool> {
            let mut validators = BTreeMap::new();
            // Random selection logic using VRF or other randomness source
            // Example implementation, needs to be replaced with actual randomness logic
            validators.insert(<T as frame_system::Config>::AccountId::default(), false);
            validators
        }

        fn adjust_reputation(validator: &T::AccountId, accurate: bool) {
            let rep = ValidatorReputation::<T>::get(validator).unwrap_or(0);
            let new_rep = if accurate { rep + 1 } else { rep.saturating_sub(1) };
            ValidatorReputation::<T>::insert(validator, new_rep);
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidationNotFound,
        // Other error variants...
    }
}
