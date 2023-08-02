// SBP-M1 review: cargo fmt
//! # Validator Set Pallet
//!
//! The Validator Set Pallet allows addition and removal of
//! authorities/validators via extrinsics (transaction calls), in
//! Substrate-based PoA networks. It also integrates with the im-online pallet
//! to automatically remove offline validators.
//!
//! The pallet uses the Session pallet and implements related traits for session
//! management. Currently it uses periodic session rotation provided by the
//! session pallet to automatically rotate sessions. For this reason, the
//! validator addition and removal becomes effective only after 2 sessions
//! (queuing + applying).

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{EstimateNextSessionRotation, Get, ValidatorSet, ValidatorSetWithIdentification},
	BoundedVec,
};
use log;
// SBP-M1 review: consider pub(crate)
pub use pallet::*;
use sp_runtime::traits::{Convert, Zero};
use sp_staking::offence::{Offence, OffenceError, ReportOffence};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

// SBP-M1 review: static lifetime redundant
// SBP-M1 review: consider pub(crate)
pub const LOG_TARGET: &'static str = "runtime::validator-set";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	// SBP-M1 review: consider loose-coupling via type ValidatorSet: ValidatorSet<Self::AccountId> config item rather than pallet_session::Config
	pub trait Config: frame_system::Config + pallet_session::Config {
		/// The Event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// RuntimeOrigin for adding or removing a validator.
		type AddRemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Minimum number of validators to leave in the validator set during
		/// auto removal.
		type MinAuthorities: Get<u32>;

		// SBP-M1 review: missing doc comment
		type MaxAuthorities: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn validators)]
	// SBP-M1 review: consider pub(super)
	pub type Validators<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxAuthorities>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn approved_validators)]
	// SBP-M1 review: consider pub(super)
	pub type ApprovedValidators<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxAuthorities>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators_to_remove)]
	// SBP-M1 review: consider pub(super)
	pub type OfflineValidators<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxAuthorities>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New validator addition initiated. Effective in ~2 sessions.
		ValidatorAdditionInitiated(T::AccountId),

		/// Validator removal initiated. Effective in ~2 sessions.
		ValidatorRemovalInitiated(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Target (post-removal) validator count is below the minimum.
		TooLowValidatorCount,
		/// Validator is already in the validator set.
		Duplicate,
		/// Validator is not approved for re-addition.
		ValidatorNotApproved,
		/// Only the validator can add itself back after coming online.
		BadOrigin,
	}

	// SBP-M1 review: not used, can be removed
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_validators: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			// SBP-M1 review: consider Vec::default() for clarity
			Self { initial_validators: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		// SBP-M1 review: consider using Vec as parameter type
		fn build(&self) {
			Pallet::<T>::initialize_validators(&self.initial_validators);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a new validator.
		///
		/// New validator's session keys should be set in Session pallet before
		/// calling this.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		#[pallet::call_index(0)]
		// SBP-M1 review: benchmark and use proper weight function to avoid spam/DoS - use DispatchResultWithPostInfo and return Pays:No as per sudo pallet
		#[pallet::weight(Weight::from_parts(0, 0))]
		pub fn add_validator(origin: OriginFor<T>, validator_id: T::AccountId) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::do_add_validator(validator_id.clone())?;
			Self::approve_validator(validator_id)?;

			Ok(())
		}

		/// Remove a validator.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		#[pallet::call_index(1)]
		// SBP-M1 review: benchmark and use proper weight function to avoid spam/DoS - use DispatchResultWithPostInfo and return Pays:No as per sudo pallet
		#[pallet::weight(Weight::from_parts(0, 0))]
		pub fn remove_validator(
			origin: OriginFor<T>,
			validator_id: T::AccountId,
		) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::do_remove_validator(validator_id.clone())?;
			// SBP-M1 review: appears to have no effect
			Self::unapprove_validator(validator_id)?;

			Ok(())
		}

		/// Add an approved validator again when it comes back online.
		///
		/// For this call, the dispatch origin must be the validator itself.
		#[pallet::call_index(2)]
		// SBP-M1 review: benchmark and use proper weight function to avoid spam/DoS - use DispatchResultWithPostInfo and return Pays:No as per sudo pallet
		#[pallet::weight(Weight::from_parts(0, 0))]
		pub fn add_validator_again(
			origin: OriginFor<T>,
			// SBP-M1 review: why have this parameter when it can just be obtained via ensure_signed?
			validator_id: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == validator_id, Error::<T>::BadOrigin);

			// SBP-M1 review: use BoundedVec.contains()
			let approved_set: BTreeSet<_> = <ApprovedValidators<T>>::get().into_iter().collect();
			ensure!(approved_set.contains(&validator_id), Error::<T>::ValidatorNotApproved);

			Self::do_add_validator(validator_id)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn initialize_validators(validators: &[T::AccountId]) {
		assert!(
			validators.len() as u32 >= T::MinAuthorities::get(),
			"Initial set of validators must be at least T::MinAuthorities"
		);
		assert!(
			(validators.len() as u32) < (T::MaxAuthorities::get()),
			// SBP-M1 review: improve comment
			"Initial set of validators must be at less than T::MaxAuthorities"
		);
		assert!(<Validators<T>>::get().is_empty(), "Validators are already initialized!");
		let validators: BoundedVec<T::AccountId, T::MaxAuthorities> =
			// SBP-M1 review: avoid unwrap (panic)
			validators.to_vec().try_into().unwrap();
		// SBP-M1 review: use & to avoid clone
		<Validators<T>>::put(validators.clone());
		<ApprovedValidators<T>>::put(validators);
	}

	fn do_add_validator(validator_id: T::AccountId) -> DispatchResult {
		// SBP-M1 review: unnecessary set, use .contains() on BoundedVec
		let validator_set: BTreeSet<_> = <Validators<T>>::get().into_iter().collect();
		ensure!(!validator_set.contains(&validator_id), Error::<T>::Duplicate);

		// SBP-M1 review: use BoundedVec from first storage query
		let mut validators = <Validators<T>>::get().to_vec();
		validators.push(validator_id.clone());
		let validators: BoundedVec<T::AccountId, T::MaxAuthorities> =
			// SBP-M1 review: avoid unwrap, return error
			validators.to_vec().try_into().unwrap();
		<Validators<T>>::put(validators);

		// SBP-M1 review: unnecessary clone
		Self::deposit_event(Event::ValidatorAdditionInitiated(validator_id.clone()));
		log::debug!(target: LOG_TARGET, "Validator addition initiated.");

		Ok(())
	}

	fn do_remove_validator(validator_id: T::AccountId) -> DispatchResult {
		let mut validators = <Validators<T>>::get();

		// Ensuring that the post removal, target validator count doesn't go
		// below the minimum.
		ensure!(
			// SBP-M1 review: cast may truncate
			validators.len().saturating_sub(1) as u32 >= T::MinAuthorities::get(),
			Error::<T>::TooLowValidatorCount
		);

		validators.retain(|v| *v != validator_id);

		<Validators<T>>::put(validators);

		// SBP-M1 review: unnecessary clone
		Self::deposit_event(Event::ValidatorRemovalInitiated(validator_id.clone()));
		log::debug!(target: LOG_TARGET, "Validator removal initiated.");

		Ok(())
	}

	fn approve_validator(validator_id: T::AccountId) -> DispatchResult {
		// SBP-M1 review: doesnt need to be a set to check if validator_id exists
		let approved_set: BTreeSet<_> = <ApprovedValidators<T>>::get().into_iter().collect();
		ensure!(!approved_set.contains(&validator_id), Error::<T>::Duplicate);

		// SBP-M1 review: use .into_inner() to use existing vector
		let mut validators = <ApprovedValidators<T>>::get().to_vec();
		// SBP-M1 review: unnecessary clone
		validators.push(validator_id.clone());
		// SBP-M1 review: use BoundedVec received directly from storage, avoid unwrap, unnecessary clone
		let validators: BoundedVec<T::AccountId, T::MaxAuthorities> =
			validators.to_vec().try_into().unwrap();
		<ApprovedValidators<T>>::put(validators);

		Ok(())
	}

	// SBP-M1 review: appears to have no effect as storage not updated with updated set
	fn unapprove_validator(validator_id: T::AccountId) -> DispatchResult {
		let mut approved_set = <ApprovedValidators<T>>::get();
		approved_set.retain(|v| *v != validator_id);
		// SBP-M1 review: return type unnecessary
		Ok(())
	}

	// SBP-M1 review: comment should be refined based on logic: validator_id added to OfflineValidators
	// Adds offline validators to a local cache for removal at new session.
	fn mark_for_removal(validator_id: T::AccountId) {
		// SBP-M1 review: use .into_inner() to access inner vec rather than implicit clone
		let mut validators = <OfflineValidators<T>>::get().to_vec();
		// SBP-M1 review: clone unnecessary
		validators.push(validator_id.clone());
		// SBP-M1 review: why not just use the BoundedVec from OfflineValidators::get()
		let validators: BoundedVec<T::AccountId, T::MaxAuthorities> =
			// SBP-M1 review: already its own vector, unwrap could panic
			validators.to_vec().try_into().unwrap();
		<OfflineValidators<T>>::put(validators);

		log::debug!(target: LOG_TARGET, "Offline validator marked for auto removal.");
	}

	// Removes offline validators from the validator set and clears the offline
	// cache. It is called in the session change hook and removes the validators
	// who were reported offline during the session that is ending. We do not
	// check for `MinAuthorities` here, because the offline validators will not
	// produce blocks and will have the same overall effect on the runtime.
	fn remove_offline_validators() {
		// SBP-M1 review: set may be unnecessary if all OfflineValidators modifications ensure distinct items
		let validators_to_remove: BTreeSet<_> = <OfflineValidators<T>>::get().into_iter().collect();

		// Delete from active validator set.
		<Validators<T>>::mutate(|vs| vs.retain(|v| !validators_to_remove.contains(v)));
		log::debug!(
			target: LOG_TARGET,
			"Initiated removal of {:?} offline validators.",
			validators_to_remove.len()
		);

		// Clear the offline validator list to avoid repeated deletion.
		// SBP-M1 review: use BoundedVec::new() or BoundedVec::default(), avoid .unwrap()
		let validators: BoundedVec<T::AccountId, T::MaxAuthorities> = vec![].try_into().unwrap();
		<OfflineValidators<T>>::put(validators);
	}
}

