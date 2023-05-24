//! Benchmarking setup for sugarfunge-bundle
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Bundle;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;


	#[benchmark]
	fn do_something() {
		let value = 100u32.into();
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		do_something(RawOrigin::Signed(caller), value);
		assert_eq!(Something::<T>::get(), Some(value));
	}

	#[benchmark]
    fn verify() {
        assert_eq!(Something::<T>::get(), Some(s));
    }

    impl_benchmark_test_suite!(Bundle, crate::mock::new_test_ext(), crate::mock::Test);
}
