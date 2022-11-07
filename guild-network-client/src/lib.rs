use subxt::{
    config::{SubstrateConfig, WithExtrinsicParams},
    events::{EventSubscription, FilterEvents},
    ext::sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32},
    rpc::Subscription,
    tx::{BaseExtrinsicParams, PlainTip},
    OnlineClient, PolkadotConfig,
};

pub type AccountId = AccountId32;
pub type Api = OnlineClient<PolkadotConfig>;

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
