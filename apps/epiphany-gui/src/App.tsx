import {
  Boxes,
  AlertTriangle,
  BriefcaseBusiness,
  CheckCircle2,
  ClipboardCheck,
  Database,
  Eye,
  FileText,
  GitBranch,
  ListChecks,
  Map,
  Play,
  RefreshCw,
} from "lucide-react";
import { EpiphanyGraphViewer, validateEpiphanyGraphsState } from "@epiphanygraph/epiphany-graph-viewer";
import { useEffect, useMemo, useState } from "react";
import { loadOperatorSnapshot, runOperatorAction } from "./operatorApi";
import type { ArtifactBundle, OperatorAction, OperatorActionResult, OperatorSnapshot, StatusRequest } from "./types";
import type { EpiphanyCodeRef, EpiphanyGraphsState } from "@epiphanygraph/epiphany-graph-viewer";

const roleOrder = ["implementation", "imagination", "modeling", "verification", "reorientation"];
const actionButtons: Array<{
  action: OperatorAction;
  label: string;
  runningLabel: string;
  title: string;
  requiresThread?: boolean;
  requiresReadyState?: boolean;
  requiresImaginationPatch?: boolean;
  requiresModelingPatch?: boolean;
  requiresVerificationResult?: boolean;
  requiresReorientResult?: boolean;
  requiresPlanningDraft?: boolean;
  requiresContinueImplementation?: boolean;
  icon: "file" | "check" | "play" | "eye" | "accept" | "runtime" | "plan" | "ide";
}> = [
  {
    action: "statusSnapshot",
    label: "Status Snapshot",
    runningLabel: "Writing",
    title: "Write an auditable status snapshot",
    icon: "file",
  },
  {
    action: "coordinatorPlan",
    label: "Coordinator Plan",
    runningLabel: "Running",
    title: "Run a review-gated coordinator plan",
    icon: "check",
  },
  {
    action: "inspectUnity",
    label: "Inspect Unity",
    runningLabel: "Inspecting",
    title: "Resolve the project-pinned Unity editor and write runtime artifacts",
    icon: "runtime",
  },
  {
    action: "inspectRider",
    label: "Inspect Rider",
    runningLabel: "Inspecting",
    title: "Inspect Rider, solution, and source control status through the local bridge",
    icon: "ide",
  },
  {
    action: "prepareCheckpoint",
    label: "Prepare Checkpoint",
    runningLabel: "Preparing",
    title: "Seed durable Epiphany state for this GUI operator thread",
    icon: "accept",
  },
  {
    action: "adoptObjectiveDraft",
    label: "Adopt Draft",
    runningLabel: "Adopting",
    title: "Adopt the selected Objective Draft as the active implementation objective",
    requiresThread: true,
    requiresReadyState: true,
    requiresPlanningDraft: true,
    icon: "plan",
  },
  {
    action: "continueImplementation",
    label: "Continue Implementation",
    runningLabel: "Implementing",
    title: "Run a bounded implementation turn when the coordinator has cleared specialist lanes",
    requiresThread: true,
    requiresReadyState: true,
    requiresContinueImplementation: true,
    icon: "play",
  },
  {
    action: "launchImagination",
    label: "Launch Imagination",
    runningLabel: "Launching",
    title: "Launch the fixed imagination/planning worker for this thread",
    requiresThread: true,
    requiresReadyState: true,
    icon: "play",
  },
  {
    action: "readImaginationResult",
    label: "Read Imagination",
    runningLabel: "Reading",
    title: "Read the latest imagination/planning finding",
    requiresThread: true,
    icon: "eye",
  },
  {
    action: "acceptImagination",
    label: "Accept Imagination",
    runningLabel: "Accepting",
    title: "Accept a reviewed planning-only patch into Epiphany state",
    requiresThread: true,
    requiresReadyState: true,
    requiresImaginationPatch: true,
    icon: "accept",
  },
  {
    action: "launchModeling",
    label: "Launch Modeling",
    runningLabel: "Launching",
    title: "Launch the fixed modeling/checkpoint worker for this thread",
    requiresThread: true,
    requiresReadyState: true,
    icon: "play",
  },
  {
    action: "readModelingResult",
    label: "Read Modeling",
    runningLabel: "Reading",
    title: "Read the latest modeling/checkpoint finding",
    requiresThread: true,
    icon: "eye",
  },
  {
    action: "acceptModeling",
    label: "Accept Modeling",
    runningLabel: "Accepting",
    title: "Accept a reviewed modeling graph/checkpoint patch into Epiphany state",
    requiresThread: true,
    requiresReadyState: true,
    requiresModelingPatch: true,
    icon: "accept",
  },
  {
    action: "launchVerification",
    label: "Launch Verification",
    runningLabel: "Launching",
    title: "Launch the fixed verification/review worker for this thread",
    requiresThread: true,
    requiresReadyState: true,
    icon: "play",
  },
  {
    action: "readVerificationResult",
    label: "Read Verification",
    runningLabel: "Reading",
    title: "Read the latest verification/review finding",
    requiresThread: true,
    icon: "eye",
  },
  {
    action: "acceptVerification",
    label: "Accept Verification",
    runningLabel: "Accepting",
    title: "Accept a reviewed verification finding into Epiphany state",
    requiresThread: true,
    requiresReadyState: true,
    requiresVerificationResult: true,
    icon: "accept",
  },
  {
    action: "launchReorient",
    label: "Launch Reorient",
    runningLabel: "Launching",
    title: "Launch the bounded reorient-worker for this thread",
    requiresThread: true,
    requiresReadyState: true,
    icon: "play",
  },
  {
    action: "readReorientResult",
    label: "Read Reorient",
    runningLabel: "Reading",
    title: "Read the latest reorient-worker finding",
    requiresThread: true,
    icon: "eye",
  },
  {
    action: "acceptReorient",
    label: "Accept Reorient",
    runningLabel: "Accepting",
    title: "Accept a completed reorient-worker finding into Epiphany state",
    requiresThread: true,
    requiresReadyState: true,
    requiresReorientResult: true,
    icon: "accept",
  },
];

