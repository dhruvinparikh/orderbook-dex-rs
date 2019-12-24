//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

#![warn(unused_extern_crates)]

use std::sync::Arc;
use client::{self, LongestChain};
use grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider};
use node_executor::Executor;
use node_runtime::{GenesisConfig, RuntimeApi};
use node_primitives::Block;
use sc_service::{
    AbstractService, ServiceBuilder, config::Configuration, error::{Error as ServiceError},
};
use network::construct_simple_protocol;
use inherents::InherentDataProviders;

construct_simple_protocol! {
    /// DNA protocol attachment for substrate.
    pub struct NodeProtocol where Block = Block { }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
    ($config:expr) => {{
        let mut import_setup = None;
        let inherent_data_providers = inherents::InherentDataProviders::new();

        let builder = sc_service::ServiceBuilder::new_full::<
            node_primitives::Block, node_runtime::RuntimeApi, node_executor::Executor
        >($config)?
            .with_select_chain(|_config, backend| {
                Ok(client::LongestChain::new(backend.clone()))
            })?
            .with_transaction_pool(|config, client, _fetcher| {
                let pool_api = txpool::FullChainApi::new(client.clone());
                let pool = txpool::BasicPool::new(config, pool_api);
                let maintainer = txpool::FullBasicPoolMaintainer::new(pool.pool().clone(), client);
                let maintainable_pool = txpool_api::MaintainableTransactionPool::new(pool, maintainer);
                Ok(maintainable_pool)
            })?
            .with_import_queue(|_config, client, mut select_chain, _transaction_pool| {
                let select_chain = select_chain.take()
                    .ok_or_else(|| sc_service::Error::SelectChainRequired)?;
                let (grandpa_block_import, grandpa_link) =
                    grandpa::block_import(
                        client.clone(),
                        &*client,
                        select_chain
                    )?;
                let justification_import = grandpa_block_import.clone();

                let (babe_block_import, babe_link) = babe::block_import(
                    babe::Config::get_or_compute(&*client)?,
                    grandpa_block_import,
                    client.clone(),
                    client.clone(),
                )?;

                let import_queue = babe::import_queue(
                    babe_link.clone(),
                    babe_block_import.clone(),
                    Some(Box::new(justification_import)),
                    None,
                    client.clone(),
                    client,
                    inherent_data_providers.clone(),
                )?;

                import_setup = Some((babe_block_import, grandpa_link, babe_link));
                Ok(import_queue)
            })?;

        (builder, import_setup, inherent_data_providers)
    }}
}

