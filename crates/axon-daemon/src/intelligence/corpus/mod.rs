pub mod corpus_ingestor;
pub mod entropy_profiler;
pub mod mutation_campaign;
pub mod corpus_governance;
pub mod divergence_cluster;

// P5-8h.1: Physical Corpus Ingestion Runtime
pub mod repo_fetcher;
pub mod workspace_materializer;
pub mod corpus_executor;
pub mod entropy_snapshot_store;
pub mod catastrophe_archive;

// P5-8h.2: Deterministic Corpus Campaign Runner
pub mod campaign_manifest;
pub mod replay_seed;
pub mod failure_lineage;
pub mod failure_classifier;
pub mod campaign_runner;
pub mod corpus_fingerprint;
pub mod xchat_hotspot;
pub mod corpus_seal;
pub mod runtime_adjacency;
pub mod physical_mount;
pub mod hierarchical_topology;
pub mod rox_filer_hotspot;