function text(value: unknown, fallback = "none"): string {
  if (value === null || value === undefined || value === "") {
    return fallback;
  }
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  return fallback;
}

function listText(value: unknown, fallback = "none"): string {
  return Array.isArray(value) && value.length > 0 ? value.map(String).join(", ") : fallback;
}

function objectList(value: unknown): any[] {
  return Array.isArray(value) ? value.filter((item) => item && typeof item === "object") : [];
}

function countText(value: unknown): string {
  return typeof value === "number" ? String(value) : text(value, "0");
}

function statusClass(value: unknown): string {
  const lower = text(value).toLowerCase();
  if (lower.includes("blocked") || lower.includes("critical") || lower.includes("regather")) return "danger";
  if (lower.includes("needed") || lower.includes("review") || lower.includes("prepare") || lower.includes("high")) return "warn";
  if (lower.includes("completed") || lower.includes("ready") || lower.includes("continue") || lower.includes("pass")) return "ok";
  return "neutral";
}

const emptyGraphState: EpiphanyGraphsState = {
  architecture: { nodes: [], edges: [] },
  dataflow: { nodes: [], edges: [] },
  links: [],
};

function normalizeGraphState(value: any): EpiphanyGraphsState {
  if (!value || typeof value !== "object") return emptyGraphState;
  return {
    architecture: normalizeGraph(value.architecture),
    dataflow: normalizeGraph(value.dataflow),
    links: objectList(value.links).map((link) => ({
      dataflow_node_id: text(link.dataflow_node_id ?? link.dataflowNodeId, ""),
      architecture_node_id: text(link.architecture_node_id ?? link.architectureNodeId, ""),
      relationship: link.relationship ?? null,
      code_refs: normalizeCodeRefs(link.code_refs ?? link.codeRefs),
    })).filter((link) => link.dataflow_node_id && link.architecture_node_id),
  };
}

function normalizeGraph(value: any) {
  return {
    nodes: objectList(value?.nodes).map((node) => ({
      id: text(node.id, ""),
      title: text(node.title ?? node.id, "Untitled node"),
      purpose: text(node.purpose ?? node.summary, "No purpose recorded."),
      mechanism: node.mechanism ?? null,
      metaphor: node.metaphor ?? null,
      status: node.status ?? null,
      code_refs: normalizeCodeRefs(node.code_refs ?? node.codeRefs),
    })).filter((node) => node.id),
    edges: objectList(value?.edges).map((edge) => ({
      id: edge.id ?? null,
      source_id: text(edge.source_id ?? edge.sourceId, ""),
      target_id: text(edge.target_id ?? edge.targetId, ""),
      kind: text(edge.kind, "link"),
      label: edge.label ?? null,
      mechanism: edge.mechanism ?? null,
      code_refs: normalizeCodeRefs(edge.code_refs ?? edge.codeRefs),
    })).filter((edge) => edge.source_id && edge.target_id),
  };
}

function normalizeCodeRefs(value: any): EpiphanyCodeRef[] {
  return objectList(value).map((ref) => ({
    path: text(ref.path, ""),
    start_line: typeof ref.start_line === "number" ? ref.start_line : ref.startLine,
    end_line: typeof ref.end_line === "number" ? ref.end_line : ref.endLine,
    symbol: ref.symbol ?? null,
    note: ref.note ?? null,
  })).filter((ref) => ref.path);
}

function graphRecordCount(state: EpiphanyGraphsState): number {
  return state.architecture.nodes.length + state.architecture.edges.length + state.dataflow.nodes.length + state.dataflow.edges.length + state.links.length;
}

function codeRefLabel(ref: EpiphanyCodeRef): string {
  const line = ref.start_line ? `:${ref.start_line}` : "";
  const symbol = ref.symbol ? ` ${ref.symbol}` : "";
  return `${ref.path}${line}${symbol}`;
}

