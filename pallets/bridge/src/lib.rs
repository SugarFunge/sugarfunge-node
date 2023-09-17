// SBP-M1 review: not used by runtime, review is not exhaustive
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    traits::{EnsureOrigin, Get},
    BoundedVec, PalletId,
};
use frame_system::ensure_root;
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, Dispatchable, Hash},
    RuntimeDebug,
};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// SBP-M1 review: consider reducing visibility
pub type ChainId = u8;
pub type DepositNonce = u64;
pub type ResourceId = [u8; 32];

/// Helper function to concatenate a chain ID and some bytes to produce a resource ID.
/// The common format is (31 bytes unique ID + 1 byte chain ID).
// SBP-M1 review: reduce visibility
pub fn derive_resource_id(chain: u8, id: &[u8]) -> ResourceId {
    let mut r_id: ResourceId = [0; 32];
    r_id[31] = chain; // last byte is chain id
    let range = if id.len() > 31 { 31 } else { id.len() }; // Use at most 31 bytes
    for i in 0..range {
        // SBP-M1 review: indexing may panic
        // SBP-M1 review: use safe math
        r_id[30 - i] = id[range - 1 - i]; // Ensure left padding for eth compatibility
    }
    // SBP-M1 review: unneeded return statement
    return r_id;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalStatus {
    Initiated,
    Approved,
    Rejected,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ProposalVotes<BlockNumber, MaxVotesOf> {
    // SBP-M1 review: change to BoundedVec<AccountId, MaxVotes> to improve clarity
    pub votes_for: MaxVotesOf,
    pub votes_against: MaxVotesOf,
    pub status: ProposalStatus,
    pub expiry: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    // SBP-M1 review: sugarfunge_asset not used, remove
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // SBP-M1 review: missing doc comment
        type PalletId: Get<PalletId>;

        /// RuntimeOrigin used to administer the pallet
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        // SBP-M1 review: add line break for consistency
        /// Proposed dispatchable call
        type Proposal: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + EncodeLike;

        /// The identifier for this chain.
        /// This must be unique and must not collide with existing IDs within a set of bridged chains.
        type ChainId: Get<ChainId>;

        // SBP-M1 review: missing doc comment
        type ProposalLifetime: Get<Self::BlockNumber>;

        // SBP-M1 review: missing doc comment
        #[pallet::constant]
        type DefaultRelayerThreshold: Get<u32>;

        // SBP-M1 review: missing doc comment
        #[pallet::constant]
        type MaxResourceMetadata: Get<u32>;

        // SBP-M1 review: missing doc comment
        #[pallet::constant]
        type MaxVotes: Get<u32>;
    }

    // SBP-M1 review: consider reducing visibility
    pub type ResourceMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxResourceMetadata>;
    pub type MaxVotesOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxVotes>;
    pub type ProposalVotesOf<T> =
        ProposalVotes<<T as frame_system::Config>::BlockNumber, MaxVotesOf<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Vote threshold has changed (new_threshold)
        RelayerThresholdChanged(u32),
        // SBP-M1 review: consider renaming to AllowListed
        /// Chain now available for transfers (chain_id)
        ChainWhitelisted(ChainId),
        /// Relayer added to set
        RelayerAdded(T::AccountId),
        /// Relayer removed from set
        RelayerRemoved(T::AccountId),
        // SBP-M1 review: typo
        /// FunglibleTransfer is for relaying fungibles (dest_id, nonce, resource_id, amount, recipient, metadata)
        FungibleTransfer(ChainId, DepositNonce, ResourceId, U256, Vec<u8>),
        /// NonFungibleTransfer is for relaying NFTS (dest_id, nonce, resource_id, token_id, recipient, metadata)
        NonFungibleTransfer(ChainId, DepositNonce, ResourceId, Vec<u8>, Vec<u8>, Vec<u8>),
        /// GenericTransfer is for a generic data payload (dest_id, nonce, resource_id, metadata)
        GenericTransfer(ChainId, DepositNonce, ResourceId, Vec<u8>),
        /// Vote submitted in favour of proposal
        VoteFor(ChainId, DepositNonce, T::AccountId),
        /// Vot submitted against proposal
        VoteAgainst(ChainId, DepositNonce, T::AccountId),
        /// Voting successful for a proposal
        ProposalApproved(ChainId, DepositNonce),
        /// Voting rejected a proposal
        ProposalRejected(ChainId, DepositNonce),
        /// Execution of call succeeded
        ProposalSucceeded(ChainId, DepositNonce),
        /// Execution of call failed
        ProposalFailed(ChainId, DepositNonce),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Relayer threshold not set
        ThresholdNotSet,
        /// Provided chain Id is not valid
        InvalidChainId,
        /// Relayer threshold cannot be 0
        InvalidThreshold,
        /// Interactions with this chain is not permitted
        // SBP-M1 review: consider renaming to AllowListed
        ChainNotWhitelisted,
        /// Chain has already been enabled
        // SBP-M1 review: consider renaming to AllowListed
        ChainAlreadyWhitelisted,
        /// Resource ID provided isn't mapped to anything
        ResourceDoesNotExist,
        /// Relayer already in set
        RelayerAlreadyExists,
        /// Provided accountId is not a relayer
        RelayerInvalid,
        /// Protected operation, must be performed by relayer
        MustBeRelayer,
        /// Relayer has already submitted some vote for this proposal
        RelayerAlreadyVoted,
        /// A proposal with these parameters has already been submitted
        ProposalAlreadyExists,
        /// No proposal with the ID was found
        ProposalDoesNotExist,
        /// Cannot complete proposal, needs more votes
        ProposalNotComplete,
        /// Proposal has either failed or succeeded
        ProposalAlreadyComplete,
        /// Lifetime of proposal has been exceeded
        ProposalExpired,
        /// Maximum allowed votes reached
        MaxVotesReached,
    }

    #[pallet::storage]
    #[pallet::getter(fn relayers)]
    pub(super) type Relayers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn resources)]
    pub(super) type Resources<T: Config> =
        StorageMap<_, Blake2_128Concat, ResourceId, ResourceMetadataOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn chains)]
    pub(super) type ChainNonces<T: Config> = StorageMap<_, Blake2_128Concat, ChainId, DepositNonce>;

    #[pallet::storage]
    #[pallet::getter(fn relayer_threshold)]
    // SBP-M1 review: pub(super)
    pub type RelayerThreshold<T: Config> =
        StorageValue<_, u32, ValueQuery, T::DefaultRelayerThreshold>;

    #[pallet::storage]
    #[pallet::getter(fn relayer_count)]
    // SBP-M1 review: pub(super)
    pub type RelayerCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub(super) type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ChainId,
        Blake2_128,
        (DepositNonce, T::Hash),
        ProposalVotesOf<T>,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets the vote threshold for proposals.
        ///
        /// This threshold is used to determine how many votes are required
        /// before a proposal is executed.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[pallet::call_index(0)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn set_threshold(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::set_relayer_threshold(threshold)
        }

        /// Stores a method name on chain under an associated resource ID.
        ///
        /// # <weight>
        /// - O(1) write
        /// # </weight>
        #[pallet::call_index(1)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn set_resource(
            origin: OriginFor<T>,
            id: ResourceId,
            method: ResourceMetadataOf<T>,
        ) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::register_resource(id, method)
        }

        /// Removes a resource ID from the resource mapping.
        ///
        /// After this call, bridge transfers with the associated resource ID will
        /// be rejected.
        ///
        /// # <weight>
        /// - O(1) removal
        /// # </weight>
        #[pallet::call_index(2)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn remove_resource(origin: OriginFor<T>, id: ResourceId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::unregister_resource(id)
        }

        /// Enables a chain ID as a source or destination for a bridge transfer.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[pallet::call_index(3)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        // SBP-M1 review: consider renaming to allow_chain
        pub fn whitelist_chain(origin: OriginFor<T>, id: ChainId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::whitelist(id)
        }

        /// Adds a new relayer to the relayer set.
        ///
        /// # <weight>
        /// - O(1) lookup and insert
        /// # </weight>
        #[pallet::call_index(4)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn add_relayer(origin: OriginFor<T>, v: T::AccountId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::register_relayer(v)
        }

        /// Removes an existing relayer from the set.
        ///
        /// # <weight>
        /// - O(1) lookup and removal
        /// # </weight>
        #[pallet::call_index(5)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn remove_relayer(origin: OriginFor<T>, v: T::AccountId) -> DispatchResult {
            Self::ensure_admin(origin)?;
            Self::unregister_relayer(v)
        }

        /// Commits a vote in favour of the provided proposal.
        ///
        /// If a proposal with the given nonce and source chain ID does not already exist, it will
        /// be created with an initial vote in favour from the caller.
        ///
        /// # <weight>
        /// - weight of proposed call, regardless of whether execution is performed
        /// # </weight>
        // SBP-M1 review: remove commented out weight attribute
        // #[weight = (call.get_dispatch_info().weight + 195_000_000, call.get_dispatch_info().class, Pays::Yes)]
        #[pallet::call_index(6)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn acknowledge_proposal(
            origin: OriginFor<T>,
            nonce: DepositNonce,
            src_id: ChainId,
            r_id: ResourceId,
            call: Box<<T as Config>::Proposal>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_relayer(&who), Error::<T>::MustBeRelayer);
            ensure!(
                Self::chain_whitelisted(src_id),
                Error::<T>::ChainNotWhitelisted
            );
            ensure!(
                Self::resource_exists(r_id),
                Error::<T>::ResourceDoesNotExist
            );

            Self::vote_for(who, nonce, src_id, call)
        }

        /// Commits a vote against a provided proposal.
        ///
        /// # <weight>
        /// - Fixed, since execution of proposal should not be included
        /// # </weight>
        #[pallet::call_index(7)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn reject_proposal(
            origin: OriginFor<T>,
            nonce: DepositNonce,
            src_id: ChainId,
            r_id: ResourceId,
            call: Box<<T as Config>::Proposal>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_relayer(&who), Error::<T>::MustBeRelayer);
            ensure!(
                Self::chain_whitelisted(src_id),
                Error::<T>::ChainNotWhitelisted
            );
            ensure!(
                Self::resource_exists(r_id),
                Error::<T>::ResourceDoesNotExist
            );

            Self::vote_against(who, nonce, src_id, call)
        }

        /// Evaluate the state of a proposal given the current vote threshold.
        ///
        /// A proposal with enough votes will be either executed or cancelled, and the status
        /// will be updated accordingly.
        ///
        /// # <weight>
        /// - weight of proposed call, regardless of whether execution is performed
        /// # </weight>
        // SBP-M1 review: remove commented out weight attribute
        // #[weight = (prop.get_dispatch_info().weight + 195_000_000, prop.get_dispatch_info().class, Pays::Yes)]
        #[pallet::call_index(8)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn eval_vote_state(
            origin: OriginFor<T>,
            nonce: DepositNonce,
            src_id: ChainId,
            prop: Box<<T as Config>::Proposal>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            Self::try_resolve_proposal(nonce, src_id, prop)
        }
    }
}

