//! Runtime API definition for sugarfunge-asset.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    pub trait SugarfungeAssetApi<AccountId, ClassId, AssetId, Balance> where
    AccountId: Codec + MaybeDisplay,
    ClassId: Codec + MaybeDisplay,
    AssetId: Codec + MaybeDisplay,
    Balance: Codec + MaybeDisplay,
    {
        fn balances_of_owner(account: AccountId) -> Vec<(ClassId, AssetId, Balance)>;
    }
}