function useSnapshot() {
  const [snapshot, setSnapshot] = useState<OperatorSnapshot | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [actionResult, setActionResult] = useState<OperatorActionResult | null>(null);
  const [runningAction, setRunningAction] = useState<OperatorAction | null>(null);
  const [request, setRequest] = useState<StatusRequest>({});

  async function refresh(nextRequest = request) {
    setLoading(true);
    setError(null);
    try {
      const result = await loadOperatorSnapshot(nextRequest);
      setSnapshot(result);
      const loadedThreadId = result.status?.threadId;
      const loadedState = result.status?.scene?.scene?.stateStatus;
      if (
        !nextRequest.threadId &&
        loadedState !== "missing" &&
        typeof loadedThreadId === "string" &&
        loadedThreadId.length > 0
      ) {
        setRequest((current) => (current.threadId ? current : { ...current, threadId: loadedThreadId }));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function runAction(action: OperatorAction) {
    setRunningAction(action);
    setError(null);
    setActionResult(null);
    try {
      const result = await runOperatorAction(action, request);
      setActionResult(result);
      const nextRequest = result.threadId ? { ...request, threadId: result.threadId } : request;
      if (result.threadId) {
        setRequest(nextRequest);
      }
      await refresh(nextRequest);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunningAction(null);
    }
  }

  useEffect(() => {
    void refresh({});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { snapshot, loading, error, request, setRequest, refresh, actionResult, runningAction, runAction };
}

export function App() {
  const { snapshot, loading, error, request, setRequest, refresh, actionResult, runningAction, runAction } = useSnapshot();
  const [selectedCodeRef, setSelectedCodeRef] = useState<EpiphanyCodeRef | null>(null);
  const status = snapshot?.status;
  const scene = status?.scene?.scene ?? {};
  const pressure = status?.pressure?.pressure ?? {};
  const reorient = status?.reorient?.decision ?? {};
  const crrc = status?.crrc?.recommendation ?? {};
  const coordinator = status?.coordinator ?? {};
  const roles = useMemo(() => {
    const lanes = status?.roles?.roles;
    if (!Array.isArray(lanes)) return [];
    return [...lanes].sort((a, b) => roleOrder.indexOf(text(a.id)) - roleOrder.indexOf(text(b.id)));
  }, [status]);
  const jobs: any[] = Array.isArray(status?.jobs?.jobs) ? status.jobs.jobs : [];
  const planningResponse = status?.planning ?? {};
  const planningState = planningResponse?.planning ?? {};
  const planningSummary = planningResponse?.summary ?? {};
  const planningCaptures = objectList(planningState?.captures);
  const backlogItems = objectList(planningState?.backlog_items ?? planningState?.backlogItems);
  const roadmapStreams = objectList(planningState?.roadmap_streams ?? planningState?.roadmapStreams);
  const objectiveDrafts = objectList(planningState?.objective_drafts ?? planningState?.objectiveDrafts);
  const roleResults = status?.roleResults ?? {};
  const reorientResult = status?.reorientResult ?? {};
  const latestArtifact = snapshot?.artifacts?.[0];
  const latestImplementationArtifact = useMemo(
    () => (snapshot?.artifacts ?? []).find((artifact) => artifact.implementationAudit),
    [snapshot?.artifacts],
  );
  const latestImplementationAudit = latestImplementationArtifact?.implementationAudit;
  const latestRuntimeArtifact = useMemo(
    () => (snapshot?.artifacts ?? []).find((artifact) => artifact.runtimeAudit),
    [snapshot?.artifacts],
  );
  const latestRuntimeAudit = latestRuntimeArtifact?.runtimeAudit;
  const latestRiderArtifact = useMemo(
    () => (snapshot?.artifacts ?? []).find((artifact) => artifact.riderAudit),
    [snapshot?.artifacts],
  );
  const latestRiderAudit = latestRiderArtifact?.riderAudit;
  const implementationNoDiffPending =
    Boolean(latestArtifact?.implementationAudit) && latestArtifact?.implementationAudit?.workspaceChanged === false;
  const readyState = scene.stateStatus === "ready";
  const currentThreadId = request.threadId;
  const imaginationFinding = roleResults?.imagination?.finding;
  const canAcceptImagination =
    text(roleResults?.imagination?.status).toLowerCase() === "completed" &&
    Boolean(imaginationFinding?.statePatch?.planning);
  const modelingFinding = roleResults?.modeling?.finding;
  const canAcceptModeling =
    text(roleResults?.modeling?.status).toLowerCase() === "completed" && Boolean(modelingFinding?.statePatch);
  const canAcceptVerification = text(roleResults?.verification?.status).toLowerCase() === "completed";
  const canAcceptReorient = text(reorientResult?.status).toLowerCase() === "completed";
  const canContinueImplementation = text(coordinator.action).toLowerCase() === "continueimplementation";
  const selectedDraft = objectiveDrafts.find((draft) => text(draft.id, "") === request.planningDraftId);
  const selectedDraftStatus = text(selectedDraft?.status).toLowerCase();
  const canAdoptDraft =
    Boolean(selectedDraft) && !["adopted", "rejected", "superseded"].includes(selectedDraftStatus);
  const unityBridge = latestRuntimeAudit?.editorBridge;
  const installedEditors = latestRuntimeAudit?.installedEditors ?? [];
  const candidatePaths = latestRuntimeAudit?.candidatePaths ?? [];
  const searchRoots = latestRuntimeAudit?.searchRoots ?? [];
  const unityBridgeReady = latestRuntimeAudit?.status === "ready" && unityBridge?.exists === true;
  const epiphanyState = status?.read?.thread?.epiphanyState ?? status?.scene?.scene?.epiphanyState ?? {};
  const graphState = useMemo(() => normalizeGraphState(epiphanyState?.graphs), [epiphanyState]);
  const graphIssues = useMemo(() => validateEpiphanyGraphsState(graphState), [graphState]);
  const graphCount = graphRecordCount(graphState);
  const riderInstallations = latestRiderAudit?.installations ?? [];
  const riderSearchRoots = latestRiderAudit?.searchRoots ?? [];
  const riderChangedFiles = latestRiderAudit?.vcs?.changedFiles ?? [];

  useEffect(() => {
    if (objectiveDrafts.length === 0) return;
    const draftIds = new Set(objectiveDrafts.map((draft) => text(draft.id, "")).filter(Boolean));
    setRequest((current) => {
      if (current.planningDraftId && draftIds.has(current.planningDraftId)) {
        return current;
      }
      const firstDraft =
        objectiveDrafts.find((draft) => text(draft.status).toLowerCase() === "draft") ?? objectiveDrafts[0];
      const firstDraftId = text(firstDraft?.id, "");
      return firstDraftId ? { ...current, planningDraftId: firstDraftId } : current;
    });
  }, [objectiveDrafts, setRequest]);

  return (
    <main className="shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Epiphany MVP</p>
          <h1>Operator Console</h1>
        </div>
        <button className="primaryButton" onClick={() => void refresh()} disabled={loading} title="Refresh status">
          <RefreshCw size={16} aria-hidden="true" />
          {loading ? "Refreshing" : "Refresh"}
        </button>
      </header>

      <section className="controls" aria-label="Connection">
        <label>
          Thread ID
          <input
            placeholder="auto-load persistent status thread"
            value={request.threadId ?? ""}
            onChange={(event) => setRequest({ ...request, threadId: event.target.value || undefined })}
          />
        </label>
        <label>
          Workspace
          <input
            placeholder={snapshot?.repoRoot ?? "repo root"}
            value={request.cwd ?? ""}
            onChange={(event) => setRequest({ ...request, cwd: event.target.value || undefined })}
          />
        </label>
      </section>

      <section className="actionStrip" aria-label="Bounded operator actions">
        {actionButtons.map((button) => {
          const needsThread = button.requiresThread && !currentThreadId;
          const needsState = button.requiresReadyState && !readyState;
          const needsImagination = button.requiresImaginationPatch && !canAcceptImagination;
          const needsModeling = button.requiresModelingPatch && !canAcceptModeling;
          const needsVerification = button.requiresVerificationResult && !canAcceptVerification;
          const needsReorient = button.requiresReorientResult && !canAcceptReorient;
          const needsPlanningDraft = button.requiresPlanningDraft && !canAdoptDraft;
          const needsImplementation = button.requiresContinueImplementation && !canContinueImplementation;
          const needsNoDiffReview = button.requiresContinueImplementation && implementationNoDiffPending;
          const disabled =
            runningAction !== null ||
            needsThread ||
            needsState ||
            needsImagination ||
            needsModeling ||
            needsVerification ||
            needsReorient ||
            needsPlanningDraft ||
            needsImplementation ||
            needsNoDiffReview;
          const title = needsThread
              ? "Prepare a checkpoint or enter a persisted thread id first"
            : needsState
              ? "Prepare Epiphany state before launching this lane"
              : needsImagination
                ? "Read a completed imagination result with a planning patch before accepting it"
              : needsModeling
                ? "Read a completed modeling result with a state patch before accepting it"
                : needsVerification
                  ? "Read a completed verification result before accepting it"
                  : needsReorient
                    ? "Read a completed reorient result before accepting it"
                    : needsPlanningDraft
                      ? "Select a draft objective that has not already been adopted"
                      : needsImplementation
                        ? "Run the coordinator and clear review gates before continuing implementation"
                        : needsNoDiffReview
                          ? "Review the latest no-diff implementation artifact or run another lane before retrying"
                          : button.title;
          return (
            <button
              className="secondaryButton"
              onClick={() => void runAction(button.action)}
              disabled={disabled}
              title={title}
              key={button.action}
            >
              <ActionIcon icon={button.icon} />
              {runningAction === button.action ? button.runningLabel : button.label}
            </button>
          );
        })}
        {actionResult && (
          <p className="actionResult">
            {actionResult.summary} <code>{actionResult.artifactPath}</code>
          </p>
        )}
      </section>

      {error && (
        <section className="notice dangerNotice" role="alert">
          <AlertTriangle size={18} aria-hidden="true" />
          <span>{error}</span>
        </section>
      )}

      {latestImplementationAudit && (
        <section className={`notice ${latestImplementationAudit.workspaceChanged ? "okNotice" : "warnNotice"}`}>
          {latestImplementationAudit.workspaceChanged ? (
            <CheckCircle2 size={18} aria-hidden="true" />
          ) : (
            <AlertTriangle size={18} aria-hidden="true" />
          )}
          <span>
            <strong>Latest implementation audit:</strong>{" "}
            {latestImplementationAudit.workspaceChanged
              ? `${latestImplementationAudit.changedFiles.length} changed file(s) need review.`
              : "the worker completed with no workspace diff; review the artifact before rerunning."}{" "}
            <code>{latestImplementationArtifact?.path}</code>
          </span>
        </section>
      )}

      {latestRuntimeAudit && (
        <section className={`notice ${latestRuntimeAudit.status === "ready" ? "okNotice" : "warnNotice"}`}>
          {latestRuntimeAudit.status === "ready" ? (
            <CheckCircle2 size={18} aria-hidden="true" />
          ) : (
            <AlertTriangle size={18} aria-hidden="true" />
          )}
          <span>
            <strong>Latest runtime audit:</strong> Unity {text(latestRuntimeAudit.projectVersion)} is{" "}
            {text(latestRuntimeAudit.status)}. <code>{latestRuntimeArtifact?.path}</code>
          </span>
        </section>
      )}

      <section className="sectionBand">
        <SectionHeader title="Environment" icon={<Database size={18} />} />
        <div className="environmentGrid">
          <article className="environmentCard">
            <div className="cardTopline">
              <h3>Unity Editor</h3>
              <Pill tone={unityBridgeReady ? "ok" : statusClass(latestRuntimeAudit?.status)}>
                {unityBridgeReady ? "bridge ready" : text(latestRuntimeAudit?.status, "unknown")}
              </Pill>
            </div>
            <dl className="facts environmentFacts">
              <div><dt>Project</dt><dd>{text(latestRuntimeAudit?.projectVersion)}</dd></div>
              <div><dt>Editor</dt><dd>{text(latestRuntimeAudit?.editorPath, "missing")}</dd></div>
              <div><dt>Package</dt><dd>{unityBridge?.exists ? "present" : "missing"}</dd></div>
              <div><dt>Method</dt><dd>{text(unityBridge?.executeMethod)}</dd></div>
            </dl>
            {latestRuntimeAudit?.note && <p className="environmentNote">{latestRuntimeAudit.note}</p>}
            <PathList title="Installed" items={installedEditors.map((editor) => `${text(editor.version)} ${text(editor.editorPath)}`)} />
            <PathList title="Candidates" items={candidatePaths} />
          </article>

          <article className="environmentCard">
            <div className="cardTopline">
              <h3>Rider</h3>
              <Pill tone={statusClass(latestRiderAudit?.status)}>{text(latestRiderAudit?.status, "unknown")}</Pill>
            </div>
            <dl className="facts environmentFacts">
              <div><dt>Workspace</dt><dd>{text(latestRiderAudit?.workspace ?? request.cwd ?? snapshot?.repoRoot)}</dd></div>
              <div><dt>Solution</dt><dd>{text(latestRiderAudit?.solutionPath)}</dd></div>
              <div><dt>Rider</dt><dd>{text(latestRiderAudit?.riderPath, "missing")}</dd></div>
              <div><dt>Branch</dt><dd>{text(latestRiderAudit?.vcs?.branch)}</dd></div>
              <div><dt>Dirty</dt><dd>{text(latestRiderAudit?.vcs?.dirty)}</dd></div>
              <div><dt>Changed</dt><dd>{riderChangedFiles.length}</dd></div>
            </dl>
            <p className="environmentNote">{text(latestRiderAudit?.note, "Run Inspect Rider to capture source-context status.")}</p>
            <PathList title="Installations" items={riderInstallations.map((installation) => `${text(installation.versionHint)} ${text(installation.path)}`)} />
            <PathList title="Changed files" items={riderChangedFiles} />
            <PathList title="Search roots" items={riderSearchRoots} />
          </article>

          <article className="environmentCard">
            <div className="cardTopline">
              <h3>Artifacts</h3>
              <Pill tone={latestRuntimeArtifact ? "ok" : "neutral"}>{latestRuntimeArtifact ? "available" : "none"}</Pill>
            </div>
            <dl className="facts environmentFacts">
              <div><dt>Runtime bundle</dt><dd>{text(latestRuntimeArtifact?.name)}</dd></div>
              <div><dt>Files</dt><dd>{text(latestRuntimeArtifact?.files.length)}</dd></div>
              <div><dt>Summary</dt><dd>{text(latestRuntimeArtifact?.summaryPath)}</dd></div>
              <div><dt>Project path</dt><dd>{text(latestRuntimeAudit?.projectPath)}</dd></div>
            </dl>
            <PathList title="Search roots" items={searchRoots} />
            <code title={latestRuntimeArtifact?.path}>{text(latestRuntimeArtifact?.path)}</code>
          </article>
        </div>
      </section>

      <section className="sectionBand">
        <SectionHeader title="Planning" icon={<ListChecks size={18} />} />
        <div className="planningGrid">
          <article className="environmentCard planningSummary">
            <div className="cardTopline">
              <h3>State</h3>
              <Pill tone={statusClass(planningResponse?.stateStatus)}>
                {text(planningResponse?.stateStatus, "missing")}
              </Pill>
            </div>
            <dl className="facts environmentFacts">
              <div><dt>Captures</dt><dd>{countText(planningSummary?.captureCount)}</dd></div>
              <div><dt>Pending</dt><dd>{countText(planningSummary?.pendingCaptureCount)}</dd></div>
              <div><dt>Backlog</dt><dd>{countText(planningSummary?.backlogItemCount)}</dd></div>
              <div><dt>Ready</dt><dd>{countText(planningSummary?.readyBacklogItemCount)}</dd></div>
              <div><dt>Streams</dt><dd>{countText(planningSummary?.roadmapStreamCount)}</dd></div>
              <div><dt>Drafts</dt><dd>{countText(planningSummary?.objectiveDraftCount)}</dd></div>
            </dl>
            <label className="draftPicker">
              Objective Draft
              <select
                value={request.planningDraftId ?? ""}
                onChange={(event) =>
                  setRequest({ ...request, planningDraftId: event.target.value || undefined })
                }
                disabled={objectiveDrafts.length === 0}
              >
                <option value="">none</option>
                {objectiveDrafts.map((draft) => (
                  <option value={text(draft.id, "")} key={text(draft.id)}>
                    {text(draft.title)} [{text(draft.status)}]
                  </option>
                ))}
              </select>
            </label>
            <PathList title="Roadmap" items={roadmapStreams.map((stream) => `${text(stream.id)}: ${text(stream.title)}`)} />
            {planningSummary?.note && <p className="environmentNote">{text(planningSummary.note)}</p>}
          </article>

          <div className="planningColumn">
            <div className="cardTopline planningColumnHeader">
              <h3>Objective Drafts</h3>
              <Pill tone={objectiveDrafts.length ? "warn" : "neutral"}>{objectiveDrafts.length}</Pill>
            </div>
            {objectiveDrafts.slice(0, 4).map((draft) => (
              <PlanningItem
                key={text(draft.id)}
                title={text(draft.title)}
                status={text(draft.status)}
                selected={text(draft.id, "") === request.planningDraftId}
                body={text(draft.summary)}
                meta={[
                  text(draft.id),
                  `${
                    Array.isArray(draft.acceptance_criteria ?? draft.acceptanceCriteria)
                      ? (draft.acceptance_criteria ?? draft.acceptanceCriteria).length
                      : 0
                  } checks`,
                  listText(draft.source_item_ids ?? draft.sourceItemIds),
                ]}
              />
            ))}
            {objectiveDrafts.length === 0 && <EmptyState label="No objective drafts loaded." />}
          </div>

          <div className="planningColumn">
            <div className="cardTopline planningColumnHeader">
              <h3>Backlog</h3>
              <Pill tone={backlogItems.length ? "ok" : "neutral"}>{backlogItems.length}</Pill>
            </div>
            {backlogItems.slice(0, 4).map((item) => (
              <PlanningItem
                key={text(item.id)}
                title={text(item.title)}
                status={text(item.status)}
                body={text(item.summary)}
                meta={[
                  text(item.priority?.value),
                  text(item.horizon),
                  text(item.product_area ?? item.productArea),
                ]}
              />
            ))}
            {backlogItems.length === 0 && <EmptyState label="No backlog items loaded." />}
          </div>

          <div className="planningColumn">
            <div className="cardTopline planningColumnHeader">
              <h3>Captures</h3>
              <Pill tone={planningCaptures.length ? "neutral" : "neutral"}>{planningCaptures.length}</Pill>
            </div>
            {planningCaptures.slice(0, 4).map((capture) => {
              const source = capture.source ?? {};
              const sourceLabel =
                source.repo && source.issue_number
                  ? `${source.repo}#${source.issue_number}`
                  : text(source.kind);
              return (
                <PlanningItem
                  key={text(capture.id)}
                  title={text(capture.title)}
                  status={text(capture.status)}
                  body={text(capture.body)}
                  meta={[text(capture.confidence), sourceLabel, listText(capture.tags)]}
                />
              );
            })}
            {planningCaptures.length === 0 && <EmptyState label="No captures loaded." />}
          </div>
        </div>
      </section>

      <section className="sectionBand graphBand">
        <SectionHeader title="State Graph" icon={<Map size={18} />} />
        <div className="graphSummary">
          <dl className="facts environmentFacts">
            <div><dt>Architecture</dt><dd>{graphState.architecture.nodes.length} nodes / {graphState.architecture.edges.length} edges</dd></div>
            <div><dt>Dataflow</dt><dd>{graphState.dataflow.nodes.length} nodes / {graphState.dataflow.edges.length} edges</dd></div>
            <div><dt>Links</dt><dd>{graphState.links.length}</dd></div>
            <div><dt>Issues</dt><dd>{graphIssues.length}</dd></div>
          </dl>
          {selectedCodeRef && (
            <div className="selectedCodeRef">
              <Boxes size={16} aria-hidden="true" />
              <span>Selected code ref</span>
              <code title={codeRefLabel(selectedCodeRef)}>{codeRefLabel(selectedCodeRef)}</code>
            </div>
          )}
        </div>
        {graphIssues.length > 0 && (
          <div className="graphIssues">
            {graphIssues.slice(0, 4).map((issue) => (
              <Pill tone="warn" key={`${issue.scope}:${issue.message}`}>{issue.scope}: {issue.message}</Pill>
            ))}
          </div>
        )}
        {graphCount > 0 ? (
          <div className="graphViewerFrame">
            <EpiphanyGraphViewer
              state={graphState}
              title="Epiphany Typed Graph"
              className="embeddedGraphViewer"
              style={{ minHeight: 520 }}
              onCodeRefSelect={(codeRef) => setSelectedCodeRef(codeRef)}
            />
          </div>
        ) : (
          <EmptyState label="No graph state loaded. Prepare a checkpoint or accept a modeling patch to grow the map." />
        )}
      </section>

      <section className="statusGrid" aria-label="Coordinator summary">
        <Panel title="Recommendation" icon={<ClipboardCheck size={18} />}>
          <div className={`actionBanner ${statusClass(coordinator.action ?? crrc.action)}`}>
            <strong>{text(coordinator.action ?? crrc.action, "unknown")}</strong>
            <span>{text(coordinator.targetRole ?? crrc.recommendedSceneAction)}</span>
          </div>
          <p className="reason">{text(coordinator.reason ?? crrc.reason, "No recommendation loaded yet.")}</p>
          <dl className="facts">
            <div><dt>Thread</dt><dd>{text(status?.threadId)}</dd></div>
            <div><dt>State</dt><dd>{text(scene.stateStatus)} rev {text(scene.revision)}</dd></div>
            <div><dt>Requires review</dt><dd>{text(coordinator.requiresReview)}</dd></div>
          </dl>
        </Panel>

        <Panel title="Continuity" icon={<GitBranch size={18} />}>
          <dl className="facts">
            <div><dt>Pressure</dt><dd><Pill tone={statusClass(pressure.level)}>{text(pressure.level)}</Pill></dd></div>
            <div><dt>Prepare compaction</dt><dd>{text(pressure.shouldPrepareCompaction)}</dd></div>
            <div><dt>Reorient</dt><dd><Pill tone={statusClass(reorient.action)}>{text(reorient.action)}</Pill></dd></div>
            <div><dt>Reasons</dt><dd>{listText(reorient.reasons)}</dd></div>
          </dl>
          <p className="reason">{text(reorient.nextAction)}</p>
        </Panel>
      </section>

      <section className="sectionBand">
        <SectionHeader title="Role Lanes" icon={<BriefcaseBusiness size={18} />} />
        <div className="cardGrid">
          {roles.map((role) => (
            <article className="laneCard" key={text(role.id)}>
              <div className="cardTopline">
                <h3>{text(role.title)}</h3>
                <Pill tone={statusClass(role.status)}>{text(role.status)}</Pill>
              </div>
              <p>{text(role.note)}</p>
              <span className="owner">{text(role.ownerRole)}</span>
            </article>
          ))}
          {roles.length === 0 && <EmptyState label="No role lanes loaded." />}
        </div>
      </section>

      <section className="sectionBand twoColumn">
        <div>
          <SectionHeader title="Findings" icon={<FileText size={18} />} />
          <div className="stack">
            <Finding title="Imagination / Planning" result={roleResults.imagination} />
            <Finding title="Modeling / Checkpoint" result={roleResults.modeling} />
            <Finding title="Verification / Review" result={roleResults.verification} />
            <Finding title="Reorientation" result={reorientResult} findingKey="finding" />
          </div>
        </div>
        <div>
          <SectionHeader title="Jobs" icon={<Database size={18} />} />
          <div className="stack">
            {jobs.map((job) => (
              <article className="jobRow" key={text(job.id)}>
                <div>
                  <strong>{text(job.id)}</strong>
                  <span>{text(job.kind)} - {text(job.ownerRole)}</span>
                </div>
                <Pill tone={statusClass(job.status)}>{text(job.status)}</Pill>
              </article>
            ))}
            {jobs.length === 0 && <EmptyState label="No jobs loaded." />}
          </div>
        </div>
      </section>

      <section className="sectionBand">
        <SectionHeader title="Artifact Bundles" icon={<FileText size={18} />} />
        <div className="artifactTable" role="table" aria-label="Artifact bundles">
          <div className="artifactHeader" role="row">
            <span>Name</span>
            <span>Outcome</span>
            <span>Files</span>
            <span>Path</span>
          </div>
          {(snapshot?.artifacts ?? []).map((artifact: ArtifactBundle) => (
            <div className="artifactRow" role="row" key={artifact.path}>
              <strong>{artifact.name}</strong>
              <span><ArtifactOutcome artifact={artifact} /></span>
              <span>{artifact.files.length}</span>
              <code title={artifact.path}>{artifact.path}</code>
            </div>
          ))}
          {(snapshot?.artifacts ?? []).length === 0 && <EmptyState label="No dogfood artifact bundles found." />}
        </div>
      </section>
    </main>
  );
}

function Panel({ title, icon, children }: { title: string; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <section className="panel">
      <SectionHeader title={title} icon={icon} />
      {children}
    </section>
  );
}

function SectionHeader({ title, icon }: { title: string; icon: React.ReactNode }) {
  return (
    <div className="sectionHeader">
      {icon}
      <h2>{title}</h2>
    </div>
  );
}

function ActionIcon({ icon }: { icon: "file" | "check" | "play" | "eye" | "accept" | "runtime" | "plan" | "ide" }) {
  if (icon === "file") return <FileText size={16} aria-hidden="true" />;
  if (icon === "check") return <ClipboardCheck size={16} aria-hidden="true" />;
  if (icon === "play") return <Play size={16} aria-hidden="true" />;
  if (icon === "eye") return <Eye size={16} aria-hidden="true" />;
  if (icon === "runtime") return <Database size={16} aria-hidden="true" />;
  if (icon === "plan") return <ListChecks size={16} aria-hidden="true" />;
  if (icon === "ide") return <GitBranch size={16} aria-hidden="true" />;
  return <CheckCircle2 size={16} aria-hidden="true" />;
}

function Pill({ tone, children }: { tone: string; children: React.ReactNode }) {
  return <span className={`pill ${tone}`}>{children}</span>;
}

function Finding({ title, result, findingKey = "finding" }: { title: string; result: any; findingKey?: string }) {
  const finding = result?.[findingKey];
  return (
    <article className="findingCard">
      <div className="cardTopline">
        <h3>{title}</h3>
        <Pill tone={statusClass(result?.status)}>{text(result?.status)}</Pill>
      </div>
      {finding ? (
        <>
          <p>{text(finding.summary ?? finding.nextSafeMove ?? finding.mode ?? finding.verdict)}</p>
          <dl className="facts compact">
            <div><dt>Verdict</dt><dd>{text(finding.verdict ?? finding.mode)}</dd></div>
            <div><dt>Next</dt><dd>{text(finding.nextSafeMove)}</dd></div>
            <div><dt>Patch</dt><dd>{finding.statePatch ? "available" : "none"}</dd></div>
          </dl>
        </>
      ) : (
        <p>{text(result?.note, "No finding available.")}</p>
      )}
    </article>
  );
}

function PlanningItem({
  title,
  status,
  body,
  meta,
  selected = false,
}: {
  title: string;
  status: string;
  body: string;
  meta: string[];
  selected?: boolean;
}) {
  const metaItems = meta.filter((item) => item && item !== "none");
  return (
    <article className={`planningItem ${selected ? "selected" : ""}`}>
      <div className="cardTopline">
        <h3>{title}</h3>
        <Pill tone={statusClass(status)}>{status}</Pill>
      </div>
      <p>{body}</p>
      {metaItems.length > 0 && <span className="planningMeta">{metaItems.join(" / ")}</span>}
    </article>
  );
}

function ArtifactOutcome({ artifact }: { artifact: ArtifactBundle }) {
  const audit = artifact.implementationAudit;
  const runtime = artifact.runtimeAudit;
  const rider = artifact.riderAudit;
  if (rider) {
    return (
      <Pill tone={rider.status === "ready" || rider.status === "captured" ? "ok" : "warn"}>
        Rider {rider.status}
      </Pill>
    );
  }
  if (runtime) {
    return (
      <Pill tone={runtime.status === "ready" ? "ok" : "warn"}>
        Unity {runtime.status}
      </Pill>
    );
  }
  if (!audit) return <span className="artifactOutcome muted">none</span>;
  return (
    <Pill tone={audit.workspaceChanged ? "ok" : "warn"}>
      {audit.workspaceChanged ? "Diff" : "No Diff"}
    </Pill>
  );
}

function PathList({ title, items }: { title: string; items: string[] }) {
  if (!items.length) return null;
  return (
    <div className="pathList">
      <strong>{title}</strong>
      {items.slice(0, 4).map((item) => (
        <code key={item} title={item}>{item}</code>
      ))}
      {items.length > 4 && <span>{items.length - 4} more</span>}
    </div>
  );
}

function EmptyState({ label }: { label: string }) {
  return <p className="emptyState">{label}</p>;
}