// Provides the new set of validators to the session module when session is
// being rotated.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	// Plan a new session and provide new validator set.
	fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
		// Remove any offline validators. This will only work when the runtime
		// also has the im-online pallet.
		// SBP-M1 review: consider returning validator set from this function as it already reads validators state
		Self::remove_offline_validators();

		log::debug!(target: LOG_TARGET, "New session called; updated validator set provided.");

		// SBP-M1 review: use .into_inner()
		Some(Self::validators().to_vec())
	}

	fn end_session(_end_index: u32) {}

	fn start_session(_start_index: u32) {}
}

// SBP-M1 review: provide justification as to why this provides default values
impl<T: Config> EstimateNextSessionRotation<T::BlockNumber> for Pallet<T> {
	fn average_session_length() -> T::BlockNumber {
		Zero::zero()
	}

	fn estimate_current_session_progress(
		_now: T::BlockNumber,
		// SBP-M1 review: unnecessary qualification
	) -> (Option<sp_runtime::Permill>, frame_support::dispatch::Weight) {
		(None, Zero::zero())
	}

	fn estimate_next_session_rotation(
		_now: T::BlockNumber,
		// SBP-M1 review: unnecessary qualification
	) -> (Option<T::BlockNumber>, frame_support::dispatch::Weight) {
		(None, Zero::zero())
	}
}

