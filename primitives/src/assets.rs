use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{RuntimeDebug, H160};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetId {
    Native,
    ETH,
    BTC,
    DOT,
    TOKEN(H160),
}