impl<T: Config> Pallet<T> {
    // *** Utility methods ***

    // SBP-M1 review: reduce visibility
    pub fn ensure_admin(o: T::RuntimeOrigin) -> DispatchResult {
        T::AdminOrigin::try_origin(o)
            .map(|_| ())
            .or_else(ensure_root)?;
        Ok(())
    }

    /// Checks if who is a relayer
    // SBP-M1 review: reduce visibility
    pub fn is_relayer(who: &T::AccountId) -> bool {
        Self::relayers(who)
    }

    /// Provides an AccountId for the pallet.
    /// This is used both as an origin check and deposit/withdrawal account.
    // SBP-M1 review: reduce visibility
    pub fn account_id() -> T::AccountId {
        <T as Config>::PalletId::get().into_account_truncating()
    }

    /// Asserts if a resource is registered
    // SBP-M1 review: reduce visibility
    // SBP-M1 review: use .is_some()
    pub fn resource_exists(id: ResourceId) -> bool {
        // SBP-M1 review: unneeded return statement
        return Self::resources(id) != None;
    }

    // SBP-M1 review: consider renaming to chain_allowlisted
    /// Checks if a chain exists as a whitelisted destination
    // SBP-M1 review: reduce visibility
    // SBP-M1 review: use .is_some()
    pub fn chain_whitelisted(id: ChainId) -> bool {
        // SBP-M1 review: unneeded return statement
        return Self::chains(id) != None;
    }

