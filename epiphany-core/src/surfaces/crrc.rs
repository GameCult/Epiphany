#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCrrcStateStatus {
    Missing,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCrrcReorientAction {
    Resume,
    Regather,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCrrcResultStatus {
    MissingState,
    MissingBinding,
    BackendUnavailable,
    BackendMissing,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCrrcAction {
    Continue,
    PrepareCheckpoint,
    LaunchReorientWorker,
    WaitForReorientWorker,
    ReviewReorientResult,
    AcceptReorientResult,
    RegatherManually,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCrrcSceneAction {
    Update,
    Reorient,
    ReorientLaunch,
    ReorientResult,
    ReorientAccept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCrrcRecommendation {
    pub action: EpiphanyCrrcAction,
    pub recommended_scene_action: Option<EpiphanyCrrcSceneAction>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpiphanyCrrcInput {
    pub loaded: bool,
    pub state_status: EpiphanyCrrcStateStatus,
    pub should_prepare_compaction: bool,
    pub reorient_action: EpiphanyCrrcReorientAction,
    pub result_status: EpiphanyCrrcResultStatus,
    pub checkpoint_present: bool,
    pub finding_present: bool,
    pub finding_accepted: bool,
}

pub fn recommend_crrc_action(input: EpiphanyCrrcInput) -> EpiphanyCrrcRecommendation {
    let build = |action: EpiphanyCrrcAction,
                 recommended_scene_action: Option<EpiphanyCrrcSceneAction>,
                 reason: &str| EpiphanyCrrcRecommendation {
        action,
        recommended_scene_action,
        reason: reason.to_string(),
    };

    if input.state_status == EpiphanyCrrcStateStatus::Missing {
        return build(
            EpiphanyCrrcAction::RegatherManually,
            None,
            "No authoritative Epiphany state exists; re-gather source context before editing.",
        );
    }

    if !input.checkpoint_present {
        return build(
            EpiphanyCrrcAction::PrepareCheckpoint,
            Some(EpiphanyCrrcSceneAction::Update),
            "Epiphany state exists, but CRRC has no durable investigation checkpoint to resume.",
        );
    }

    match input.result_status {
        EpiphanyCrrcResultStatus::Pending | EpiphanyCrrcResultStatus::Running => {
            return build(
                EpiphanyCrrcAction::WaitForReorientWorker,
                Some(EpiphanyCrrcSceneAction::ReorientResult),
                "A reorientation worker is already in flight; wait or read the bound result.",
            );
        }
        EpiphanyCrrcResultStatus::Completed => {
            if input.finding_present && !input.finding_accepted {
                return build(
                    EpiphanyCrrcAction::AcceptReorientResult,
                    Some(EpiphanyCrrcSceneAction::ReorientAccept),
                    "A completed reorientation finding is available; review and explicitly accept it before continuing.",
                );
            }
            if input.finding_accepted
                && input.reorient_action == EpiphanyCrrcReorientAction::Regather
            {
                if input.loaded {
                    return build(
                        EpiphanyCrrcAction::LaunchReorientWorker,
                        Some(EpiphanyCrrcSceneAction::ReorientLaunch),
                        "The accepted reorientation finding is stale against the current regather checkpoint; launch a fresh bounded worker before implementation continues.",
                    );
                }
                return build(
                    EpiphanyCrrcAction::RegatherManually,
                    Some(EpiphanyCrrcSceneAction::Reorient),
                    "The accepted reorientation finding is stale against the current regather checkpoint, but the thread is not loaded.",
                );
            }
            if input.finding_accepted {
                return build(
                    EpiphanyCrrcAction::Continue,
                    Some(EpiphanyCrrcSceneAction::Reorient),
                    "The reorientation finding is already accepted and the checkpoint remains resume-ready; continue the bounded task.",
                );
            }
            if input.finding_present {
                return build(
                    EpiphanyCrrcAction::ReviewReorientResult,
                    Some(EpiphanyCrrcSceneAction::ReorientResult),
                    "A completed reorientation finding is available, but it has not been accepted yet.",
                );
            }
            return build(
                EpiphanyCrrcAction::ReviewReorientResult,
                Some(EpiphanyCrrcSceneAction::ReorientResult),
                "The reorientation worker completed, but no structured finding was recorded.",
            );
        }
        EpiphanyCrrcResultStatus::Failed | EpiphanyCrrcResultStatus::Cancelled => {
            return build(
                EpiphanyCrrcAction::RegatherManually,
                Some(EpiphanyCrrcSceneAction::ReorientResult),
                "The reorientation worker ended without a usable finding; inspect the failure before relaunching.",
            );
        }
        EpiphanyCrrcResultStatus::MissingState => {
            return build(
                EpiphanyCrrcAction::RegatherManually,
                None,
                "No authoritative Epiphany state exists; re-gather source context before editing.",
            );
        }
        EpiphanyCrrcResultStatus::MissingBinding
        | EpiphanyCrrcResultStatus::BackendUnavailable
        | EpiphanyCrrcResultStatus::BackendMissing => {}
    }

    if input.should_prepare_compaction
        || input.reorient_action == EpiphanyCrrcReorientAction::Regather
    {
        if input.loaded {
            return build(
                EpiphanyCrrcAction::LaunchReorientWorker,
                Some(EpiphanyCrrcSceneAction::ReorientLaunch),
                "The current pressure/reorientation verdict needs a bounded worker before safe continuation.",
            );
        }
        return build(
            EpiphanyCrrcAction::RegatherManually,
            Some(EpiphanyCrrcSceneAction::Reorient),
            "The thread is not loaded, so CRRC can only report the regather verdict.",
        );
    }

    build(
        EpiphanyCrrcAction::Continue,
        Some(EpiphanyCrrcSceneAction::Reorient),
        "Pressure is tolerable and the checkpoint remains resume-ready; continue the bounded task.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> EpiphanyCrrcInput {
        EpiphanyCrrcInput {
            loaded: true,
            state_status: EpiphanyCrrcStateStatus::Ready,
            should_prepare_compaction: false,
            reorient_action: EpiphanyCrrcReorientAction::Resume,
            result_status: EpiphanyCrrcResultStatus::MissingBinding,
            checkpoint_present: true,
            finding_present: false,
            finding_accepted: false,
        }
    }

    #[test]
    fn continues_clean_checkpoint() {
        let recommendation = recommend_crrc_action(input());

        assert_eq!(recommendation.action, EpiphanyCrrcAction::Continue);
        assert_eq!(
            recommendation.recommended_scene_action,
            Some(EpiphanyCrrcSceneAction::Reorient)
        );
    }

    #[test]
    fn launches_worker_for_regather_verdict() {
        let recommendation = recommend_crrc_action(EpiphanyCrrcInput {
            reorient_action: EpiphanyCrrcReorientAction::Regather,
            ..input()
        });

        assert_eq!(
            recommendation.action,
            EpiphanyCrrcAction::LaunchReorientWorker
        );
        assert_eq!(
            recommendation.recommended_scene_action,
            Some(EpiphanyCrrcSceneAction::ReorientLaunch)
        );
    }

    #[test]
    fn accepts_unaccepted_completed_finding() {
        let recommendation = recommend_crrc_action(EpiphanyCrrcInput {
            result_status: EpiphanyCrrcResultStatus::Completed,
            finding_present: true,
            ..input()
        });

        assert_eq!(
            recommendation.action,
            EpiphanyCrrcAction::AcceptReorientResult
        );
    }

    #[test]
    fn relaunches_stale_accepted_regather() {
        let recommendation = recommend_crrc_action(EpiphanyCrrcInput {
            reorient_action: EpiphanyCrrcReorientAction::Regather,
            result_status: EpiphanyCrrcResultStatus::Completed,
            finding_present: true,
            finding_accepted: true,
            ..input()
        });

        assert_eq!(
            recommendation.action,
            EpiphanyCrrcAction::LaunchReorientWorker
        );
    }
}
