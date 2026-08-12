use anyhow::{Result, anyhow};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerProcessStatus {
    Claimed,
    Active,
    TerminalResult,
    TerminalDeath,
    TerminalUnactivated,
    TerminalFailure,
}

impl WorkerProcessStatus {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "claimed" => Ok(Self::Claimed),
            "active" => Ok(Self::Active),
            "terminal-result" => Ok(Self::TerminalResult),
            "terminal-death" => Ok(Self::TerminalDeath),
            "terminal-unactivated" => Ok(Self::TerminalUnactivated),
            "terminal-failure" => Ok(Self::TerminalFailure),
            other => Err(anyhow!("unknown runtime worker process status {other:?}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::Active => "active",
            Self::TerminalResult => "terminal-result",
            Self::TerminalDeath => "terminal-death",
            Self::TerminalUnactivated => "terminal-unactivated",
            Self::TerminalFailure => "terminal-failure",
        }
    }

    pub fn is_live(self) -> bool {
        matches!(self, Self::Claimed | Self::Active)
    }
    pub fn is_fulfilled_terminal(self) -> bool {
        self == Self::TerminalResult
    }
    pub fn is_failed_terminal(self) -> bool {
        matches!(
            self,
            Self::TerminalDeath | Self::TerminalUnactivated | Self::TerminalFailure
        )
    }
    pub fn is_terminal(self) -> bool {
        self.is_fulfilled_terminal() || self.is_failed_terminal()
    }
    pub fn allows_retry(self) -> bool {
        self.is_failed_terminal()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeTypedRequestRef<'a> {
    ProposalModeling(&'a str),
    ImaginationConsideration(&'a str),
    AdmittedModelDirection(&'a str),
}

impl<'a> RuntimeTypedRequestRef<'a> {
    pub fn request_id(self) -> &'a str {
        match self {
            Self::ProposalModeling(id)
            | Self::ImaginationConsideration(id)
            | Self::AdmittedModelDirection(id) => id,
        }
    }

    pub fn kind(self) -> &'static str {
        match self {
            Self::ProposalModeling(_) => "proposal-modeling",
            Self::ImaginationConsideration(_) => "imagination-consideration",
            Self::AdmittedModelDirection(_) => "admitted-model-direction",
        }
    }

    pub fn matches_launch(self, launch: &crate::EpiphanyRuntimeWorkerLaunchRequest) -> bool {
        match self {
            Self::ProposalModeling(id) => {
                launch.proposal_modeling_request_id.as_deref() == Some(id)
            }
            Self::ImaginationConsideration(id) => {
                launch.imagination_consideration_request_id.as_deref() == Some(id)
            }
            Self::AdmittedModelDirection(id) => {
                launch
                    .admitted_model_direction_consideration_request_id
                    .as_deref()
                    == Some(id)
            }
        }
    }

    pub fn matches_result(self, result: &crate::EpiphanyRuntimeRoleWorkerResult) -> bool {
        match self {
            Self::ProposalModeling(id) => {
                result.proposal_modeling_request_id.as_deref() == Some(id)
            }
            Self::ImaginationConsideration(id) => {
                result.imagination_consideration_request_id.as_deref() == Some(id)
            }
            Self::AdmittedModelDirection(id) => {
                result
                    .admitted_model_direction_consideration_request_id
                    .as_deref()
                    == Some(id)
            }
        }
    }
}

impl crate::EpiphanyRuntimeWorkerLaunchRequest {
    pub fn typed_request_ref(&self) -> Result<Option<RuntimeTypedRequestRef<'_>>> {
        let requests = [
            self.proposal_modeling_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::ProposalModeling),
            self.imagination_consideration_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::ImaginationConsideration),
            self.admitted_model_direction_consideration_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::AdmittedModelDirection),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        match requests.as_slice() {
            [] => Ok(None),
            [request] => Ok(Some(*request)),
            _ => Err(anyhow!(
                "worker launch carries multiple typed request authorities"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_classes_are_exhaustive_and_disjoint() -> Result<()> {
        for value in [
            "claimed",
            "active",
            "terminal-result",
            "terminal-death",
            "terminal-unactivated",
            "terminal-failure",
        ] {
            let status = WorkerProcessStatus::parse(value)?;
            assert_eq!(status.as_str(), value);
            assert_ne!(status.is_live(), status.is_terminal());
            assert_eq!(status.allows_retry(), status.is_failed_terminal());
        }
        assert!(WorkerProcessStatus::parse("completed").is_err());
        Ok(())
    }
}
