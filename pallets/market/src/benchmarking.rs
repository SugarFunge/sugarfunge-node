//! Benchmarking setup for sugarfunge-market

#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Market;
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

    impl_benchmark_test_suite!(Market, crate::mock::new_test_ext(), crate::mock::Test);
}
