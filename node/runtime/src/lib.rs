//! This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

pub mod constants;
pub mod impls;

use crate::constants::{currency::*, time::*};
use authority_discovery_primitives::AuthorityId as AuthorityDiscoveryId;
use grandpa::fg_primitives;
use grandpa::AuthorityList as GrandpaAuthorityList;
use im_online::sr25519::AuthorityId as ImOnlineId;
use impls::{CurrencyToVoteHandler, LinearWeightToFee, TargetedFeeAdjustment};
use inherents::{CheckInherentsResult, InherentData};
use node_primitives::{
    AccountId, AccountIndex, Balance, BlockNumber, Hash, Index, Moment, Signature,
};
use primitives::u32_trait::{_1, _2, _3, _4, _5};
use primitives::OpaqueMetadata;
use sp_api::impl_runtime_apis;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::traits::{
    self, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys, SaturatedConversion, StaticLookup,
};
use sp_runtime::transaction_validity::TransactionValidity;
use sp_runtime::{create_runtime_str, generic, ApplyExtrinsicResult, Percent};
use sp_std::prelude::*;
use support::{
    construct_runtime, debug, parameter_types,
    traits::{Currency, Imbalance, OnUnbalanced, Randomness, SplitTwoWays},
    weights::Weight,
};
use system::offchain::TransactionSubmitter;
use transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
#[cfg(feature = "std")]
use version::NativeVersion;
use version::RuntimeVersion;
// use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::Imbalance;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use balances::Call as BalancesCall;
pub use sp_runtime::{impl_opaque_keys, Perbill, Permill};
pub use staking::StakerStatus;
pub use support::StorageValue;
pub use system::EventRecord;
pub use timestamp::Call as TimestampCall;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("dnachain"),
    impl_name: create_runtime_str!("Metaverse-DNA-BlockX-Labs"),
    authoring_version: 1,
    // Per convention: if the runtime behavior changes, increment spec_version
    // and set impl_version to equal spec_version. If only runtime
    // implementation changes and behavior does not, then leave spec_version as
    // is and increment impl_version.
    spec_version: 2,
    impl_version: 2,
    apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
    type Call = Call;
    type Version = Version;
    type AccountId = AccountId;
    type Lookup = Indices;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type ModuleToIndex = ModuleToIndex;
    type AccountData = balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl utility::Trait for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type MultisigDepositFactor = ();
    type MaxSignatories = ();
    type MultisigDepositBase = ();
}

parameter_types! {
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
    type Moment = Moment;
    type OnTimestampSet = Babe;
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 5;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl babe::Trait for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = babe::ExternalTrigger;
}

impl authorship::Trait for Runtime {
    type FindAuthor = session::FindAccountFromAuthorIndex<Self, Babe>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = (Staking, ImOnline);
}

impl indices::Trait for Runtime {
    type AccountIndex = AccountIndex;
    type Event = Event;
    type Currency = Balances;
    type Deposit = ExistentialDeposit;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1 * CENTS;
    pub const TransferFee: Balance = 0_1 * CENTS;
    pub const CreationFee: Balance = 1 * CENTS;

}

impl balances::Trait for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<Runtime>;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 1 * CENTS;
    pub const TransactionByteFee: Balance = 0 * CENTS;
    // setting this to zero will disable the weight fee.
    pub const WeightFeeCoefficient: Balance = 0;
    // for a sane configuration, this should always be less than `AvailableBlockRatio`.
    pub const TargetBlockFullness: Perbill = Perbill::from_percent(25);
    pub const PriceFactor: u128 = 1;
    pub const BlocksPerDay: u32 = 6 * 60 * 24;
    pub const OpenedOrdersArrayCap: u8 = 20;
    pub const ClosedOrdersArrayCap: u8 = 100;
}

pub type NegativeImbalance<T> =
    <balances::Module<T> as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
    R: balances::Trait + authorship::Trait,
    <R as system::Trait>::AccountId: From<AccountId>,
    <R as system::Trait>::AccountId: Into<AccountId>,
    <R as system::Trait>::Event: From<
        balances::RawEvent<
            <R as system::Trait>::AccountId,
            <R as balances::Trait>::Balance,
            balances::DefaultInstance,
        >,
    >,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
        let numeric_amount = amount.peek();
        let author = <authorship::Module<R>>::author();
        <balances::Module<R>>::resolve_creating(&<authorship::Module<R>>::author(), amount);
        <system::Module<R>>::deposit_event(balances::RawEvent::Deposit(author, numeric_amount));
    }
}

/// Splits fees 80/20 between treasury and block author.
pub type DealWithFees = SplitTwoWays<
    Balance,
    NegativeImbalance<Runtime>,
    _4,
    Treasury, // 4 parts (80%) goes to the treasury.
    _1,
    ToAuthor<Runtime>, // 1 part (20%) goes to the block author.
