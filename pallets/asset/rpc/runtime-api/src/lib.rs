//! Runtime API definition for sugarfunge-asset.

#![cfg_attr(not(feature = "std"), no_std)]

// use codec::{Codec, Decode, Encode};
// use frame_support::dispatch::DispatchError;
use sp_runtime::traits::MaybeDisplay;
use sp_std::prelude::*;




use codec::{Codec, Decode, Encode};
// use frame_support::dispatch::DispatchError;
// #[cfg(feature = "std")]
// // use serde::{Deserialize, Deserializer, Serialize, Serializer};
// use sp_runtime::serde::{Deserialize, Deserializer, Serialize, Serializer};

// #[derive(Eq, PartialEq, Encode, Decode, Default)]
// #[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
// #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
// /// a wrapper around a balance, used in RPC to workaround a bug where using u128
// /// in runtime-apis fails. See <https://github.com/paritytech/substrate/issues/4641>
// pub struct BalanceWrapper<T> {
//     #[cfg_attr(feature = "std", serde(bound(serialize = "T: std::fmt::Display")))]
//     #[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
//     #[cfg_attr(feature = "std", serde(bound(deserialize = "T: std::str::FromStr")))]
//     #[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
//     pub amount: T,
// }

// #[cfg(feature = "std")]
// fn serialize_as_string<S: Serializer, T: std::fmt::Display>(t: &T, serializer: S) -> Result<S::Ok, S::Error> {
//     serializer.serialize_str(&t.to_string())
// }

// #[cfg(feature = "std")]
// fn deserialize_from_string<'de, D: Deserializer<'de>, T: std::str::FromStr>(deserializer: D) -> Result<T, D::Error> {
//     let s = String::deserialize(deserializer)?;
//     s.parse::<T>()
//         .map_err(|_| serde::de::Error::custom("Parse from string failed"))
// }


sp_api::decl_runtime_apis! {
    pub trait SugarfungeAssetApi<AccountId, ClassId, AssetId, Balance> where
    AccountId: Codec + Decode + Encode + MaybeDisplay,
    ClassId:   Codec + Decode + Encode + MaybeDisplay,
    AssetId:   Codec + Decode + Encode + MaybeDisplay,
    Balance:   Codec + Decode + Encode + MaybeDisplay,
    {
        fn balances_of_owner(account: AccountId) -> Vec<(ClassId, AssetId, Balance)>;
    }
}