// Implementation of Convert trait for mapping ValidatorId with AccountId.
// SBP-M1 review: unnecessary qualification
pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::ValidatorId, Option<T::ValidatorId>> for ValidatorOf<T> {
	fn convert(account: T::ValidatorId) -> Option<T::ValidatorId> {
		Some(account)
	}
}

// SBP-M1 review: consider loose coupling with pallet_session injected via type with ValidatorSet trait bound on Config
impl<T: Config> ValidatorSet<T::AccountId> for Pallet<T> {
	type ValidatorId = T::ValidatorId;
	type ValidatorIdOf = T::ValidatorIdOf;

	fn session_index() -> sp_staking::SessionIndex {
		pallet_session::Pallet::<T>::current_index()
	}

	fn validators() -> Vec<Self::ValidatorId> {
		pallet_session::Pallet::<T>::validators()
	}
}

impl<T: Config> ValidatorSetWithIdentification<T::AccountId> for Pallet<T> {
	type Identification = T::ValidatorId;
	type IdentificationOf = ValidatorOf<T>;
}

// Offence reporting and unresponsiveness management.
impl<T: Config, O: Offence<(T::AccountId, T::AccountId)>>
	ReportOffence<T::AccountId, (T::AccountId, T::AccountId), O> for Pallet<T>
{
	fn report_offence(_reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
		let offenders = offence.offenders();

		// SBP-M1 review: simply as 'for (v, _) in offenders {}'
		for (v, _) in offenders.into_iter() {
			// SBP-M1 review: pass offenders as parameters to process as batch
			// SBP-M1 review: should timeslot be passed so that is_known_offence can be properly implemented
			Self::mark_for_removal(v);
		}

		Ok(())
	}

	fn is_known_offence(
		_offenders: &[(T::AccountId, T::AccountId)],
		_time_slot: &O::TimeSlot,
		// SBP-M1 review: provide justification as to why this is not implemented
	) -> bool {
		false
	}
}