    /// Increments the deposit nonce for the specified chain ID
    fn bump_nonce(id: ChainId) -> DepositNonce {
        // SBP-M1 review: use <ChainNonces<T>>.mutate() which also allows a return value
        // SBP-M1 review: use safe math
        let nonce = Self::chains(id).unwrap_or_default() + 1;
        ChainNonces::<T>::insert(id, nonce);
        nonce
    }

    // *** Admin methods ***

    /// Set a new voting threshold
    // SBP-M1 review: reduce visibility
    pub fn set_relayer_threshold(threshold: u32) -> DispatchResult {
        ensure!(threshold > 0, Error::<T>::InvalidThreshold);
        RelayerThreshold::<T>::put(threshold);
        Self::deposit_event(Event::RelayerThresholdChanged(threshold));
        Ok(())
    }

    /// Register a method for a resource Id, enabling associated transfers
    // SBP-M1 review: reduce visibility
    pub fn register_resource(id: ResourceId, method: ResourceMetadataOf<T>) -> DispatchResult {
        Resources::<T>::insert(id, method);
        Ok(())
    }

    /// Removes a resource ID, disabling associated transfer
    // SBP-M1 review: reduce visibility
    pub fn unregister_resource(id: ResourceId) -> DispatchResult {
        Resources::<T>::remove(id);
        Ok(())
    }