>;

impl transaction_payment::Trait for Runtime {
    type Currency = Balances;
    type OnTransactionPayment = DealWithFees;
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = LinearWeightToFee<WeightFeeCoefficient>;
    type FeeMultiplierUpdate = TargetedFeeAdjustment<TargetBlockFullness>;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub babe: Babe,
        pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl session::Trait for Runtime {
    type Event = Event;
    type ValidatorId = <Self as system::Trait>::AccountId;
    type ValidatorIdOf = staking::StashOf<Self>;
    type ShouldEndSession = Babe;
    type SessionManager = Staking;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl session::historical::Trait for Runtime {
    type FullIdentification = staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: staking::EraIndex = 24 * 28;
    pub const SlashDeferDuration: staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 20 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 6 * DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
}

// impl treasury::Trait for Runtime {
//     type Currency = Balances;
//     type ApproveOrigin = collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>;
//     type RejectOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
//     type Event = Event;
//     type ProposalRejection = Treasury;
//     type ProposalBond = ProposalBond;
//     type ProposalBondMinimum = ProposalBondMinimum;
//     type SpendPeriod = SpendPeriod;
//     type Burn = Burn;
// }

impl treasury::Trait for Runtime {
    type Currency = Balances;
    type ApproveOrigin = collective::EnsureMembers<_4, AccountId, CouncilCollective>;
    type RejectOrigin = collective::EnsureMembers<_2, AccountId, CouncilCollective>;
    type Tippers = ElectionsPhragmen;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type TipReportDepositPerByte = TipReportDepositPerByte;
    type Event = Event;
    type ProposalRejection = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
}

impl staking::Trait for Runtime {
    type Currency = Balances;
    type Time = Timestamp;
    type CurrencyToVote = CurrencyToVoteHandler;
    type Event = Event;
    type Slash = Treasury;
    type Reward = ();
    type RewardRemainder = ();
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = system::EnsureRoot<<Self as system::Trait>::AccountId>;
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
}

impl authority_discovery::Trait for Runtime {}

impl grandpa::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    pub const WindowSize: BlockNumber = 101;
    pub const ReportLatency: BlockNumber = 1000;
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * DOLLARS;
    pub const TipReportDepositPerByte: Balance = 1 * CENTS;
}

impl finality_tracker::Trait for Runtime {
    type OnFinalizationStalled = Grandpa;
    type WindowSize = WindowSize;
    type ReportLatency = ReportLatency;
}

impl sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

pub type SubmitTransaction = TransactionSubmitter<ImOnlineId, Runtime, UncheckedExtrinsic>;

parameter_types! {
    pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_BLOCKS as _;
}

impl im_online::Trait for Runtime {
    type AuthorityId = ImOnlineId;
    type Event = Event;
    type Call = Call;
    type SubmitTransaction = SubmitTransaction;
    type SessionDuration = SessionDuration;
    type ReportUnresponsiveness = Offences;
}

impl offences::Trait for Runtime {
    type Event = Event;
    type IdentificationTuple = session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

impl assets::Trait for Runtime {
    type Event = Event;
}

impl dex::Trait for Runtime {
    type Event = Event;
    type Price = u128;
    type PriceFactor = PriceFactor;
    type BlocksPerDay = BlocksPerDay;
    type OpenedOrdersArrayCap = OpenedOrdersArrayCap;
    type ClosedOrdersArrayCap = ClosedOrdersArrayCap;
}

impl system::offchain::CreateTransaction<Runtime, UncheckedExtrinsic> for Runtime {
    type Public = <Signature as traits::Verify>::Signer;
    type Signature = Signature;

    fn create_transaction<TSigner: system::offchain::Signer<Self::Public, Self::Signature>>(
        call: Call,
        public: Self::Public,
        account: AccountId,
        index: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as traits::Extrinsic>::SignaturePayload,
    )> {
        // take the biggest period possible.
        let period = BlockHashCount::get()
            .checked_next_power_of_two()
            .map(|c| c / 2)
            .unwrap_or(2) as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let tip = 0;
        let extra: SignedExtra = (
            system::CheckVersion::<Runtime>::new(),
            system::CheckGenesis::<Runtime>::new(),
            system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            system::CheckNonce::<Runtime>::from(index),
            system::CheckWeight::<Runtime>::new(),
            transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
            // Default::default(),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                debug::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = TSigner::sign(public, &raw_payload)?;
        let address = Indices::unlookup(account);
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (address, signature, extra)))
    }
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 7 * DAYS;
    pub const VotingPeriod: BlockNumber = 7 * DAYS;
    pub const EmergencyVotingPeriod: BlockNumber = 3 * HOURS;
    pub const MinimumDeposit: Balance = 1 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 8 * DAYS;
    pub const CooloffPeriod: BlockNumber = 7 * DAYS;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 10 * MILLICENTS;
}

impl democracy::Trait for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type EmergencyVotingPeriod = EmergencyVotingPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin = collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
    /// A majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin =
        collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin =
        collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin =
        collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin =
        collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type Slash = Treasury;
}

