use super::*;

// This function creates tokens of a given asset and deposits them into an address. If the recipient address doesn't exist, it is created.
impl<T: Trait> Module<T> {
    pub fn mint(to_address: H256, asset_id: u32, amount: i64, name: u32) -> DispatchResult {
        // Checking that amount is non-negative.
        if amount < 0 {
             Err("Amount can't be negative.")?
        }

        // Increasing supply.
        if <Self as Store>::TotalSupply::exists(asset_id) {
            let new_supply = <Self as Store>::TotalSupply::get(asset_id) + amount;
            <Self as Store>::TotalSupply::insert(asset_id, new_supply);
        } else {
            <Self as Store>::TotalSupply::insert(asset_id, amount);
            <Self as Store>::Tokens::insert(name, amount);
        }

        // Crediting amount to to_address.
        if <Self as Store>::Balances::exists((asset_id, to_address)) {
            let new_balance = <Self as Store>::Balances::get((asset_id, to_address)) + amount;
            <Self as Store>::Balances::insert((asset_id, to_address), new_balance);
        } else {
            <Self as Store>::Balances::insert((asset_id, to_address), amount);
        }

        // Return Ok.
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use primitives::H256;
//     // The testing primitives are very useful for avoiding having to work with signatures
//     // or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
//     use sp_runtime::{
//         testing::Header,
//         traits::{BlakeTwo256, IdentityLookup},
//         Perbill,
//     };
//     use support::{assert_noop, assert_ok, impl_outer_origin, parameter_types};

//     impl_outer_origin! {
//         pub enum Origin for Test {}
//     }

//     // For testing the module, we construct most of a mock runtime. This means
//     // first constructing a configuration type (`Test`) which `impl`s each of the
//     // configuration traits of modules we want to use.
//     #[derive(Clone, Eq, PartialEq)]
//     pub struct Test;
//     parameter_types! {
//         pub const BlockHashCount: u64 = 250;
//         pub const MaximumBlockWeight: u32 = 1024;
//         pub const MaximumBlockLength: u32 = 2 * 1024;
//         pub const AvailableBlockRatio: Perbill = Perbill::one();
//     }
//     impl system::Trait for Test {
//         type Origin = Origin;
//         type Index = u64;
//         type Call = ();
//         type BlockNumber = u64;
//         type Hash = H256;
//         type Hashing = BlakeTwo256;
//         type AccountId = u64;
//         type Lookup = IdentityLookup<Self::AccountId>;
//         type Header = Header;
//         type Event = ();
//         type BlockHashCount = BlockHashCount;
//         type MaximumBlockWeight = MaximumBlockWeight;
//         type AvailableBlockRatio = AvailableBlockRatio;
//         type MaximumBlockLength = MaximumBlockLength;
//         type Version = ();
//     }
//     impl Trait for Test {
//         // If Events are ever added to this module, then the next line
//         // needs to be commented out.
//         // type Event = ();
//     }
//     // This next line should have the name of the module, in this
//     // case it is Assets
//     type Assets = Module<Test>;

//     // This function basically just builds a genesis storage key/value store according to
//     // our desired mockup.
//     fn new_test_ext() -> runtime_io::TestExternalities {
//         system::GenesisConfig::default()
//             .build_storage::<Test>()
//             .unwrap()
//             .into()
//     }

//     #[test]
//     fn mint_works() {
//         new_test_ext().execute_with(|| {
//             // Initialize some values.
//             let supply = 999000;
//             let to_address = H256::random();
//             let to_balance = 200;
//             let asset_id = 1;

//             // Manually store addresses with balances.
//             <Assets as Store>::TotalSupply::insert(asset_id, supply);
//             <Assets as Store>::Balances::insert((asset_id, to_address), to_balance);

//             // Test case of negative transfer amount.
//             let mut amount = -100;
//             assert!(Assets::mint(to_address, asset_id, amount).is_err());

//             // Test normal case.
//             amount = 1000;
//             assert!(Assets::mint(to_address, asset_id, amount).is_ok());
//             assert_eq!(
//                 supply + amount,
//                 <Assets as Store>::TotalSupply::get(asset_id)
//             );
//             assert_eq!(
//                 to_balance + amount,
//                 <Assets as Store>::Balances::get((asset_id, to_address))
//             );
//         });
//     }
// }