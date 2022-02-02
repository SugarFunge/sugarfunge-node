//! RPC interface for sugarfunge-asset.

pub use self::gen_client::Client as SugarfungeAssetClient;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, MaybeDisplay},
};
use std::sync::Arc;
pub use sugarfunge_asset_rpc_runtime_api::SugarfungeAssetApi as SugarfungeAssetRuntimeApi;

#[rpc]
pub trait SugarfungeAssetApi<BlockHash, AccountId, ClassId, AssetId, Balance> {
    #[rpc(name = "asset_balancesOfOwner")]
    fn balances_of_owner(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<(ClassId, AssetId, Balance)>>;
}

/// A struct that implements the [`SugarfungeAssetApi`].
pub struct SugarfungeAsset<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> SugarfungeAsset<C, P> {
    /// Create new `SugarfungeAsset` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, Block, AccountId, ClassId, AssetId, Balance>
    SugarfungeAssetApi<<Block as BlockT>::Hash, AccountId, ClassId, AssetId, Balance>
    for SugarfungeAsset<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: SugarfungeAssetRuntimeApi<Block, AccountId, ClassId, AssetId, Balance>,
    AccountId: Codec + MaybeDisplay + Clone,
    ClassId: Codec + MaybeDisplay + Copy,
    AssetId: Codec + MaybeDisplay + Copy,
    Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + Default,
{
    fn balances_of_owner(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<(ClassId, AssetId, Balance)>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.balances_of_owner(&at, account);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
