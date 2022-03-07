use crate as sugarfunge_dex;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{GenesisBuild, Nothing, OnFinalize, OnInitialize},
    PalletId,
};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, Zero},
};
use sugarfunge_primitives::{Amount, Balance, BlockNumber, CurrencyId};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const MILLICENTS: Balance = 10_000_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

pub const SUGAR: CurrencyId = CurrencyId(0, 0);
pub const DOT: CurrencyId = CurrencyId(0, 1);
pub const ETH: CurrencyId = CurrencyId(0, 2);
pub const BTC: CurrencyId = CurrencyId(0, 3);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
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
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
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
    pub const GetNativeCurrencyId: CurrencyId = SUGAR;
}

pub type AdaptedBasicCurrency =
    orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = OrmlTokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub const CreateAssetClassDeposit: Balance = 1;
    pub const CreateExchangeDeposit: Balance = 1;
    pub const CreateCurrencyClassDeposit: Balance = 1;
    pub const MaxClassMetadata: u32 = 1;
    pub const MaxAssetMetadata: u32 = 1;
}

impl sugarfunge_asset::Config for Test {
    type Event = Event;
    type CreateAssetClassDeposit = CreateAssetClassDeposit;
    type Currency = Balances;
    type AssetId = u64;
    type ClassId = u64;
    type MaxClassMetadata = MaxClassMetadata;
    type MaxAssetMetadata = MaxAssetMetadata;
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

impl sugarfunge_dex::Config for Test {
    type Event = Event;
    type PalletId = DexModuleId;
    type CreateExchangeDeposit = CreateExchangeDeposit;
    type Currency = Balances;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        OrmlTokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
        OrmlCurrencies: orml_currencies::{Pallet, Call, Event<T>},
        Dex: sugarfunge_dex::{Pallet, Call, Storage, Event<T>},
        Asset: sugarfunge_asset::{Pallet, Call, Storage, Event<T>},
        Currency: sugarfunge_currency::{Pallet, Call, Storage, Event<T>},
    }
);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 100_000_000 * DOLLARS), (2, 100_000_000 * DOLLARS)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    orml_tokens::GenesisConfig::<Test> {
        balances: vec![
            (1, DOT, 100_000_000 * DOLLARS),
            (1, ETH, 100_000_000 * DOLLARS),
            (1, BTC, 100_000_000 * DOLLARS),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    sugarfunge_currency::GenesisConfig::<Test> {
        class: (1, 0, 0, vec![], vec![]),
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Dex::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Dex::on_initialize(System::block_number());
    }
}