parameter_types! {
    // Minimum 100 bytes/KSM deposited (1 CENT/byte)
    pub const BasicDeposit: Balance = 10 * DOLLARS;       // 258 bytes on-chain
    pub const FieldDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
    pub const SubAccountDeposit: Balance = 2 * DOLLARS;   // 53 bytes on-chain
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;

}

impl identity::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Slashed = Treasury;
    type BasicDeposit = BasicDeposit;
    type FieldDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type RegistrarOrigin =
        collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type ForceOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type MaxAdditionalFields = MaxAdditionalFields;
}

parameter_types! {
    pub const CandidacyBond: Balance = 1 * DOLLARS;
    pub const VotingBond: Balance = 5 * CENTS;
    /// Daily council elections.
    pub const TermDuration: BlockNumber = 24 * HOURS;
    pub const DesiredMembers: u32 = 13;
    pub const DesiredRunnersUp: u32 = 7;
}

impl elections_phragmen::Trait for Runtime {
    type Event = Event;
    type Currency = Balances;
    type ChangeMembers = Council;
    type CurrencyToVote = CurrencyToVoteHandler;
    type CandidacyBond = CandidacyBond;
    type VotingBond = VotingBond;
    type TermDuration = TermDuration;
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type LoserCandidate = Treasury;
    type BadReport = Treasury;
    type KickedMember = Treasury;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = node_primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Basic stuff.
        System: system::{Module, Call, Storage, Config, Event<T>},
        Timestamp: timestamp::{Module, Call, Storage, Inherent},

        // Native currency and accounts.
        Indices: indices::{Module, Call, Storage, Event<T>, Config<T>},
        Balances: balances::{Module, Call, Storage, Event<T>, Config<T>},
        TransactionPayment: transaction_payment::{Module, Storage},

        // Randomness.
        RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},

        // PoS consensus modules.
        Session: session::{Module, Call, Storage, Event, Config<T>},
        Authorship: authorship::{Module, Call, Storage, Inherent},
        Staking: staking::{Module, Call, Storage, Event<T>, Config<T>},
        Offences: offences::{Module, Call, Storage, Event},
        Babe: babe::{Module, Call, Storage, Config, Inherent(Timestamp)},
        FinalityTracker: finality_tracker::{Module, Call, Inherent},
        Grandpa: grandpa::{Module, Call, Storage, Config, Event},
        ImOnline: im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
        AuthorityDiscovery: authority_discovery::{Module, Call, Config},
        Sudo: sudo::{Module, Call, Storage, Event<T>, Config<T>},

        // Governance stuff; uncallable initiallly
        Democracy: democracy::{Module, Call, Storage, Config, Event<T>},
        Council: collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        Treasury: treasury::{Module, Call, Storage, Event<T>},
        ElectionsPhragmen: elections_phragmen::{Module, Call, Storage, Event<T>},
		TechnicalMembership: membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        TechnicalCommittee: collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},

        // Custom modules
        Assets: assets::{Module, Call, Storage,Event<T>},
        Dex: dex::{Module,Call,Storage,Event<T>},

        // Utility module
        Utility: utility::{Module, Call, Event<T>},

		// Less simple identity module.
        Identity: identity::{Module, Call, Storage, Event<T>},
    }
);

// TODO: KP: What should be our Council Motion duration, if we need it.
parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
}
type TechnicalCollective = collective::Instance2;
impl collective::Trait<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
}

type CouncilCollective = collective::Instance1;
impl collective::Trait<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
}

impl membership::Trait<membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type RemoveOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type SwapOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type ResetOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type PrimeOrigin = collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
}

/// The type used as a helper for interpreting the sender of transactions.
pub type Context = system::ChainContext<Runtime>;

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    system::CheckVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<Runtime, Block, Context, Runtime, AllModules>;

// Implement our runtime API endpoints. This is just a bunch of proxying.
impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl block_builder_api::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            Executive::validate_transaction(tx)
        }
    }

    impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }
    }



    impl babe_primitives::BabeApi<Block> for Runtime {
        fn configuration() -> babe_primitives::BabeConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            babe_primitives::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                secondary_slots: true,
            }
        }

        fn current_epoch_start() -> babe_primitives::SlotNumber {
            Babe::current_epoch_start()
        }
    }

    impl authority_discovery_primitives::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
        UncheckedExtrinsic,
    > for Runtime {
        fn query_info(uxt: UncheckedExtrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, primitives::crypto::KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }


}
