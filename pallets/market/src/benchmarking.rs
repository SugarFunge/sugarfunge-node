//! Benchmarking setup for sugarfunge-market

#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Market;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

// SBP-M1 review: missing benchmarks for dispatchable functions
// SBP-M1 review: add ci to require successful benchmark tests before merging
#[benchmarks]
mod benchmarks {
    use super::*;

	// SBP-M1 review: remove sample benchmark
	#[benchmark]
	fn do_something() {
		let value = 100u32.into();
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		do_something(RawOrigin::Signed(caller), value);
		assert_eq!(Something::<T>::get(), Some(value));
	}

	// SBP-M1 review: remove sample benchmark
	#[benchmark]
    fn verify() {
        assert_eq!(Something::<T>::get(), Some(s));
    }

    impl_benchmark_test_suite!(Market, crate::mock::new_test_ext(), crate::mock::Test);
}
