// subnet/tests/uids_tests.rs

use crate::uids::{Pallet as UidsPallet};
use frame_support::{assert_ok, assert_err};
use sp_runtime::AccountId32;

#[test]
fn test_generate_uid() {
    new_test_ext().execute_with(|| {
        let user: AccountId32 = AccountId32::new([1u8; 32]);

        assert_ok!(UidsPallet::generate_uid(frame_system::RawOrigin::Signed(user.clone()).into()));

        let uid = UidsPallet::get_uid(&user).unwrap();
        assert!(uid > 0); // Ensure UID is non-zero
    });
}

#[test]
fn test_uid_uniqueness() {
    new_test_ext().execute_with(|| {
        let user1: AccountId32 = AccountId32::new([1u8; 32]);
        let user2: AccountId32 = AccountId32::new([2u8; 32]);

        assert_ok!(UidsPallet::generate_uid(frame_system::RawOrigin::Signed(user1.clone()).into()));
        assert_ok!(UidsPallet::generate_uid(frame_system::RawOrigin::Signed(user2.clone()).into()));

        let uid1 = UidsPallet::get_uid(&user1).unwrap();
        let uid2 = UidsPallet::get_uid(&user2).unwrap();

        assert_ne!(uid1, uid2); // Ensure UIDs are unique
    });
}

fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<crate::mock::Test>()
        .unwrap();
    t.into()
}
