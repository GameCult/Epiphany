//! Compatibility bridge between Epiphany's typed CultNet/CultCache-shaped
//! contracts and the vendored Codex JSON-RPC/app-server shell.
//!
//! This crate is a bridge, not an authority organ. Its job is to translate
//! Codex `ThreadEpiphany*` DTOs into Epiphany core documents, project typed
//! Epiphany surfaces back into legacy Codex responses, and call the narrow
//! host trait needed while Codex still owns thread persistence. Verdicts,
//! launch policy, reorientation policy, acceptance rules, runtime-spine
//! lifecycle semantics, and durable state invariants belong in `epiphany-core`
//! or `epiphany-state-model`.
//!
//! Host facts and side effects that still pass through this crate are
//! quarantine scaffolding for the current Codex shell. Do not let them become
//! a second policy throne. The long cut is to replace these adapters with
//! native CultNet intents and receipts, leaving Codex only as the OpenAI
//! subscription auth/model transport reliquary.

pub mod checkpoint;
pub mod context;
pub mod context_protocol;
pub mod coordinator;
pub mod coordinator_protocol;
pub mod cultnet;
pub mod error;
pub mod invalidation;
pub mod jobs;
pub mod launch;
pub mod launch_context;
pub mod mutation;
pub mod mutation_service;
pub mod pressure;
pub mod protocol_edge;
pub mod reorient;
pub mod results;
pub mod retrieve;
pub mod runtime_results;
pub mod scene;
pub mod scene_protocol;
pub mod state;
pub mod view_protocol;
