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

    pub(crate) fn matches_launch(
        self,
        launch: &crate::EpiphanyRuntimeWorkerLaunchRequest,
    ) -> Result<bool> {
        let document = launch.launch_document()?;
        Ok(document.typed_request_ref()? == Some(self))
    }
}

impl crate::EpiphanyWorkerLaunchDocument {
    pub(crate) fn typed_request_ref(&self) -> Result<Option<RuntimeTypedRequestRef<'_>>> {
        let crate::EpiphanyWorkerLaunchDocument::Role(document) = self else {
            return Ok(None);
        };
        let requests = [
            document
                .proposal_modeling_context
                .as_ref()
                .map(|context| context.request_id.as_str())
                .map(RuntimeTypedRequestRef::ProposalModeling),
            document
                .frontier_verdict_modeling_context
                .as_ref()
                .map(|context| context.request.request_id.as_str())
                .map(RuntimeTypedRequestRef::FrontierVerdictModeling),
            document
                .frontier_research_context
                .as_ref()
                .map(|context| context.request_id.as_str())
                .map(RuntimeTypedRequestRef::FrontierResearch),
            document
                .frontier_verification_context
                .as_ref()
                .map(|context| context.request.request_id.as_str())
                .map(RuntimeTypedRequestRef::FrontierVerification),
            document
                .imagination_consideration_context
                .as_ref()
                .map(|context| context.request.request_id.as_str())
                .map(RuntimeTypedRequestRef::ImaginationConsideration),
            document
                .admitted_model_direction_consideration_context
                .as_ref()
                .map(|context| context.request.request_id.as_str())
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
