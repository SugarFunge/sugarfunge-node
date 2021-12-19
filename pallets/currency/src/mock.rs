use crate as sugarfunge_currency;
use frame_support::{
    parameter_types,
    traits::{Everything, GenesisBuild, Nothing, OnFinalize, OnInitialize},
    PalletId,
};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, Zero},
};
use sugarfunge_primitives::{Amount, AssetSymbol, Balance, BlockNumber, CurrencyId};

pub const MILLICENTS: Balance = 10_000_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS;
pub const DOLLARS: Balance = 100 * CENTS;

pub const SUGAR: CurrencyId = CurrencyId::Asset(AssetSymbol::SUGAR);
pub const ETH: CurrencyId = CurrencyId::Asset(AssetSymbol::ETH);
pub const BTC: CurrencyId = CurrencyId::Asset(AssetSymbol::BTC);

parameter_types! {
    pub const CreateAssetClassDeposit: Balance = 500 * MILLICENTS;
    pub const CreateCurrencyClassDeposit: Balance = 500 * MILLICENTS;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

impl pallet_sudo::Config for Test {
    type Event = Event;
    type Call = Call;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
        Zero::zero()
    };
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
    type MaxLocks = ();
    type DustRemovalWhitelist = Nothing;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Asset(AssetSymbol::SUGAR);
}

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = OrmlTokens;
    type NativeCurrency =
        orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

impl sugarfunge_asset::Config for Test {
    type Event = Event;
    type CreateAssetClassDeposit = CreateAssetClassDeposit;
    type Currency = Balances;
    type AssetId = u64;
    type ClassId = u64;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        OrmlTokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        OrmlCurrencies: orml_currencies::{Pallet, Storage, Call, Event<T>},
        Asset: sugarfunge_asset::{Pallet, Call, Storage, Event<T>},
        Currency: sugarfunge_currency::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const CurrencyModuleId: PalletId = PalletId(*b"sug/curr");
    pub const DexModuleId: PalletId = PalletId(*b"sug/dexm");
}

impl sugarfunge_currency::Config for Test {
    type Event = Event;
    type PalletId = CurrencyModuleId;
    type Currency = OrmlCurrencies;
    type CreateCurrencyClassDeposit = CreateCurrencyClassDeposit;
    type GetNativeCurrencyId = GetNativeCurrencyId;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 1000000 * DOLLARS), (2, 1000000 * DOLLARS)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_sudo::GenesisConfig::<Test> { key: 1 }
        .assimilate_storage(&mut t)
        .unwrap();
    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (1, CurrencyId::Asset(AssetSymbol::DOT), 1000000 * DOLLARS),
            (1, CurrencyId::Asset(AssetSymbol::ETH), 1000000 * DOLLARS),
            (1, CurrencyId::Asset(AssetSymbol::BTC), 1000000 * DOLLARS),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    sugarfunge_currency::GenesisConfig::<Test> {
        class: (1, 0, [].to_vec()),
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Currency::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Currency::on_initialize(System::block_number());
    }
}