    // SBP-M1 review: consider renaming to 'allow'
    /// Whitelist a chain ID for transfer
    // SBP-M1 review: reduce visibility
    pub fn whitelist(id: ChainId) -> DispatchResult {
        // Cannot whitelist this chain
        ensure!(id != T::ChainId::get(), Error::<T>::InvalidChainId);
        // Cannot whitelist with an existing entry
        ensure!(
            !Self::chain_whitelisted(id),
            Error::<T>::ChainAlreadyWhitelisted
        );
        // SBP-M1 review: unnecessary borrow
        ChainNonces::<T>::insert(&id, 0);
        Self::deposit_event(Event::ChainWhitelisted(id));
        Ok(())
    }

    /// Adds a new relayer to the set
    // SBP-M1 review: reduce visibility
    pub fn register_relayer(relayer: T::AccountId) -> DispatchResult {
        ensure!(
            !Self::is_relayer(&relayer),
            Error::<T>::RelayerAlreadyExists
        );
        Relayers::<T>::insert(&relayer, true);
        // SBP-M1 review: use safe math
        RelayerCount::<T>::mutate(|i| *i += 1);

        Self::deposit_event(Event::RelayerAdded(relayer));
        Ok(())
    }

    /// Removes a relayer from the set
    // SBP-M1 review: reduce visibility
    pub fn unregister_relayer(relayer: T::AccountId) -> DispatchResult {
        ensure!(Self::is_relayer(&relayer), Error::<T>::RelayerInvalid);
        Relayers::<T>::remove(&relayer);
        // SBP-M1 review: use safe math
        RelayerCount::<T>::mutate(|i| *i -= 1);
        Self::deposit_event(Event::RelayerRemoved(relayer));
        Ok(())
    }

    // *** Proposal voting and execution methods ***

    /// Commits a vote for a proposal. If the proposal doesn't exist it will be created.
    // SBP-M1 review: too many lines, refactor
    fn commit_vote(
        // SBP-M1 review: not consumed, consider borrow
        who: T::AccountId,
        nonce: DepositNonce,
        src_id: ChainId,
        // SBP-M1 review: not consumed, consider borrow
        prop: Box<T::Proposal>,
        in_favour: bool,
    ) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let encoded_call = prop.encode();
        let call_hash = <T as frame_system::Config>::Hashing::hash(&encoded_call[..]);
        // SBP-M1 review: consider <Votes<T>>::try_mutate(.. -> DispatchResult) or .map_or_else(..)
        let mut votes = match <Votes<T>>::get(src_id, (nonce, call_hash)) {
            Some(v) => v,
            None => {
                let mut v = ProposalVotes {
                    votes_for: BoundedVec::default(),
                    votes_against: BoundedVec::default(),
                    status: ProposalStatus::Initiated,
                    expiry: T::BlockNumber::default(),
                };
                // SBP-M1 review: move to initializer, negating need for mut
                // SBP-M1 review: use safe math
                v.expiry = now + T::ProposalLifetime::get();
                v
            }
        };

        // Ensure the proposal isn't complete and relayer hasn't already voted
        ensure!(
            !Self::is_complete(&votes),
            Error::<T>::ProposalAlreadyComplete
        );
        ensure!(!Self::is_expired(&votes, now), Error::<T>::ProposalExpired);
        ensure!(
            !Self::has_voted(&votes, &who),
            Error::<T>::RelayerAlreadyVoted
        );

