use subxt::{
    config::{SubstrateConfig, WithExtrinsicParams},
    events::{EventSubscription, FilterEvents},
    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
    rpc::Subscription,
    tx::{BaseExtrinsicParams, PairSigner, PlainTip, TxProgress},
    OnlineClient, PolkadotConfig,
};

pub use sp_keyring::sr25519::sr25519::Pair as Keypair;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata.scale")]
pub mod api {}

pub type AccountId = AccountId32;
pub type Api = OnlineClient<PolkadotConfig>;
pub type BlockSubscription = Subscription<Header<u32, BlakeTwo256>>;
pub type Signer = PairSigner<PolkadotConfig, Keypair>;
pub type TransactionProgress = TxProgress<PolkadotConfig, Api>;

pub type FilteredEvents<'a, T> = FilterEvents<
    'a,
    EventSubscription<
        WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
        OnlineClient<
            WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
        >,
        Subscription<Header<u32, BlakeTwo256>>,
    >,
    WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
    T,
>;
