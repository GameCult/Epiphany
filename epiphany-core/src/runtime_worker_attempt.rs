use anyhow::{Result, anyhow};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkerProcessStatus {
    Claimed,
    Active,
    TerminalResult,
    TerminalDeath,
    TerminalUnactivated,
    TerminalFailure,
}

impl WorkerProcessStatus {
    pub(crate) fn parse(value: &str) -> Result<Self> {
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

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::Active => "active",
            Self::TerminalResult => "terminal-result",
            Self::TerminalDeath => "terminal-death",
            Self::TerminalUnactivated => "terminal-unactivated",
            Self::TerminalFailure => "terminal-failure",
        }
    }

    pub(crate) fn is_live(self) -> bool {
        matches!(self, Self::Claimed | Self::Active)
    }
    pub(crate) fn is_fulfilled_terminal(self) -> bool {
        self == Self::TerminalResult
    }
    pub(crate) fn is_failed_terminal(self) -> bool {
        matches!(
            self,
            Self::TerminalDeath | Self::TerminalUnactivated | Self::TerminalFailure
        )
    }
    pub(crate) fn allows_retry(self) -> bool {
        self.is_failed_terminal()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeTypedRequestRef<'a> {
    ProposalModeling(&'a str),
    FrontierVerdictModeling(&'a str),
    FrontierResearch(&'a str),
    FrontierVerification(&'a str),
    ImaginationConsideration(&'a str),
    AdmittedModelDirection(&'a str),
}

impl<'a> RuntimeTypedRequestRef<'a> {
    pub(crate) fn request_id(self) -> &'a str {
        match self {
            Self::ProposalModeling(id)
            | Self::FrontierVerdictModeling(id)
            | Self::FrontierResearch(id)
            | Self::FrontierVerification(id)
            | Self::ImaginationConsideration(id)
            | Self::AdmittedModelDirection(id) => id,
        }
    }

    pub(crate) fn kind(self) -> &'static str {
        match self {
            Self::ProposalModeling(_) => "proposal-modeling",
            Self::FrontierVerdictModeling(_) => "frontier-verdict-modeling",
            Self::FrontierResearch(_) => "frontier-research",
            Self::FrontierVerification(_) => "frontier-verification",
            Self::ImaginationConsideration(_) => "imagination-consideration",
            Self::AdmittedModelDirection(_) => "admitted-model-direction",
        }
    }

    pub(crate) fn matches_launch(self, launch: &crate::EpiphanyRuntimeWorkerLaunchRequest) -> bool {
        match self {
            Self::ProposalModeling(id) => {
                launch.proposal_modeling_request_id.as_deref() == Some(id)
            }
            Self::FrontierVerdictModeling(id) => {
                launch.repo_frontier_modeling_request_id.as_deref() == Some(id)
            }
            Self::FrontierResearch(id) => {
                launch.repo_frontier_research_request_id.as_deref() == Some(id)
            }
            Self::FrontierVerification(id) => {
                launch.repo_frontier_verification_request_id.as_deref() == Some(id)
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

    pub(crate) fn matches_result(self, result: &crate::EpiphanyRuntimeRoleWorkerResult) -> bool {
        match self {
            Self::ProposalModeling(id) => {
                result.proposal_modeling_request_id.as_deref() == Some(id)
            }
            Self::FrontierVerdictModeling(id) => {
                result.repo_frontier_modeling_request_id.as_deref() == Some(id)
            }
            Self::FrontierResearch(id) => {
                result.repo_frontier_research_request_id.as_deref() == Some(id)
            }
            Self::FrontierVerification(id) => result.verification_request_id.as_deref() == Some(id),
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
    pub(crate) fn typed_request_ref(&self) -> Result<Option<RuntimeTypedRequestRef<'_>>> {
        let requests = [
            self.proposal_modeling_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::ProposalModeling),
            self.repo_frontier_modeling_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::FrontierVerdictModeling),
            self.repo_frontier_research_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::FrontierResearch),
            self.repo_frontier_verification_request_id
                .as_deref()
                .map(RuntimeTypedRequestRef::FrontierVerification),
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
