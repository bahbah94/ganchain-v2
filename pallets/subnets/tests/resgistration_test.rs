// subnet/tests/registration_tests.rs

use crate::registration::{Pallet as RegistrationPallet};
use frame_support::{assert_ok, assert_err};
use sp_runtime::AccountId32;

#[test]
fn test_register_node() {
    new_test_ext().execute_with(|| {
        let node_owner: AccountId32 = AccountId32::new([1u8; 32]);
        let node_type = NodeType::Provider;

        assert_ok!(RegistrationPallet::register_node(
            frame_system::RawOrigin::Signed(node_owner.clone()).into(),
            node_type.clone()
        ));

        // Verify that the node is registered with the correct UID
        let node_info = RegistrationPallet::nodes(&node_owner).unwrap();
        assert_eq!(node_info.node_type, node_type);
        assert!(node_info.uid.is_some());
    });
}

#[test]
fn test_register_node_uid_uniqueness() {
    new_test_ext().execute_with(|| {
        let node_owner_1: AccountId32 = AccountId32::new([1u8; 32]);
        let node_owner_2: AccountId32 = AccountId32::new([2u8; 32]);
        let node_type = NodeType::Provider;

        assert_ok!(RegistrationPallet::register_node(
            frame_system::RawOrigin::Signed(node_owner_1.clone()).into(),
            node_type.clone()
        ));
        assert_ok!(RegistrationPallet::register_node(
            frame_system::RawOrigin::Signed(node_owner_2.clone()).into(),
            node_type.clone()
        ));

        let uid_1 = RegistrationPallet::nodes(&node_owner_1).unwrap().uid.unwrap();
        let uid_2 = RegistrationPallet::nodes(&node_owner_2).unwrap().uid.unwrap();

        assert_ne!(uid_1, uid_2);
    });
}

fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<crate::mock::Test>()
        .unwrap();
    t.into()
}
