use super::MemorySemanticProjectionInput;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemorySemanticProjectorPulseClassification {
    Pending,
    Failed,
    Repair,
    Ready,
    Running,
    RunningOwned { claim_id: String },
    Stale,
}

impl MemorySemanticProjectorPulseClassification {
    fn automatic_purpose(&self) -> Option<&'static str> {
        match self {
            Self::Pending | Self::Failed => Some("execute"),
            Self::Repair => Some("repair"),
            Self::Ready | Self::Running | Self::RunningOwned { .. } | Self::Stale => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySemanticProjectorPulseInspection {
    pub scope_id: String,
    pub classification: Option<MemorySemanticProjectorPulseClassification>,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemorySemanticProjectorPulseStatus {
    Idle,
    Executed,
    Contended,
    Refused,
    Busy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySemanticProjectorPulseOutcome {
    pub status: MemorySemanticProjectorPulseStatus,
    pub selected_scope_id: Option<String>,
    pub inspections: Vec<MemorySemanticProjectorPulseInspection>,
    pub error: Option<String>,
}

pub(crate) trait MemorySemanticProjectorPulsePort {
    fn classify(
        &self,
        input: &MemorySemanticProjectionInput,
    ) -> Result<MemorySemanticProjectorPulseClassification>;

    /// `Ok(None)` means another pulse won the exact acquisition CAS.
    fn acquire(
        &self,
        input: &MemorySemanticProjectionInput,
        purpose: &str,
    ) -> Result<Option<String>>;

    fn execute(&self, input: &MemorySemanticProjectionInput, claim_id: &str) -> Result<()>;
}

pub(crate) struct MemorySemanticProjectorPulser<P> {
    pub(crate) port: P,
    active: AtomicBool,
}

impl<P> MemorySemanticProjectorPulser<P>
where
    P: MemorySemanticProjectorPulsePort,
{
    pub(crate) fn new(port: P) -> Self {
        Self {
            port,
            active: AtomicBool::new(false),
        }
    }

    pub(crate) fn pulse(
        &self,
        sealed_inputs: &[MemorySemanticProjectionInput],
        fairness_cursor: Option<&str>,
    ) -> MemorySemanticProjectorPulseOutcome {
        if self
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return outcome(MemorySemanticProjectorPulseStatus::Busy, vec![], None, None);
        }
        let _guard = PulseGuard(&self.active);
        if sealed_inputs.is_empty() {
            return outcome(MemorySemanticProjectorPulseStatus::Idle, vec![], None, None);
        }
        let scopes = sealed_inputs.iter().map(scope_key).collect::<Vec<_>>();
        let start = fairness_cursor
            .and_then(|cursor| scopes.iter().position(|scope| scope == cursor))
            .map_or(0, |index| (index + 1) % sealed_inputs.len());
        let mut inspections = Vec::with_capacity(sealed_inputs.len());
        let mut first_source_error = None;
        for offset in 0..sealed_inputs.len() {
            let index = (start + offset) % sealed_inputs.len();
            let input = &sealed_inputs[index];
            let scope_id = scopes[index].clone();
            let classification = match self.port.classify(input) {
                Ok(classification) => classification,
                Err(error) => {
                    let error = format!("{error:#}");
                    first_source_error.get_or_insert_with(|| error.clone());
                    inspections.push(MemorySemanticProjectorPulseInspection {
                        scope_id,
                        classification: None,
                        error: Some(error),
                    });
                    continue;
                }
            };
            inspections.push(MemorySemanticProjectorPulseInspection {
                scope_id: scope_id.clone(),
                classification: Some(classification.clone()),
                error: None,
            });
            if let MemorySemanticProjectorPulseClassification::RunningOwned { claim_id } =
                classification
            {
                if let Err(error) = self.port.execute(input, &claim_id) {
                    return outcome(
                        MemorySemanticProjectorPulseStatus::Refused,
                        inspections,
                        Some(scope_id),
                        Some(format!("{error:#}")),
                    );
                }
                return outcome(
                    MemorySemanticProjectorPulseStatus::Executed,
                    inspections,
                    Some(scope_id),
                    None,
                );
            }
            let Some(purpose) = classification.automatic_purpose() else {
                continue;
            };
            let claim_id = match self.port.acquire(input, purpose) {
                Ok(Some(claim_id)) => claim_id,
                Ok(None) => {
                    return outcome(
                        MemorySemanticProjectorPulseStatus::Contended,
                        inspections,
                        Some(scope_id),
                        None,
                    );
                }
                Err(error) => {
                    return outcome(
                        MemorySemanticProjectorPulseStatus::Refused,
                        inspections,
                        Some(scope_id),
                        Some(format!("{error:#}")),
                    );
                }
            };
            if let Err(error) = self.port.execute(input, &claim_id) {
                return outcome(
                    MemorySemanticProjectorPulseStatus::Refused,
                    inspections,
                    Some(scope_id),
                    Some(format!("{error:#}")),
                );
            }
            return outcome(
                MemorySemanticProjectorPulseStatus::Executed,
                inspections,
                Some(scope_id),
                None,
            );
        }
        if let Some(error) = first_source_error {
            outcome(
                MemorySemanticProjectorPulseStatus::Refused,
                inspections,
                None,
                Some(error),
            )
        } else {
            outcome(
                MemorySemanticProjectorPulseStatus::Idle,
                inspections,
                None,
                None,
            )
        }
    }
}

struct PulseGuard<'a>(&'a AtomicBool);

impl Drop for PulseGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

fn scope_key(input: &MemorySemanticProjectionInput) -> String {
    format!(
        "{}:{}",
        input.obligation().swarm_id,
        input.obligation().partition
    )
}

fn outcome(
    status: MemorySemanticProjectorPulseStatus,
    inspections: Vec<MemorySemanticProjectorPulseInspection>,
    selected_scope_id: Option<String>,
    error: Option<String>,
) -> MemorySemanticProjectorPulseOutcome {
    MemorySemanticProjectorPulseOutcome {
        status,
        selected_scope_id,
        inspections,
        error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::semantic_projector::MemorySemanticProjectionAuthoritySnapshot;
    use crate::memory_graph::semantic_projector::{
        classify_memory_semantic_projection_for_pulse, semantic_projector_cache,
    };
    use crate::{
        MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION, MemorySemanticProjectionObligation,
        MemorySemanticProjectionSourceHead, SEMANTIC_PROJECTION_SCHEMA_VERSION,
    };
    use cultcache_rs::CultCache;
    use std::sync::{Arc, Barrier, Mutex};
    use tempfile::tempdir;

    fn input(partition: &str) -> MemorySemanticProjectionInput {
        let obligation = MemorySemanticProjectionObligation {
            schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
            obligation_id: format!("obligation-{partition}"),
            swarm_id: "swarm-a".to_string(),
            partition: partition.to_string(),
            canonical_source_id: format!("source-{partition}"),
            source_commit_id: "commit-1".to_string(),
            graph_id: format!("graph-{partition}"),
            source_generation: 1,
            source_model_hash: "model-hash".to_string(),
            canonical_content_set_hash: "content-hash".to_string(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            created_at: "2026-07-15T10:00:00Z".to_string(),
        };
        MemorySemanticProjectionInput {
            snapshot: crate::EpiphanyMemoryGraphSnapshot {
                schema_version: Some("v0".to_string()),
                graph_id: obligation.graph_id.clone(),
                model_revision: 1,
                ..Default::default()
            },
            authority: MemorySemanticProjectionAuthoritySnapshot {
                head: MemorySemanticProjectionSourceHead {
                    swarm_id: obligation.swarm_id.clone(),
                    partition: obligation.partition.clone(),
                    canonical_source_id: obligation.canonical_source_id.clone(),
                    source_commit_id: obligation.source_commit_id.clone(),
                    graph_id: obligation.graph_id.clone(),
                    source_generation: 1,
                    source_model_hash: obligation.source_model_hash.clone(),
                    canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
                },
                envelopes: vec![],
            },
            obligation,
        }
    }

    struct MockPort {
        classifications: Mutex<
            Vec<std::result::Result<MemorySemanticProjectorPulseClassification, &'static str>>,
        >,
        acquire_result: Mutex<Option<Option<String>>>,
        acquisitions: Mutex<Vec<(String, String)>>,
        executions: Mutex<Vec<String>>,
        execute_barriers: Option<(Arc<Barrier>, Arc<Barrier>)>,
    }

    impl MockPort {
        fn new(classifications: Vec<MemorySemanticProjectorPulseClassification>) -> Self {
            Self {
                classifications: Mutex::new(classifications.into_iter().map(Ok).collect()),
                acquire_result: Mutex::new(Some(Some("claim-1".to_string()))),
                acquisitions: Mutex::new(vec![]),
                executions: Mutex::new(vec![]),
                execute_barriers: None,
            }
        }
    }

    impl MemorySemanticProjectorPulsePort for MockPort {
        fn classify(
            &self,
            _input: &MemorySemanticProjectionInput,
        ) -> Result<MemorySemanticProjectorPulseClassification> {
            self.classifications
                .lock()
                .unwrap()
                .remove(0)
                .map_err(anyhow::Error::msg)
        }

        fn acquire(
            &self,
            input: &MemorySemanticProjectionInput,
            purpose: &str,
        ) -> Result<Option<String>> {
            self.acquisitions
                .lock()
                .unwrap()
                .push((scope_key(input), purpose.to_string()));
            Ok(self.acquire_result.lock().unwrap().take().unwrap_or(None))
        }

        fn execute(&self, _input: &MemorySemanticProjectionInput, claim_id: &str) -> Result<()> {
            self.executions.lock().unwrap().push(claim_id.to_string());
            if let Some((entered, release)) = &self.execute_barriers {
                entered.wait();
                release.wait();
            }
            Ok(())
        }
    }

    #[test]
    fn decision_table_never_runs_ready_running_or_stale() {
        for classification in [
            MemorySemanticProjectorPulseClassification::Ready,
            MemorySemanticProjectorPulseClassification::Running,
            MemorySemanticProjectorPulseClassification::Stale,
        ] {
            let pulser = MemorySemanticProjectorPulser::new(MockPort::new(vec![classification]));
            let outcome = pulser.pulse(&[input("mind")], None);
            assert_eq!(outcome.status, MemorySemanticProjectorPulseStatus::Idle);
            assert!(pulser.port.acquisitions.lock().unwrap().is_empty());
            assert!(pulser.port.executions.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn repair_classification_acquires_the_typed_repair_path() {
        let pulser = MemorySemanticProjectorPulser::new(MockPort::new(vec![
            MemorySemanticProjectorPulseClassification::Repair,
        ]));
        let outcome = pulser.pulse(&[input("modeling")], None);
        assert_eq!(outcome.status, MemorySemanticProjectorPulseStatus::Executed);
        assert_eq!(
            pulser.port.acquisitions.lock().unwrap().as_slice(),
            &[("swarm-a:modeling".to_string(), "repair".to_string())]
        );
    }

    #[test]
    fn native_classification_refuses_an_advanced_sealed_input_as_stale() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("classification.msgpack");
        let current = input("mind");
        let mut cache: CultCache = semantic_projector_cache(&store)?;
        cache.put(&current.obligation().obligation_id, current.obligation())?;
        assert_eq!(
            classify_memory_semantic_projection_for_pulse(&store, &current)?,
            MemorySemanticProjectorPulseClassification::Pending
        );
        let mut stale = current.clone();
        stale.obligation.source_generation += 1;
        assert_eq!(
            classify_memory_semantic_projection_for_pulse(&store, &stale)?,
            MemorySemanticProjectorPulseClassification::Stale
        );
        Ok(())
    }

    #[test]
    fn pending_and_failed_select_one_global_action_with_rotating_fairness() {
        for classification in [
            MemorySemanticProjectorPulseClassification::Pending,
            MemorySemanticProjectorPulseClassification::Failed,
        ] {
            let pulser = MemorySemanticProjectorPulser::new(MockPort::new(vec![classification]));
            let outcome = pulser.pulse(&[input("modeling")], None);
            assert_eq!(outcome.status, MemorySemanticProjectorPulseStatus::Executed);
            assert_eq!(pulser.port.acquisitions.lock().unwrap().len(), 1);
            assert_eq!(pulser.port.executions.lock().unwrap().len(), 1);
        }

        let pulser = MemorySemanticProjectorPulser::new(MockPort::new(vec![
            MemorySemanticProjectorPulseClassification::Pending,
        ]));
        let candidates = [input("mind"), input("modeling")];
        let outcome = pulser.pulse(&candidates, Some("swarm-a:mind"));
        assert_eq!(
            outcome.selected_scope_id.as_deref(),
            Some("swarm-a:modeling")
        );
        assert_eq!(outcome.inspections.len(), 1);
        assert_eq!(pulser.port.executions.lock().unwrap().len(), 1);
    }

    #[test]
    fn contention_consumes_the_one_action_slot_without_false_execution() {
        let port = MockPort::new(vec![MemorySemanticProjectorPulseClassification::Pending]);
        *port.acquire_result.lock().unwrap() = Some(None);
        let pulser = MemorySemanticProjectorPulser::new(port);
        let outcome = pulser.pulse(&[input("mind"), input("modeling")], None);
        assert_eq!(
            outcome.status,
            MemorySemanticProjectorPulseStatus::Contended
        );
        assert_eq!(outcome.inspections.len(), 1);
        assert!(pulser.port.executions.lock().unwrap().is_empty());
    }

    #[test]
    fn owned_running_claim_resumes_without_a_second_acquisition() {
        let pulser = MemorySemanticProjectorPulser::new(MockPort::new(vec![
            MemorySemanticProjectorPulseClassification::RunningOwned {
                claim_id: "recovered-claim".to_string(),
            },
        ]));
        let outcome = pulser.pulse(&[input("mind")], None);
        assert_eq!(outcome.status, MemorySemanticProjectorPulseStatus::Executed);
        assert!(pulser.port.acquisitions.lock().unwrap().is_empty());
        assert_eq!(
            pulser.port.executions.lock().unwrap().as_slice(),
            ["recovered-claim"]
        );
    }

    #[test]
    fn one_source_fault_does_not_hide_an_actionable_other_source() {
        let port = MockPort::new(vec![]);
        *port.classifications.lock().unwrap() = vec![
            Err("stale sealed source"),
            Ok(MemorySemanticProjectorPulseClassification::Pending),
        ];
        let pulser = MemorySemanticProjectorPulser::new(port);
        let outcome = pulser.pulse(&[input("mind"), input("modeling")], None);
        assert_eq!(outcome.status, MemorySemanticProjectorPulseStatus::Executed);
        assert_eq!(outcome.inspections.len(), 2);
        assert!(outcome.inspections[0].classification.is_none());
        assert_eq!(
            outcome.selected_scope_id.as_deref(),
            Some("swarm-a:modeling")
        );
        assert_eq!(pulser.port.executions.lock().unwrap().len(), 1);
    }

    #[test]
    fn overlapping_pulse_is_busy_and_cannot_execute() {
        let entered = Arc::new(Barrier::new(2));
        let release = Arc::new(Barrier::new(2));
        let mut port = MockPort::new(vec![MemorySemanticProjectorPulseClassification::Pending]);
        port.execute_barriers = Some((entered.clone(), release.clone()));
        let pulser = Arc::new(MemorySemanticProjectorPulser::new(port));
        let worker = {
            let pulser = pulser.clone();
            std::thread::spawn(move || pulser.pulse(&[input("mind")], None))
        };
        entered.wait();
        let busy = pulser.pulse(&[input("modeling")], None);
        assert_eq!(busy.status, MemorySemanticProjectorPulseStatus::Busy);
        release.wait();
        assert_eq!(
            worker.join().unwrap().status,
            MemorySemanticProjectorPulseStatus::Executed
        );
        assert_eq!(pulser.port.executions.lock().unwrap().len(), 1);
    }
}