        if in_favour {
            // SBP-M1 review: .ok_or()? another approach
            ensure!(
                votes.votes_for.try_push(who.clone()).is_ok(),
                Error::<T>::MaxVotesReached
            );
            // SBP-M1 review: unnecessary clone
            Self::deposit_event(Event::VoteFor(src_id, nonce, who.clone()));
        } else {
            // SBP-M1 review: .ok_or()? another approach
            ensure!(
                votes.votes_against.try_push(who.clone()).is_ok(),
                Error::<T>::MaxVotesReached
            );
            // SBP-M1 review: unnecessary clone
            Self::deposit_event(Event::VoteAgainst(src_id, nonce, who.clone()));
        }

        // SBP-M1 review: unnecessary clone
        <Votes<T>>::insert(src_id, (nonce, call_hash), votes.clone());

        Ok(())
    }

    /// Attempts to mark the proposal as approve or rejected.
    /// Returns true if the status changes from active.
    // SBP-M1 review: consider moving to ProposalVotes struct impl as method
    fn try_to_complete(
        votes: &mut ProposalVotes<T::BlockNumber, MaxVotesOf<T>>,
        threshold: u32,
        total: u32,
    ) -> ProposalStatus {
        if votes.votes_for.len() >= threshold as usize {
            votes.status = ProposalStatus::Approved;
            ProposalStatus::Approved
            // SBP-M1 review: use safe math and cast may truncate
        } else if total >= threshold && votes.votes_against.len() as u32 + threshold > total {
            votes.status = ProposalStatus::Rejected;
            ProposalStatus::Rejected
        } else {
            ProposalStatus::Initiated
        }
    }

    /// Returns true if the proposal has been rejected or approved, otherwise false.
    // SBP-M1 review: consider moving to ProposalVotes struct impl as method
    fn is_complete(votes: &ProposalVotes<T::BlockNumber, MaxVotesOf<T>>) -> bool {
        votes.status != ProposalStatus::Initiated
    }

    /// Return true if the expiry time has been reached
    // SBP-M1 review: consider moving to ProposalVotes struct impl as method
    fn is_expired(
        votes: &ProposalVotes<T::BlockNumber, MaxVotesOf<T>>,
        now: T::BlockNumber,
    ) -> bool {
        votes.expiry <= now
    }

    /// Returns true if `who` has voted for or against the proposal
    // SBP-M1 review: consider moving to ProposalVotes struct impl as method
    fn has_voted(votes: &ProposalVotes<T::BlockNumber, MaxVotesOf<T>>, who: &T::AccountId) -> bool {
        // SBP-M1 review: needless borrows
        votes.votes_for.contains(&who) || votes.votes_against.contains(&who)
    }

    /// Attempts to finalize or cancel the proposal if the vote count allows.
    fn try_resolve_proposal(
        nonce: DepositNonce,
        src_id: ChainId,
        prop: Box<T::Proposal>,
    ) -> DispatchResult {
        let encoded_call = prop.encode();
        let call_hash = <T as frame_system::Config>::Hashing::hash(&encoded_call[..]);
        // SBP-M1 review: consider <Votes<T>>::try_mutate(..) or let-else { return Err(..) }
        if let Some(mut votes) = <Votes<T>>::get(src_id, (nonce, call_hash)) {
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                !Self::is_complete(&votes),
                Error::<T>::ProposalAlreadyComplete
            );
            ensure!(!Self::is_expired(&votes, now), Error::<T>::ProposalExpired);

            let status = Self::try_to_complete(
                &mut votes,
                RelayerThreshold::<T>::get(),
                RelayerCount::<T>::get(),
            );
            <Votes<T>>::insert(src_id, (nonce, call_hash), votes.clone());

            match status {
                ProposalStatus::Approved => Self::finalize_execution(src_id, nonce, prop),
                ProposalStatus::Rejected => Self::cancel_execution(src_id, nonce),
                // SBP-M1 review: handle remaining variants explicitly, wildcard will match future added variants
                _ => Ok(()),
            }
        } else {
            // SBP-M1 review: use 'return Err(..)' syntax instead
            Err(Error::<T>::ProposalDoesNotExist)?
        }
    }

    /// Commits a vote in favour of the proposal and executes it if the vote threshold is met.
    fn vote_for(
        who: T::AccountId,
        nonce: DepositNonce,
        src_id: ChainId,
        prop: Box<T::Proposal>,
    ) -> DispatchResult {
        Self::commit_vote(who, nonce, src_id, prop.clone(), true)?;
        Self::try_resolve_proposal(nonce, src_id, prop)
    }

    /// Commits a vote against the proposal and cancels it if more than (relayers.len() - threshold)
    /// votes against exist.
    fn vote_against(
        who: T::AccountId,
        nonce: DepositNonce,
        src_id: ChainId,
        prop: Box<T::Proposal>,
    ) -> DispatchResult {
        Self::commit_vote(who, nonce, src_id, prop.clone(), false)?;
        Self::try_resolve_proposal(nonce, src_id, prop)
    }

    /// Execute the proposal and signals the result as an event
    fn finalize_execution(
        src_id: ChainId,
        nonce: DepositNonce,
        call: Box<T::Proposal>,
    ) -> DispatchResult {
        Self::deposit_event(Event::ProposalApproved(src_id, nonce));
        call.dispatch(frame_system::RawOrigin::Signed(Self::account_id()).into())
            .map(|_| ())
            .map_err(|e| e.error)?;
        Self::deposit_event(Event::ProposalSucceeded(src_id, nonce));
        Ok(())
    }

    /// Cancels a proposal.
    // SBP-M1 review: return value unnecessary
    fn cancel_execution(src_id: ChainId, nonce: DepositNonce) -> DispatchResult {
        Self::deposit_event(Event::ProposalRejected(src_id, nonce));
        Ok(())
    }

    /// Initiates a transfer of a fungible asset out of the chain. This should be called by another pallet.
    // SBP-M1 review: doesnt seem to initiate any transfer, just increments nonce and emits event
    // SBP-M1 review: not used beyond test
    // SBP-M1 review: reduce visibility
    pub fn transfer_fungible(
        dest_id: ChainId,
        resource_id: ResourceId,
        to: Vec<u8>,
        amount: U256,
    ) -> DispatchResult {
        ensure!(
            Self::chain_whitelisted(dest_id),
            Error::<T>::ChainNotWhitelisted
        );
        let nonce = Self::bump_nonce(dest_id);
        Self::deposit_event(Event::FungibleTransfer(
            dest_id,
            nonce,
            resource_id,
            amount,
            to,
        ));
        Ok(())
    }

    /// Initiates a transfer of a nonfungible asset out of the chain. This should be called by another pallet.
    // SBP-M1 review: doesnt seem to initiate any transfer, just increments nonce and emits event
    // SBP-M1 review: not used beyond test
    // SBP-M1 review: reduce visibility
    pub fn transfer_nonfungible(
        dest_id: ChainId,
        resource_id: ResourceId,
        token_id: Vec<u8>,
        to: Vec<u8>,
        metadata: Vec<u8>,
    ) -> DispatchResult {
        ensure!(
            Self::chain_whitelisted(dest_id),
            Error::<T>::ChainNotWhitelisted
        );
        let nonce = Self::bump_nonce(dest_id);
        Self::deposit_event(Event::NonFungibleTransfer(
            dest_id,
            nonce,
            resource_id,
            token_id,
            to,
            metadata,
        ));
        Ok(())
    }

    /// Initiates a transfer of generic data out of the chain. This should be called by another pallet.
    // SBP-M1 review: reduce visibility
    pub fn transfer_generic(
        dest_id: ChainId,
        resource_id: ResourceId,
        metadata: Vec<u8>,
    ) -> DispatchResult {
        ensure!(
            Self::chain_whitelisted(dest_id),
            Error::<T>::ChainNotWhitelisted
        );
        let nonce = Self::bump_nonce(dest_id);
        Self::deposit_event(Event::GenericTransfer(
            dest_id,
            nonce,
            resource_id,
            metadata,
        ));
        Ok(())
    }
}

/// Simple ensure origin for the bridge account
pub struct EnsureBridge<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> EnsureOrigin<T::RuntimeOrigin> for EnsureBridge<T> {
    type Success = T::AccountId;
    fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
        // SBP-M1 review: use <Pallet<T>>::account_id() for consistency
        let bridge_id = <T as Config>::PalletId::get().into_account_truncating();
        o.into().and_then(|o| match o {
            frame_system::RawOrigin::Signed(who) if who == bridge_id => Ok(bridge_id),
            r => Err(T::RuntimeOrigin::from(r)),
        })
    }

    /// Returns an outer origin capable of passing `try_origin` check.
    ///
    /// ** Should be used for benchmarking only!!! **
    #[cfg(feature = "runtime-benchmarks")]
    // SBP-M1 review: should by try_successful_origin, assume this wasn't updated after updating Substrate
    fn successful_origin() -> T::RuntimeOrigin {
        T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(<Module<T>>::account_id()))
    }
}
