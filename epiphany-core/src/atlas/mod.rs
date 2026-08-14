//! Atlas is a read-only federated Modeling projection.
//!
//! Repository identities sign only documents owned by their local repository.
//! Providers advertise surfaces, consumers declare reliance and impact, and
//! consumer Soul publishes exact-edge verification. Atlas validates those
//! publications and derives joins, cycles, and blast radius. It owns no edge,
//! grants no capability, and writes no foreign Mind state.

mod contracts;
mod eve_surface;
mod identity;
mod impact_ingress;
mod planners;
mod projector;
mod publisher;
mod runtime;
mod store;
mod transport;

pub use contracts::*;
pub use eve_surface::*;
pub use identity::*;
pub use impact_ingress::*;
pub use planners::*;
pub use projector::*;
pub use publisher::*;
pub use runtime::*;
pub use store::*;
pub use transport::*;

#[cfg(test)]
mod tests;
