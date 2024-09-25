#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, Hash};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod registration {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
    }

    // Storage for subnets
    #[pallet::storage]
    #[pallet::getter(fn subnets)]
    pub type Subnets<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Subnet<T>>;

    // Event definitions
    #[pallet::event]
    #[pallet::generate_store(pub(super) trait Store)]
    pub enum Event<T: Config> {
        SubnetCreated(T::AccountId, T::Hash),
        SubnetValidated(T::Hash),
        QueenAdded(T::Hash, T::AccountId),
        ProviderAdded(T::Hash, T::AccountId),
        DrillMechanismSpecified(T::Hash, DrillMechanism), // Event for specifying drill mechanism
    }

    // Define the drill mechanism structure
    #[derive(Clone, Encode, Decode, Default, RuntimeDebug, PartialEq)]
    pub struct DrillMechanism {
        pub test_name: Vec<u8>,          // Name of the drill test (e.g., MLPerf)
        pub test_type: DrillTestType,    // Type of test (Training, Inference, etc.)
        pub performance_metrics: Vec<Metric>, // Performance metrics to evaluate (accuracy, speed, etc.)
    }

    #[derive(Clone, Encode, Decode, RuntimeDebug, PartialEq)]
    pub enum DrillTestType {
        Training,
        Inference,
        Custom(Vec<u8>), // Allow for custom test types defined by the King
    }

    #[derive(Clone, Encode, Decode, Default, RuntimeDebug, PartialEq)]
    pub struct Metric {
        pub name: Vec<u8>,  // Name of the metric (e.g., Accuracy, Latency)
        pub target_value: f64, // Target value that must be met or exceeded
    }

    // Structure for the Subnet
    #[derive(Clone, Encode, Decode, Default, RuntimeDebug, PartialEq)]
    pub struct Subnet<T: Config> {
        pub king: T::AccountId,
        pub vision: Vec<u8>,
        pub mission: Vec<u8>,
        pub queens: Vec<T::AccountId>,
        pub providers: Vec<T::AccountId>,
        pub drill_mechanism: DrillMechanism,
        pub active: bool, // Track if the subnet is active
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_subnet(
            origin: OriginFor<T>,
            vision: Vec<u8>,
            mission: Vec<u8>,
            drill_mechanism: DrillMechanism, // Updated to accept DrillMechanism
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check if the King has staked enough GPoints
            let stake = T::Currency::free_balance(&who);
            ensure!(stake >= 100_000, "Insufficient stake to become a King");

            // Create a unique identifier for the subnet
            let subnet_id = T::Hash::from(<T::Hash as Hash>::hash(&who.encode()));

            // Ensure the subnet does not already exist
            ensure!(!Subnets::<T>::contains_key(&subnet_id), "Subnet already exists");

            // Create new subnet with specified drill mechanism
            let subnet = Subnet {
                king: who.clone(),
                vision,
                mission,
                queens: Vec::new(),
                providers: Vec::new(),
                drill_mechanism: drill_mechanism.clone(),
                active: true,
            };

            // Store the subnet
            Subnets::<T>::insert(subnet_id, subnet);

            // Emit events
            Self::deposit_event(Event::SubnetCreated(who, subnet_id));
            Self::deposit_event(Event::DrillMechanismSpecified(subnet_id, drill_mechanism));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn add_queen(origin: OriginFor<T>, subnet_id: T::Hash, queen: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure subnet exists
            let mut subnet = Subnets::<T>::get(subnet_id).ok_or(Error::<T>::SubnetNotFound)?;

            // Ensure the caller is the King of the subnet
            ensure!(subnet.king == who, "Only the King can add a Queen");

            // Add Queen to the subnet
            subnet.queens.push(queen.clone());
            Subnets::<T>::insert(subnet_id, subnet);

            // Emit event
            Self::deposit_event(Event::QueenAdded(subnet_id, queen));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn add_provider(origin: OriginFor<T>, subnet_id: T::Hash, provider: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure subnet exists
            let mut subnet = Subnets::<T>::get(subnet_id).ok_or(Error::<T>::SubnetNotFound)?;

            // Ensure the caller is the King of the subnet
            ensure!(subnet.king == who, "Only the King can add a Provider");

            // Add Provider to the subnet
            subnet.providers.push(provider.clone());
            Subnets::<T>::insert(subnet_id, subnet);

            // Emit event
            Self::deposit_event(Event::ProviderAdded(subnet_id, provider));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn validate_subnet(origin: OriginFor<T>, subnet_id: T::Hash) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure the subnet exists
            let mut subnet = Subnets::<T>::get(subnet_id).ok_or(Error::<T>::SubnetNotFound)?;

            // Ensure the caller is authorized (e.g., the King of the subnet or an authorized Queen)
            ensure!(subnet.king == who || subnet.queens.contains(&who), "Unauthorized");

            // Execute drill tests on Providers
            let mut validation_results = Vec::new();
            for provider in &subnet.providers {
                // Execute the drill test specified in the drill mechanism
                let result = Self::execute_drill_test(&subnet.drill_mechanism, provider);
                validation_results.push((provider.clone(), result));
            }

            // Aggregate results and determine subnet validity
            let valid = Self::aggregate_results(&validation_results, &subnet.drill_mechanism);

            // Update subnet validation status
            subnet.active = valid;
            Subnets::<T>::insert(subnet_id, subnet);

            // Emit validation event
            Self::deposit_event(Event::SubnetValidated(subnet_id));
            Ok(())
        }
    }

    // Helper function to execute a drill test on a provider
    impl<T: Config> Pallet<T> {
        fn execute_drill_test(mechanism: &DrillMechanism, provider: &T::AccountId) -> DrillTestResult {
            // Simulated drill test execution logic
            // In a real implementation, this would involve actual compute tasks
            DrillTestResult {
                success: true, // Placeholder value; real results should be based on actual test execution
                metrics: vec![
                    ("Accuracy".to_string(), 95.0), // Example metric result
                    ("Latency".to_string(), 100.0), // Example metric result
                ],
            }
        }

        // Helper function to aggregate results and determine overall subnet validity
        fn aggregate_results(
            results: &Vec<(T::AccountId, DrillTestResult)>,
            mechanism: &DrillMechanism,
        ) -> bool {
            // Check each result against the drill mechanism's performance metrics
            let mut total_pass = 0;
            for (_provider, result) in results {
                if result.success {
                    total_pass += 1;
                }
            }

            // Determine if enough providers passed to consider the subnet valid
            let pass_rate = (total_pass as f64) / (results.len() as f64);
            pass_rate >= 0.75 // Example threshold: 75% of providers must pass
        }
    }

    // Structure for drill test results
    #[derive(Clone, Encode, Decode, Default, RuntimeDebug, PartialEq)]
    pub struct DrillTestResult {
        pub success: bool,           // Whether the test was passed
        pub metrics: Vec<(String, f64)>, // Performance metrics from the drill test
    }

    #[pallet::error]
    pub enum Error<T> {
        SubnetNotFound,
        Unauthorized,
        // Other error variants...
    }
}