/// Creates a full service from the configuration.
///
/// We need to use a macro because the test suit doesn't work with an opaque service. It expects
/// concrete types instead.
macro_rules! new_full {
    ($config:expr) => {{
        let (
            name,
            impl_name,
            impl_version,
            is_authority,
            force_authoring,
            disable_grandpa,
            sentry_nodes,
            chain_spec,
        ) = (
            $config.name.clone(),
            $config.impl_name.clone(),
            $config.impl_version.clone(),
            $config.roles.is_authority(),
            $config.force_authoring,
            $config.disable_grandpa,
            $config.network.sentry_nodes.clone(),
            $config.chain_spec.clone(),
        );
        use futures01::sync::mpsc;
        use network::DhtEvent;
        use futures::{
            compat::Stream01CompatExt,
            stream::StreamExt,
            future::{FutureExt, TryFutureExt},
        };

        // sentry nodes announce themselves as authorities to the network
        // and should run the same protocols authorities do, but it should
        // never actively participate in any consensus process.
        let participates_in_consensus = is_authority && !$config.sentry_mode;

        let (builder, mut import_setup, inherent_data_providers) = new_full_start!($config);

        // Dht event channel from the network to the authority discovery module. Use bounded channel to ensure
        // back-pressure. Authority discovery is triggering one event per authority within the current authority set.
        // This estimates the authority set size to be somewhere below 10000 thereby setting the channel buffer size to
        // 10 000.
        let (dht_event_tx, dht_event_rx) = mpsc::channel::<DhtEvent>(10_000);

        let service = builder.with_network_protocol(|_| Ok(crate::service::NodeProtocol::new()))?
            .with_finality_proof_provider(|client, backend|
                Ok(Arc::new(grandpa::FinalityProofProvider::new(backend, client)) as _)
            )?
            .with_dht_event_tx(dht_event_tx)?
            .build()?;

        let (block_import, grandpa_link, babe_link) = import_setup.take()
                .expect("Link Half and Block Import are present for Full Services or setup failed before. qed");

        if participates_in_consensus {
            let proposer = basic_authorship::ProposerFactory {
                client: service.client(),
                transaction_pool: service.transaction_pool(),
            };

            let client = service.client();
            let select_chain = service.select_chain()
                .ok_or(sc_service::Error::SelectChainRequired)?;

            let can_author_with =
                consensus_common::CanAuthorWithNativeVersion::new(client.executor().clone());

            let babe_config = babe::BabeParams {
                keystore: service.keystore(),
                client,
                select_chain,
                env: proposer,
                block_import,
                sync_oracle: service.network(),
                inherent_data_providers: inherent_data_providers.clone(),
                force_authoring,
                babe_link,
                can_author_with,
            };

            let babe = babe::start_babe(babe_config)?;
            service.spawn_essential_task(babe);

            let future03_dht_event_rx = dht_event_rx.compat()
                .map(|x| x.expect("<mpsc::channel::Receiver as Stream> never returns an error; qed"))
                .boxed();
            let authority_discovery = authority_discovery::AuthorityDiscovery::new(
                service.client(),
                service.network(),
                sentry_nodes,
                service.keystore(),
                future03_dht_event_rx,
            );
            let future01_authority_discovery = authority_discovery.map(|x| Ok(x)).compat();
            service.spawn_task(future01_authority_discovery);
        }

        // if the node isn't actively participating in consensus then it doesn't
        // need a keystore, regardless of which protocol we use below.
        let keystore = if participates_in_consensus {
            Some(service.keystore())
        } else {
            None
        };

        let config = grandpa::Config {
            // FIXME #1578 make this available through chainspec
            gossip_duration: std::time::Duration::from_millis(333),
            justification_period: 512,
            name: Some(name),
            observer_enabled: true,
            keystore,
            is_authority,
        };

        match (is_authority, disable_grandpa) {
            (false, false) => {
                // start the lightweight GRANDPA observer
                service.spawn_task(grandpa::run_grandpa_observer(
                    config,
                    grandpa_link,
                    service.network(),
                    service.on_exit(),
                    service.spawn_task_handle(),
                )?);
            },
            (true, false) => {
                // start the full GRANDPA voter
                let grandpa_config = grandpa::GrandpaParams {
                    config: config,
                    link: grandpa_link,
                    network: service.network(),
                    inherent_data_providers: inherent_data_providers.clone(),
                    on_exit: service.on_exit(),
                    telemetry_on_connect: Some(service.telemetry_on_connect_stream()),
                    voting_rule: grandpa::VotingRulesBuilder::default().build(),
                    executor: service.spawn_task_handle(),
                };
                // the GRANDPA voter task is considered infallible, i.e.
                // if it fails we take down the service with it.
                service.spawn_essential_task(grandpa::run_grandpa_voter(grandpa_config)?);
            },
            (_, true) => {
                grandpa::setup_disabled_grandpa(
                    service.client(),
                    &inherent_data_providers,
                    service.network(),
                )?;
            },
        }

        Ok((service, inherent_data_providers))
    }}
}

/// Builds a new service for a full client.
pub fn new_full<C: Send + Default + 'static>(
    config: Configuration<C, GenesisConfig>
) -> Result<impl AbstractService, ServiceError> {
    new_full!(config).map(|(service, _)| service)
}

/// Builds a new service for a light client.
pub fn new_light<C: Send + Default + 'static>(
    config: Configuration<C, GenesisConfig>
) -> Result<impl AbstractService, ServiceError> {

    let inherent_data_providers = InherentDataProviders::new();

    let service = ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
        .with_select_chain(|_config, backend| {
            Ok(LongestChain::new(backend.clone()))
        })?
        .with_transaction_pool(|config, client, fetcher| {
            let fetcher = fetcher
                .ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;
            let pool_api = txpool::LightChainApi::new(client.clone(), fetcher.clone());
            let pool = txpool::BasicPool::new(config, pool_api);
            let maintainer = txpool::LightBasicPoolMaintainer::with_defaults(pool.pool().clone(), client, fetcher);
            let maintainable_pool = txpool_api::MaintainableTransactionPool::new(pool, maintainer);
            Ok(maintainable_pool)
        })?
        .with_import_queue_and_fprb(|_config, client, backend, fetcher, _select_chain, _transaction_pool| {
            let fetch_checker = fetcher
                .map(|fetcher| fetcher.checker().clone())
                .ok_or_else(|| "Trying to start light import queue without active fetch checker")?;
            let grandpa_block_import = grandpa::light_block_import::<_, _, _, RuntimeApi>(
                client.clone(),
                backend,
                &*client,
                Arc::new(fetch_checker),
            )?;

            let finality_proof_import = grandpa_block_import.clone();
            let finality_proof_request_builder =
                finality_proof_import.create_finality_proof_request_builder();

            let (babe_block_import, babe_link) = babe::block_import(
                babe::Config::get_or_compute(&*client)?,
                grandpa_block_import,
                client.clone(),
                client.clone(),
            )?;

            let import_queue = babe::import_queue(
                babe_link,
                babe_block_import,
                None,
                Some(Box::new(finality_proof_import)),
                client.clone(),
                client,
                inherent_data_providers.clone(),
            )?;

            Ok((import_queue, finality_proof_request_builder))
        })?
        .with_network_protocol(|_| Ok(NodeProtocol::new()))?
        .with_finality_proof_provider(|client, backend|
            Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, client)) as _)
        )?
        .build()?;

    Ok(service)
}
