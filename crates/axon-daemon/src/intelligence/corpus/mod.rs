pub(crate) mod corpus_ingestor;
pub(crate) mod entropy_profiler;
pub(crate) mod mutation_campaign;
pub(crate) mod corpus_governance;
pub(crate) mod divergence_cluster;

// P5-8h.1: Physical Corpus Ingestion Runtime
pub(crate) mod repo_fetcher;
pub(crate) mod workspace_materializer;
pub(crate) mod corpus_executor;
pub(crate) mod entropy_snapshot_store;
pub(crate) mod catastrophe_archive;

// P5-8h.2: Deterministic Corpus Campaign Runner
pub(crate) mod campaign_manifest;
pub(crate) mod replay_seed;
pub(crate) mod failure_lineage;
pub(crate) mod failure_classifier;
pub(crate) mod campaign_runner;
pub(crate) mod corpus_fingerprint;
pub(crate) mod xchat_hotspot;
pub mod corpus_seal;
pub mod runtime_adjacency;
pub(crate) mod physical_mount;
pub(crate) mod hierarchical_topology;
pub(crate) mod rox_filer_hotspot;
