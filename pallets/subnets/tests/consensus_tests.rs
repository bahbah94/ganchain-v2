// subnet/tests/consensus_tests.rs

use crate::consensus::{Pallet as ConsensusPallet, DrillTestResult};
use frame_support::{assert_ok, assert_err, traits::Currency};
use sp_core::H256;
use sp_runtime::AccountId32;

#[test]
fn test_submit_drill_test_success() {
    new_test_ext().execute_with(|| {
        let provider: AccountId32 = AccountId32::new([1u8; 32]);
        let task_id: u64 = 1;
        let mock_proof: Vec<u8> = vec![1, 2, 3, 4];
        let mock_inputs: Vec<u8> = vec![1, 2, 3];

        assert_ok!(ConsensusPallet::submit_drill_test(
            frame_system::RawOrigin::Signed(provider.clone()).into(),
            task_id,
            mock_proof.clone(),
            mock_inputs.clone()
        ));

        let stored_result = ConsensusPallet::drill_test_results(&provider).unwrap();
        assert_eq!(stored_result.task_id, task_id);
        assert_eq!(stored_result.proof, mock_proof);
        assert_eq!(stored_result.public_inputs, mock_inputs);
    });
}

#[test]
fn test_proof_verification_success() {
    new_test_ext().execute_with(|| {
        let provider: AccountId32 = AccountId32::new([1u8; 32]);
        let task_id: u64 = 1;
        let valid_proof: Vec<u8> = vec![1, 2, 3, 4];
        let public_inputs: Vec<u8> = vec![1, 2, 3];

        assert_ok!(ConsensusPallet::submit_drill_test(
            frame_system::RawOrigin::Signed(provider.clone()).into(),
            task_id,
            valid_proof.clone(),
            public_inputs.clone()
        ));

        let validation_id = H256::random();
        let is_verified = ConsensusPallet::verify_proof(valid_proof, public_inputs);
        assert!(is_verified);

        assert_ok!(ConsensusPallet::finalize_validation(validation_id, true, provider.clone()));

        let reward = ConsensusPallet::validation_reward();
        let balance = frame_support::traits::Currency::<AccountId32>::free_balance(&provider);
        assert_eq!(balance, reward);
    });
}

#[test]
fn test_proof_verification_failure() {
    new_test_ext().execute_with(|| {
        let provider: AccountId32 = AccountId32::new([1u8; 32]);
        let task_id: u64 = 1;
        let invalid_proof: Vec<u8> = vec![4, 3, 2, 1];
        let public_inputs: Vec<u8> = vec![1, 2, 3];

        assert_ok!(ConsensusPallet::submit_drill_test(
            frame_system::RawOrigin::Signed(provider.clone()).into(),
            task_id,
            invalid_proof.clone(),
            public_inputs.clone()
        ));

        let validation_id = H256::random();
        let is_verified = ConsensusPallet::verify_proof(invalid_proof, public_inputs);
        assert!(!is_verified);

        assert_ok!(ConsensusPallet::finalize_validation(validation_id, false, provider.clone()));

        let penalty = ConsensusPallet::penalty_amount();
        let balance = frame_support::traits::Currency::<AccountId32>::free_balance(&provider);
        assert_eq!(balance, 0);
    });
}

fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<crate::mock::Test>()
        .unwrap();
    t.into()
}
