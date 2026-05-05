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
import { useEffect, useMemo, useRef, useState } from "react";
import { loadOperatorSnapshot, runOperatorAction } from "./operatorApi";
import type { ArtifactBundle, OperatorAction, OperatorActionResult, OperatorSnapshot, StatusRequest } from "./types";
import type { EpiphanyCodeRef, EpiphanyGraphsState } from "@epiphanygraph/epiphany-graph-viewer";

const roleOrder = ["implementation", "imagination", "research", "eyes", "modeling", "verification", "reorientation"];
const constellationSpecs = [
  {
    id: "coordinator",
    laneId: "coordinator",
    name: "Self",
    title: "Coordinator",
    glyph: "S",
    shape: "core",
    color: "#f15f45",
    glow: "#f7bd58",
    baseX: 49,
    baseY: 43,
    driftX: 1.9,
    driftY: 1.4,
    phase: 0.2,
  },
  {
    id: "imagination",
    laneId: "imagination",
    name: "Imagination",
    title: "Planner",
    glyph: "I",
    shape: "kite",
    color: "#9f6ee7",
    glow: "#eb8fc9",
    baseX: 29,
    baseY: 25,
    driftX: 2.3,
    driftY: 1.6,
    phase: 1.4,
  },
  {
    id: "research",
    laneId: "research",
    name: "Eyes",
    title: "Research",
    glyph: "E",
    shape: "lens",
    color: "#2877b8",
    glow: "#63c5da",
    baseX: 80,
    baseY: 23,
    driftX: 2.1,
    driftY: 1.7,
    phase: 2.1,
  },
  {
    id: "modeling",
    laneId: "modeling",
    name: "Body",
    title: "Modeling",
    glyph: "B",
    shape: "hex",
    color: "#2f9b67",
    glow: "#92d876",
    baseX: 21,
    baseY: 58,
    driftX: 1.8,
    driftY: 1.5,
    phase: 3.2,
  },
  {
    id: "implementation",
    laneId: "implementation",
    name: "Hands",
    title: "Implementation",
    glyph: "H",
    shape: "capsule",
    color: "#cf5a2f",
    glow: "#f1ad4e",
    baseX: 48,
    baseY: 64,
    driftX: 2.6,
    driftY: 1.2,
    phase: 4.4,
  },
  {
    id: "verification",
    laneId: "verification",
    name: "Soul",
    title: "Verification",
    glyph: "V",
    shape: "diamond",
    color: "#4e63b6",
    glow: "#a6a9f4",
    baseX: 78,
    baseY: 61,
    driftX: 1.7,
    driftY: 1.9,
    phase: 5.2,
  },
  {
    id: "reorientation",
    laneId: "reorientation",
    name: "Life",
    title: "Continuity",
    glyph: "L",
    shape: "seed",
    color: "#148d87",
    glow: "#58ddc4",
    baseX: 63,
    baseY: 29,
    driftX: 1.4,
    driftY: 2.2,
    phase: 6.0,
  },
] as const;

const compactConstellationPositions: Record<string, { x: number; y: number }> = {
  coordinator: { x: 50, y: 25 },
  imagination: { x: 14, y: 34 },
  research: { x: 86, y: 34 },
  reorientation: { x: 50, y: 43 },
  modeling: { x: 14, y: 52 },
  verification: { x: 86, y: 52 },
  implementation: { x: 50, y: 61 },
};
const fullscreenConstellationPositions: Record<string, { x: number; y: number }> = {
  coordinator: { x: 70, y: 30 },
  imagination: { x: 60, y: 18 },
  research: { x: 90, y: 24 },
  reorientation: { x: 82, y: 42 },
  modeling: { x: 64, y: 55 },
  verification: { x: 92, y: 58 },
  implementation: { x: 76, y: 66 },
};

type ConstellationSpec = (typeof constellationSpecs)[number];
type ProjectedAgent = ConstellationSpec & {
  status: string;
  tone: string;
  thought: string;
  detail: string;
  activity: number;
  jobs: number;
  review: string;
};
type AquariumOption = {
  label: string;
  deck?: DeckId;
  subdeck?: string;
  action?: OperatorAction;
};
const deckSubmenus = {
  command: ["run", "connection", "signals"],
  state: ["environment", "planning", "graph"],
  agents: ["lanes", "findings", "jobs"],
  artifacts: ["bundles"],
} as const;
const deckLabels: Record<keyof typeof deckSubmenus, string> = {
  command: "Command",
  state: "State",
  agents: "Agents",
  artifacts: "Artifacts",
};
type DeckId = keyof typeof deckSubmenus;
const aquariumOptionsByAgent: Record<string, AquariumOption[]> = {
  coordinator: [
    { label: "Signals", deck: "command", subdeck: "signals" },
    { label: "Run", deck: "command", subdeck: "run" },
    { label: "Checkpoint", action: "prepareCheckpoint" },
  ],
  imagination: [
    { label: "Planning", deck: "state", subdeck: "planning" },
    { label: "Launch", action: "launchImagination" },
    { label: "Read", action: "readImaginationResult" },
    { label: "Accept", action: "acceptImagination" },
  ],
  research: [
    { label: "State", deck: "state", subdeck: "graph" },
    { label: "Artifacts", deck: "artifacts", subdeck: "bundles" },
  ],
  modeling: [
    { label: "Graph", deck: "state", subdeck: "graph" },
    { label: "Launch", action: "launchModeling" },
    { label: "Read", action: "readModelingResult" },
    { label: "Accept", action: "acceptModeling" },
  ],
  implementation: [
    { label: "Run", deck: "command", subdeck: "run" },
    { label: "Continue", action: "continueImplementation" },
    { label: "Artifacts", deck: "artifacts", subdeck: "bundles" },
  ],
  verification: [
    { label: "Findings", deck: "agents", subdeck: "findings" },
    { label: "Launch", action: "launchVerification" },
    { label: "Read", action: "readVerificationResult" },
    { label: "Accept", action: "acceptVerification" },
  ],
  reorientation: [
    { label: "Continuity", deck: "command", subdeck: "signals" },
    { label: "Launch", action: "launchReorient" },
    { label: "Read", action: "readReorientResult" },
    { label: "Accept", action: "acceptReorient" },
  ],
};
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

function projectedThought(value: unknown, fallback: string): string {
  const cleaned = text(value, fallback).replace(/\s+/g, " ").trim();
  if (cleaned.length <= 136) return cleaned;
  return `${cleaned.slice(0, 132).trim()}...`;
}

function projectedActivity(status: unknown, jobCount = 0): number {
  const lower = text(status).toLowerCase();
  const jobBoost = Math.min(jobCount * 0.08, 0.24);
  if (lower.includes("critical") || lower.includes("running") || lower.includes("launch")) return 1;
  if (lower.includes("blocked") || lower.includes("needed") || lower.includes("regather")) return 0.82 + jobBoost;
  if (lower.includes("prepare") || lower.includes("review") || lower.includes("ready")) return 0.68 + jobBoost;
  if (lower.includes("completed") || lower.includes("continue") || lower.includes("pass")) return 0.48 + jobBoost;
  if (lower.includes("idle")) return 0.24 + jobBoost;
  return 0.34 + jobBoost;
}

function findingSummary(result: any): string | undefined {
  const finding = result?.finding;
  return finding?.summary ?? finding?.nextSafeMove ?? finding?.mode ?? finding?.verdict ?? result?.note;
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
  const [activeDeck, setActiveDeck] = useState<DeckId>("command");
  const [subdeckByDeck, setSubdeckByDeck] = useState<Record<DeckId, string>>({
    command: "run",
    state: "environment",
    agents: "lanes",
    artifacts: "bundles",
  });
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
  const activeSubdeck = subdeckByDeck[activeDeck];
  const activeDeckTitle = deckLabels[activeDeck];
  const graphMenuActive = activeDeck === "state" && activeSubdeck === "graph";

  function selectDeck(deck: DeckId) {
    setActiveDeck(deck);
  }

  function selectSubdeck(deck: DeckId, subdeck: string) {
    setSubdeckByDeck((current) => ({ ...current, [deck]: subdeck }));
  }

  function actionBlocked(action: OperatorAction) {
    const button = actionButtons.find((item) => item.action === action);
    if (!button) return true;
    return Boolean(
      runningAction !== null ||
        (button.requiresThread && !currentThreadId) ||
        (button.requiresReadyState && !readyState) ||
        (button.requiresImaginationPatch && !canAcceptImagination) ||
        (button.requiresModelingPatch && !canAcceptModeling) ||
        (button.requiresVerificationResult && !canAcceptVerification) ||
        (button.requiresReorientResult && !canAcceptReorient) ||
        (button.requiresPlanningDraft && !canAdoptDraft) ||
        (button.requiresContinueImplementation && !canContinueImplementation) ||
        (button.requiresContinueImplementation && implementationNoDiffPending),
    );
  }

  function handleAquariumOption(option: AquariumOption) {
    if (option.deck) {
      selectDeck(option.deck);
      if (option.subdeck) {
        selectSubdeck(option.deck, option.subdeck);
      }
    }
    if (option.action && !actionBlocked(option.action)) {
      void runAction(option.action);
    }
  }

  const actionControls = actionButtons.map((button) => {
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
        className="secondaryButton hudActionButton"
        onClick={() => void runAction(button.action)}
        disabled={disabled}
        title={title}
        key={button.action}
      >
        <ActionIcon icon={button.icon} />
        {runningAction === button.action ? button.runningLabel : button.label}
      </button>
    );
  });

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
    <main className="immersiveShell">
      <AgentConstellation
        roles={roles}
        roleResults={roleResults}
        reorientResult={reorientResult}
        coordinator={coordinator}
        crrc={crrc}
        pressure={pressure}
        reorient={reorient}
        jobs={jobs}
        variant="fullscreen"
        activeDeck={activeDeck}
        activeSubdeck={activeSubdeck}
        onAgentOption={handleAquariumOption}
        isActionBlocked={actionBlocked}
      />
      <div className="hudGrid" aria-hidden="true" />

      <header className="immersiveTopbar">
        <div className="operatorIdentity">
          <p className="eyebrow">Epiphany MVP</p>
          <h1>Operator Console</h1>
          <span>{text(coordinator.reason ?? crrc.reason, "No recommendation loaded yet.")}</span>
        </div>
        <div className="operatorTopControls">
          <Pill tone={statusClass(coordinator.action ?? crrc.action)}>
            {text(coordinator.action ?? crrc.action, "unknown")}
          </Pill>
          <Pill tone={statusClass(pressure.level)}>pressure {text(pressure.level, "unknown")}</Pill>
          <Pill tone={statusClass(reorient.action)}>continuity {text(reorient.action, "unknown")}</Pill>
          <button className="primaryButton" onClick={() => void refresh()} disabled={loading} title="Refresh status">
            <RefreshCw size={16} aria-hidden="true" />
            {loading ? "Refreshing" : "Refresh"}
          </button>
        </div>
      </header>

      <nav className="deckRail" aria-label="Primary operator menus">
        {(Object.keys(deckSubmenus) as DeckId[]).map((deck) => (
          <button
            type="button"
            className={activeDeck === deck ? "active" : ""}
            onClick={() => selectDeck(deck)}
            key={deck}
          >
            {deck === "command" && <ClipboardCheck size={17} aria-hidden="true" />}
            {deck === "state" && <Map size={17} aria-hidden="true" />}
            {deck === "agents" && <BriefcaseBusiness size={17} aria-hidden="true" />}
            {deck === "artifacts" && <FileText size={17} aria-hidden="true" />}
            <span>{deckLabels[deck]}</span>
          </button>
        ))}
      </nav>

      <section className={`diegeticPanel ${graphMenuActive ? "widePanel" : ""}`} aria-label={`${activeDeckTitle} menu`}>
        <div className="deckHeader">
          <div>
            <span>{activeDeckTitle}</span>
            <h2>{activeSubdeck}</h2>
          </div>
          <div className="subdeckTabs" role="tablist" aria-label={`${activeDeckTitle} sections`}>
            {deckSubmenus[activeDeck].map((subdeck) => (
              <button
                type="button"
                className={activeSubdeck === subdeck ? "active" : ""}
                onClick={() => selectSubdeck(activeDeck, subdeck)}
                key={subdeck}
              >
                {subdeck}
              </button>
            ))}
          </div>
        </div>

        <div className="deckBody">
          {activeDeck === "command" && activeSubdeck === "run" && (
            <>
              <section className="hudActionGrid" aria-label="Bounded operator actions">
                {actionControls}
              </section>
              {actionResult && (
                <p className="actionResult hudResult">
                  {actionResult.summary} <code>{actionResult.artifactPath}</code>
                </p>
              )}
            </>
          )}

          {activeDeck === "command" && activeSubdeck === "connection" && (
            <section className="hudFormGrid" aria-label="Connection">
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
              <dl className="facts compact">
                <div><dt>Thread</dt><dd>{text(status?.threadId)}</dd></div>
                <div><dt>State</dt><dd>{text(scene.stateStatus)} rev {text(scene.revision)}</dd></div>
                <div><dt>Repo</dt><dd>{text(snapshot?.repoRoot)}</dd></div>
                <div><dt>Draft</dt><dd>{text(request.planningDraftId)}</dd></div>
              </dl>
            </section>
          )}

          {activeDeck === "command" && activeSubdeck === "signals" && (
            <section className="signalStack" aria-label="Coordinator and continuity">
              <div className={`actionBanner ${statusClass(coordinator.action ?? crrc.action)}`}>
                <strong>{text(coordinator.action ?? crrc.action, "unknown")}</strong>
                <span>{text(coordinator.targetRole ?? crrc.recommendedSceneAction)}</span>
              </div>
              <p className="reason">{text(coordinator.reason ?? crrc.reason, "No recommendation loaded yet.")}</p>
              <dl className="facts">
                <div><dt>Requires review</dt><dd>{text(coordinator.requiresReview)}</dd></div>
                <div><dt>Pressure</dt><dd><Pill tone={statusClass(pressure.level)}>{text(pressure.level)}</Pill></dd></div>
                <div><dt>Prepare compaction</dt><dd>{text(pressure.shouldPrepareCompaction)}</dd></div>
                <div><dt>Reorient</dt><dd><Pill tone={statusClass(reorient.action)}>{text(reorient.action)}</Pill></dd></div>
                <div><dt>Reasons</dt><dd>{listText(reorient.reasons)}</dd></div>
                <div><dt>Next</dt><dd>{text(reorient.nextAction)}</dd></div>
              </dl>
            </section>
          )}

          {activeDeck === "state" && activeSubdeck === "environment" && (
            <div className="environmentGrid hudEnvironmentGrid">
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
                  <h3>Runtime Artifacts</h3>
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
          )}

          {activeDeck === "state" && activeSubdeck === "planning" && (
            <div className="planningGrid hudPlanningGrid">
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
                    meta={[text(item.priority?.value), text(item.horizon), text(item.product_area ?? item.productArea)]}
                  />
                ))}
                {backlogItems.length === 0 && <EmptyState label="No backlog items loaded." />}
              </div>

              <div className="planningColumn">
                <div className="cardTopline planningColumnHeader">
                  <h3>Captures</h3>
                  <Pill tone="neutral">{planningCaptures.length}</Pill>
                </div>
                {planningCaptures.slice(0, 4).map((capture) => {
                  const source = capture.source ?? {};
                  const sourceLabel =
                    source.repo && source.issue_number ? `${source.repo}#${source.issue_number}` : text(source.kind);
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
          )}

          {activeDeck === "state" && activeSubdeck === "graph" && (
            <section className="graphBand hudGraphBand">
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
          )}

          {activeDeck === "agents" && activeSubdeck === "lanes" && (
            <div className="cardGrid hudCardGrid">
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
          )}

          {activeDeck === "agents" && activeSubdeck === "findings" && (
            <div className="stack">
              <Finding title="Imagination / Planning" result={roleResults.imagination} />
              <Finding title="Modeling / Checkpoint" result={roleResults.modeling} />
              <Finding title="Verification / Review" result={roleResults.verification} />
              <Finding title="Reorientation" result={reorientResult} findingKey="finding" />
            </div>
          )}

          {activeDeck === "agents" && activeSubdeck === "jobs" && (
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
          )}

          {activeDeck === "artifacts" && activeSubdeck === "bundles" && (
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
          )}
        </div>
      </section>

      <aside className="hudToastStack" aria-label="Audit alerts">
        {error && (
          <section className="hudToast dangerNotice" role="alert">
            <AlertTriangle size={18} aria-hidden="true" />
            <span>{error}</span>
          </section>
        )}
        {latestImplementationAudit && (
          <section className={`hudToast ${latestImplementationAudit.workspaceChanged ? "okNotice" : "warnNotice"}`}>
            {latestImplementationAudit.workspaceChanged ? (
              <CheckCircle2 size={18} aria-hidden="true" />
            ) : (
              <AlertTriangle size={18} aria-hidden="true" />
            )}
            <span>
              <strong>Implementation:</strong>{" "}
              {latestImplementationAudit.workspaceChanged
                ? `${latestImplementationAudit.changedFiles.length} changed file(s).`
                : "no workspace diff; review before rerun."}
            </span>
          </section>
        )}
        {latestRuntimeAudit && (
          <section className={`hudToast ${latestRuntimeAudit.status === "ready" ? "okNotice" : "warnNotice"}`}>
            {latestRuntimeAudit.status === "ready" ? (
              <CheckCircle2 size={18} aria-hidden="true" />
            ) : (
              <AlertTriangle size={18} aria-hidden="true" />
            )}
            <span>
              <strong>Unity:</strong> {text(latestRuntimeAudit.projectVersion)} is {text(latestRuntimeAudit.status)}.
            </span>
          </section>
        )}
      </aside>
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

function AgentConstellation({
  roles,
  roleResults,
  reorientResult,
  coordinator,
  crrc,
  pressure,
  reorient,
  jobs,
  variant = "band",
  activeDeck,
  activeSubdeck,
  onAgentOption,
  isActionBlocked,
}: {
  roles: any[];
  roleResults: any;
  reorientResult: any;
  coordinator: any;
  crrc: any;
  pressure: any;
  reorient: any;
  jobs: any[];
  variant?: "band" | "fullscreen";
  activeDeck?: DeckId;
  activeSubdeck?: string;
  onAgentOption?: (option: AquariumOption) => void;
  isActionBlocked?: (action: OperatorAction) => boolean;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const pointerRef = useRef({ active: false, x: 0, y: 0 });
  const hotZonesRef = useRef<Array<{ x: number; y: number; radius: number; option: AquariumOption }>>([]);
  const latestProjectedRef = useRef<Array<ProjectedAgent & { x: number; y: number }>>([]);
  const [selectedAgentId, setSelectedAgentId] = useState("coordinator");
  const agents = useMemo<ProjectedAgent[]>(() => {
    return constellationSpecs.map((spec) => {
      const lane = roles.find((role) => text(role.id).toLowerCase() === spec.laneId);
      const result =
        roleResults?.[spec.laneId] ??
        (spec.id === "research" ? roleResults?.eyes ?? roleResults?.research : undefined);
      const ownedJobs = jobs.filter((job) => {
        const owner = text(job.ownerRole).toLowerCase();
        const kind = text(job.kind).toLowerCase();
        return owner.includes(spec.laneId) || owner.includes(spec.id) || kind.includes(spec.laneId);
      });
      let status = text(lane?.status ?? result?.status, "idle");
      let thought = projectedThought(
        lane?.note ?? findingSummary(result),
        "Quiet. Waiting for a bounded signal.",
      );
      let detail = text(lane?.ownerRole ?? result?.bindingId, spec.title);
      let review = result?.finding?.statePatch ? "patch review" : "none";

      if (spec.id === "coordinator") {
        status = text(coordinator?.action ?? crrc?.action, "unknown");
        thought = projectedThought(
          coordinator?.reason ?? crrc?.reason,
          "No coordinator projection loaded yet.",
        );
        detail = `target ${text(coordinator?.targetRole ?? crrc?.recommendedSceneAction, "none")}`;
        review = text(coordinator?.requiresReview, "false") === "true" ? "required" : "not required";
      } else if (spec.id === "reorientation") {
        status = text(lane?.status ?? reorient?.action ?? reorientResult?.status, "idle");
        thought = projectedThought(
          lane?.note ?? findingSummary(reorientResult) ?? reorient?.nextAction,
          "Continuity is quiet.",
        );
        detail = `pressure ${text(pressure?.level, "unknown")}`;
        review = text(reorientResult?.status).toLowerCase() === "completed" ? "read result" : "none";
      } else if (spec.id === "research" && !lane && !result) {
        status = "idle";
        thought = "Watching for proven paths before the machine invents one in a shed.";
        detail = "future lane";
        review = "none";
      }

      return {
        ...spec,
        status,
        tone: statusClass(status),
        thought,
        detail,
        activity: Math.min(projectedActivity(status, ownedJobs.length), 1),
        jobs: ownedJobs.length,
        review,
      };
    });
  }, [coordinator, crrc, jobs, pressure, reorient, reorientResult, roleResults, roles]);
  const selectedAgent = agents.find((agent) => agent.id === selectedAgentId) ?? agents[0];

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const canvasElement = canvas;
    const screen = canvasElement.getContext("2d", { alpha: false });
    if (!screen) return;
    const colorBuffer = document.createElement("canvas");
    const scratchBuffer = document.createElement("canvas");
    const color = colorBuffer.getContext("2d", { alpha: false });
    const scratch = scratchBuffer.getContext("2d", { alpha: false });
    if (!color || !scratch) return;
    const screenContext = screen;
    const colorContext = color;
    const scratchContext = scratch;
    let frame = 0;
    let tick = 0;
    const motion = new globalThis.Map<string, {
      x: number;
      y: number;
      vx: number;
      vy: number;
      lastTextTick: number;
    }>();
    const agentData = agents.map((agent, index) => ({
      ...agent,
      index,
    }));

    function resizeCanvas() {
      const bounds = canvasElement.getBoundingClientRect();
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const width = Math.max(1, Math.floor(bounds.width * dpr));
      const height = Math.max(1, Math.floor(bounds.height * dpr));
      if (canvasElement.width !== width || canvasElement.height !== height) {
        canvasElement.width = width;
        canvasElement.height = height;
      }
      if (colorBuffer.width !== width || colorBuffer.height !== height) {
        colorBuffer.width = width;
        colorBuffer.height = height;
        scratchBuffer.width = width;
        scratchBuffer.height = height;
        colorContext.fillStyle = "#07110e";
        colorContext.fillRect(0, 0, width, height);
      }
      return { width, height, dpr };
    }

    function basePoint(agent: (typeof agentData)[number], width: number, height: number) {
      const compactPosition = width < 540 ? compactConstellationPositions[agent.id] : undefined;
      const fullscreenPosition = variant === "fullscreen" ? fullscreenConstellationPositions[agent.id] : undefined;
      const baseY = compactPosition?.y ?? fullscreenPosition?.y ?? agent.baseY;
      const resolvedBaseX = compactPosition?.x ?? fullscreenPosition?.x ?? agent.baseX;
      return {
        x: (resolvedBaseX / 100) * width,
        y: (baseY / 100) * height,
      };
    }

    function updateMotion(time: number, width: number, height: number) {
      return agentData.map((agent) => {
        const base = basePoint(agent, width, height);
        const state = motion.get(agent.id) ?? {
          x: base.x,
          y: base.y,
          vx: 0,
          vy: 0,
          lastTextTick: -999,
        };
        const liveliness = Math.max(0.05, agent.activity);
        const hoverPull = pointerRef.current.active ? pointerPull(agent, state.x, state.y) : { x: 0, y: 0 };
        const swim = variant === "fullscreen" ? 46 + liveliness * 92 : 10 + liveliness * 24;
        const speed = 0.010 + liveliness * 0.034;
        const targetX = base.x + Math.sin(time * (0.42 + liveliness * 0.52) + agent.phase) * swim + hoverPull.x;
        const targetY = base.y + Math.cos(time * (0.36 + liveliness * 0.44) + agent.phase * 1.7) * swim * 0.62 + hoverPull.y;
        state.vx = state.vx * 0.88 + (targetX - state.x) * speed;
        state.vy = state.vy * 0.88 + (targetY - state.y) * speed;
        state.x = clamp(state.x + state.vx, 44, width - 44);
        state.y = clamp(state.y + state.vy, 56, height - 56);
        motion.set(agent.id, state);
        return { ...agent, ...state, speed: Math.hypot(state.vx, state.vy) };
      });
    }

    function pointerPull(agent: ProjectedAgent, x: number, y: number) {
      const pointer = pointerRef.current;
      const dx = pointer.x - x;
      const dy = pointer.y - y;
      const distance = Math.hypot(dx, dy);
      if (distance > 180 || distance < 1) return { x: 0, y: 0 };
      const strength = (1 - distance / 180) * (18 + agent.activity * 38);
      return { x: (dx / distance) * strength, y: (dy / distance) * strength };
    }

    function drawFrame(millis: number) {
      const time = millis / 1000;
      tick += 1;
      const { width, height } = resizeCanvas();
      const projected = updateMotion(time, width, height);
      latestProjectedRef.current = projected;
      const hovered = pointerRef.current.active ? nearestAgent(projected, pointerRef.current.x, pointerRef.current.y) : null;
      const activeAgent = hovered ?? projected.find((agent) => agent.id === selectedAgentId) ?? projected[0];

      scratchContext.globalCompositeOperation = "source-over";
      scratchContext.globalAlpha = 1;
      scratchContext.filter = "none";
      scratchContext.fillStyle = "rgba(4, 9, 7, 0.16)";
      scratchContext.fillRect(0, 0, width, height);
      scratchContext.globalAlpha = 0.972;
      scratchContext.filter = "blur(1.4px) saturate(1.045)";
      const driftX = Math.sin(time * 0.17) * 6;
      const driftY = Math.cos(time * 0.13) * 5;
      scratchContext.drawImage(colorBuffer, -8 + driftX, -6 + driftY, width + 16, height + 12);
      scratchContext.filter = "none";
      colorContext.globalAlpha = 1;
      colorContext.drawImage(scratchBuffer, 0, 0);
      drawBackgroundTint(colorContext, width, height, time);
      drawWakes(colorContext, projected, time);

      for (const agent of projected) {
        const isHot = agent.id === hovered?.id || agent.id === selectedAgentId;
        const state = motion.get(agent.id);
        if (state && (isHot || tick - state.lastTextTick > 60 + agent.index * 19)) {
          drawThought(colorContext, agent, width, height, isHot);
          state.lastTextTick = tick;
        }
        drawAgent(colorContext, agent, isHot);
      }

      if (activeAgent) {
        drawOptions(colorContext, activeAgent, width, height);
      }

      drawActiveDeckGlyph(colorContext, width, height);
      screenContext.imageSmoothingEnabled = true;
      screenContext.drawImage(colorBuffer, 0, 0);
      frame = requestAnimationFrame(drawFrame);
    }

    frame = requestAnimationFrame(drawFrame);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [activeDeck, activeSubdeck, agents, isActionBlocked, selectedAgentId, variant]);

  function canvasPoint(event: React.PointerEvent<HTMLCanvasElement>) {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * canvas.width,
      y: ((event.clientY - rect.top) / rect.height) * canvas.height,
    };
  }

  function handlePointerMove(event: React.PointerEvent<HTMLCanvasElement>) {
    const point = canvasPoint(event);
    pointerRef.current = { active: true, ...point };
  }

  function handlePointerLeave() {
    pointerRef.current = { active: false, x: 0, y: 0 };
  }

  function handleCanvasClick() {
    const pointer = pointerRef.current;
    const hit = hotZonesRef.current.find((zone) => Math.hypot(zone.x - pointer.x, zone.y - pointer.y) <= zone.radius);
    if (hit) {
      onAgentOption?.(hit.option);
      return;
    }
    const agent = nearestAgent(latestProjectedRef.current, pointer.x, pointer.y, 96);
    if (agent) {
      setSelectedAgentId(agent.id);
    }
  }

  function nearestAgent<T extends ProjectedAgent & { x: number; y: number }>(projected: T[], x: number, y: number, limit = 180) {
    let best: T | null = null;
    let bestDistance = limit;
    for (const agent of projected) {
      const distance = Math.hypot(agent.x - x, agent.y - y);
      if (distance < bestDistance) {
        best = agent;
        bestDistance = distance;
      }
    }
    return best;
  }

  function drawBackgroundTint(ctx: CanvasRenderingContext2D, width: number, height: number, time: number) {
    ctx.save();
    ctx.globalCompositeOperation = "source-over";
    ctx.globalAlpha = 0.42;
    const gradient = ctx.createRadialGradient(
      width * (0.42 + Math.sin(time * 0.11) * 0.05),
      height * (0.44 + Math.cos(time * 0.09) * 0.04),
      80,
      width * 0.5,
      height * 0.5,
      Math.max(width, height) * 0.82,
    );
    gradient.addColorStop(0, "rgba(52, 84, 70, 0.22)");
    gradient.addColorStop(0.46, "rgba(23, 17, 36, 0.14)");
    gradient.addColorStop(1, "rgba(5, 10, 8, 0.34)");
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, width, height);
    ctx.restore();
  }

  function drawWakes(ctx: CanvasRenderingContext2D, projected: Array<ProjectedAgent & { x: number; y: number; vx: number; vy: number; speed: number }>, time: number) {
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    for (const agent of projected) {
      const speed = Math.min(1, agent.speed / 8);
      const radius = 42 + agent.activity * 62 + speed * 80;
      const glow = ctx.createRadialGradient(agent.x, agent.y, 5, agent.x, agent.y, radius);
      glow.addColorStop(0, hexAlpha(agent.glow, 0.28 + agent.activity * 0.22));
      glow.addColorStop(0.38, hexAlpha(agent.color, 0.12 + speed * 0.18));
      glow.addColorStop(1, "rgba(0, 0, 0, 0)");
      ctx.fillStyle = glow;
      ctx.beginPath();
      ctx.arc(agent.x, agent.y, radius, 0, Math.PI * 2);
      ctx.fill();

      ctx.strokeStyle = hexAlpha(agent.color, 0.08 + agent.activity * 0.12);
      ctx.lineWidth = 1.4 + agent.activity * 2.2;
      for (let ring = 0; ring < 3; ring += 1) {
        ctx.beginPath();
        ctx.arc(
          agent.x - agent.vx * (5 + ring * 4),
          agent.y - agent.vy * (5 + ring * 4),
          24 + ring * 32 + Math.sin(time * 2.3 + ring + agent.phase) * 7,
          0,
          Math.PI * 2,
        );
        ctx.stroke();
      }
    }
    ctx.restore();
  }

  function drawAgent(ctx: CanvasRenderingContext2D, agent: ProjectedAgent & { x: number; y: number; vx: number; vy: number; speed: number }, hot: boolean) {
    const size = 34 + agent.activity * 18 + (hot ? 8 : 0);
    const tilt = Math.atan2(agent.vy, agent.vx || 0.001) * 0.16;
    ctx.save();
    ctx.translate(agent.x, agent.y);
    ctx.rotate(tilt);
    ctx.globalCompositeOperation = "lighter";
    ctx.shadowColor = agent.glow;
    ctx.shadowBlur = 18 + agent.activity * 26;
    ctx.fillStyle = agent.color;
    drawAgentPath(ctx, agent.shape, size);
    ctx.fill();
    ctx.shadowBlur = 0;
    ctx.strokeStyle = "rgba(255, 255, 255, 0.72)";
    ctx.lineWidth = hot ? 2.4 : 1.2;
    ctx.stroke();
    ctx.globalCompositeOperation = "source-over";
    ctx.fillStyle = "#fffaf0";
    ctx.font = `800 ${Math.max(14, size * 0.42)}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(agent.glyph, 0, 1);
    ctx.restore();

    ctx.save();
    ctx.globalCompositeOperation = "source-over";
    ctx.fillStyle = "rgba(5, 12, 9, 0.72)";
    ctx.strokeStyle = hexAlpha(agent.color, 0.52);
    roundedRect(ctx, agent.x - 42, agent.y + size * 0.64, 84, 36, 7);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "rgba(247, 255, 247, 0.92)";
    ctx.font = "800 11px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(agent.name, agent.x, agent.y + size * 0.64 + 14);
    ctx.fillStyle = "rgba(226, 245, 225, 0.74)";
    ctx.font = "800 9px Inter, system-ui, sans-serif";
    ctx.fillText(agent.status.slice(0, 16).toUpperCase(), agent.x, agent.y + size * 0.64 + 28);
    ctx.restore();
  }

  function drawAgentPath(ctx: CanvasRenderingContext2D, shape: string, size: number) {
    const r = size / 2;
    ctx.beginPath();
    if (shape === "kite" || shape === "diamond") {
      ctx.moveTo(0, -r);
      ctx.lineTo(r * 0.9, 0);
      ctx.lineTo(0, r);
      ctx.lineTo(-r * 0.9, 0);
      ctx.closePath();
    } else if (shape === "hex") {
      for (let index = 0; index < 6; index += 1) {
        const angle = Math.PI / 6 + index * (Math.PI / 3);
        const x = Math.cos(angle) * r;
        const y = Math.sin(angle) * r;
        if (index === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.closePath();
    } else if (shape === "capsule") {
      roundedRect(ctx, -r * 1.18, -r * 0.74, r * 2.36, r * 1.48, r * 0.54);
    } else if (shape === "lens") {
      ctx.ellipse(0, 0, r * 1.08, r * 0.76, Math.PI / 4, 0, Math.PI * 2);
    } else if (shape === "seed") {
      ctx.ellipse(0, 0, r * 0.82, r * 1.08, Math.PI / 4, 0, Math.PI * 2);
    } else {
      ctx.arc(0, 0, r, 0, Math.PI * 2);
    }
  }

  function drawThought(ctx: CanvasRenderingContext2D, agent: ProjectedAgent & { x: number; y: number }, width: number, height: number, hot: boolean) {
    const boxWidth = Math.min(280, Math.max(180, width * 0.2));
    const x = clamp(agent.x + (agent.x > width * 0.7 ? -boxWidth - 44 : 44), 16, width - boxWidth - 16);
    const y = clamp(agent.y - 86, 16, height - 132);
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.shadowColor = agent.glow;
    ctx.shadowBlur = hot ? 18 : 8;
    ctx.fillStyle = hexAlpha(agent.color, hot ? 0.22 : 0.12);
    roundedRect(ctx, x - 6, y - 6, boxWidth + 12, 104, 10);
    ctx.fill();
    ctx.globalCompositeOperation = "source-over";
    ctx.shadowBlur = 0;
    ctx.fillStyle = hot ? "rgba(248, 252, 242, 0.86)" : "rgba(248, 252, 242, 0.52)";
    ctx.strokeStyle = hexAlpha(agent.color, hot ? 0.82 : 0.38);
    roundedRect(ctx, x, y, boxWidth, 92, 9);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = agent.color;
    ctx.font = "900 12px Inter, system-ui, sans-serif";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText(agent.name.toUpperCase(), x + 12, y + 10);
    ctx.fillStyle = "rgba(23, 32, 24, 0.78)";
    ctx.font = "700 13px Inter, system-ui, sans-serif";
    wrapCanvasText(ctx, agent.thought, x + 12, y + 30, boxWidth - 24, 17, 3);
    ctx.restore();
  }

  function drawOptions(ctx: CanvasRenderingContext2D, agent: ProjectedAgent & { x: number; y: number }, width: number, height: number) {
    const options = aquariumOptionsByAgent[agent.id] ?? [];
    hotZonesRef.current = [];
    if (!options.length) return;
    const radius = width < 540 ? 74 : 96;
    const arc = Math.min(Math.PI * 1.25, Math.max(Math.PI * 0.72, options.length * 0.36));
    const start = -Math.PI / 2 - arc / 2;
    ctx.save();
    ctx.font = "900 11px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    for (let index = 0; index < options.length; index += 1) {
      const option = options[index];
      const angle = start + (arc * (index + 0.5)) / options.length;
      const x = clamp(agent.x + Math.cos(angle) * radius, 48, width - 48);
      const y = clamp(agent.y + Math.sin(angle) * radius, 56, height - 56);
      const disabled = option.action ? Boolean(isActionBlocked?.(option.action)) : false;
      const hot = pointerRef.current.active && Math.hypot(pointerRef.current.x - x, pointerRef.current.y - y) < 34;
      if (!disabled) {
        hotZonesRef.current.push({ x, y, radius: 36, option });
      }
      ctx.globalCompositeOperation = "lighter";
      ctx.fillStyle = hexAlpha(agent.glow, hot ? 0.32 : 0.16);
      ctx.beginPath();
      ctx.arc(x, y, hot ? 42 : 34, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalCompositeOperation = "source-over";
      ctx.fillStyle = disabled ? "rgba(8, 14, 12, 0.52)" : hot ? hexAlpha(agent.color, 0.8) : "rgba(8, 14, 12, 0.74)";
      ctx.strokeStyle = disabled ? "rgba(226, 245, 225, 0.18)" : hexAlpha(agent.glow, hot ? 0.92 : 0.54);
      roundedRect(ctx, x - 38, y - 17, 76, 34, 17);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = disabled ? "rgba(236, 246, 235, 0.34)" : "#fbfff8";
      ctx.fillText(option.label.toUpperCase(), x, y + 1);
    }
    ctx.restore();
  }

  function drawActiveDeckGlyph(ctx: CanvasRenderingContext2D, width: number, height: number) {
    if (!activeDeck) return;
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.fillStyle = "rgba(247, 189, 88, 0.08)";
    ctx.font = `900 ${Math.max(48, Math.min(width, height) * 0.09)}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "right";
    ctx.textBaseline = "bottom";
    ctx.fillText(`${deckLabels[activeDeck]} / ${activeSubdeck ?? ""}`.toUpperCase(), width - 24, height - 22);
    ctx.restore();
  }

  function roundedRect(ctx: CanvasRenderingContext2D, x: number, y: number, width: number, height: number, radius: number) {
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.arcTo(x + width, y, x + width, y + height, radius);
    ctx.arcTo(x + width, y + height, x, y + height, radius);
    ctx.arcTo(x, y + height, x, y, radius);
    ctx.arcTo(x, y, x + width, y, radius);
    ctx.closePath();
  }

  function wrapCanvasText(ctx: CanvasRenderingContext2D, value: string, x: number, y: number, maxWidth: number, lineHeight: number, maxLines: number) {
    const words = value.split(/\s+/);
    let line = "";
    let lineCount = 0;
    for (const word of words) {
      const test = line ? `${line} ${word}` : word;
      if (ctx.measureText(test).width > maxWidth && line) {
        ctx.fillText(lineCount + 1 === maxLines ? `${line}...` : line, x, y + lineCount * lineHeight);
        line = word;
        lineCount += 1;
        if (lineCount >= maxLines) return;
      } else {
        line = test;
      }
    }
    if (line && lineCount < maxLines) {
      ctx.fillText(line, x, y + lineCount * lineHeight);
    }
  }

  return (
    <section className={`${variant === "fullscreen" ? "immersiveConstellation" : "sectionBand agentConstellation"}`} aria-label="Agent state overview">
      {variant === "band" && (
        <div className="constellationHeader">
          <SectionHeader title="Agent State" icon={<Boxes size={18} />} />
          <div className="constellationSignals" aria-label="Global signals">
            <Pill tone={statusClass(coordinator?.action ?? crrc?.action)}>
              {text(coordinator?.action ?? crrc?.action, "unknown")}
            </Pill>
            <Pill tone={statusClass(pressure?.level)}>pressure {text(pressure?.level, "unknown")}</Pill>
            <Pill tone={statusClass(reorient?.action)}>continuity {text(reorient?.action, "unknown")}</Pill>
          </div>
        </div>
      )}
      <div className="agentStage">
        <canvas
          ref={canvasRef}
          className="agentSmokeCanvas"
          aria-hidden="true"
          onPointerMove={handlePointerMove}
          onPointerLeave={handlePointerLeave}
          onClick={handleCanvasClick}
        />
        <div className="agentStageVignette" aria-hidden="true" />
        {variant === "band" ? (
          agents.map((agent) => (
            <button
              className={`agentCharacter ${agent.shape} ${agent.tone} ${selectedAgentId === agent.id ? "selected" : ""}`}
              key={agent.id}
              type="button"
              data-agent-node={agent.id}
              onClick={() => setSelectedAgentId(agent.id)}
              title={`${agent.name}: ${agent.thought}`}
              style={
                {
                  "--agent-x": `${agent.baseX}%`,
                  "--agent-y": `${agent.baseY}%`,
                  "--agent-color": agent.color,
                  "--agent-glow": agent.glow,
                  "--agent-activity": agent.activity,
                  "--agent-bubble-opacity": 0.38 + agent.activity * 0.28,
                } as React.CSSProperties
              }
            >
              <span className="agentGlyph" aria-hidden="true">{agent.glyph}</span>
              <span className="agentCaption">
                <strong>{agent.name}</strong>
                <span>{agent.status}</span>
              </span>
            </button>
          ))
        ) : (
          <div className="simulationOnlyControls">
            {agents.map((agent) => (
              <button
                type="button"
                key={agent.id}
                onClick={() => setSelectedAgentId(agent.id)}
                aria-label={`${agent.name} ${agent.status}`}
              />
            ))}
          </div>
        )}
        {variant === "band" && (
          <div className="constellationInspector">
            <div>
              <span>{selectedAgent.title}</span>
              <strong>{selectedAgent.name}</strong>
              <p>{selectedAgent.thought}</p>
            </div>
            <dl className="facts compact">
              <div><dt>Status</dt><dd><Pill tone={selectedAgent.tone}>{selectedAgent.status}</Pill></dd></div>
              <div><dt>Detail</dt><dd>{selectedAgent.detail}</dd></div>
              <div><dt>Jobs</dt><dd>{selectedAgent.jobs}</dd></div>
              <div><dt>Review</dt><dd>{selectedAgent.review}</dd></div>
            </dl>
          </div>
        )}
      </div>
    </section>
  );
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function hexAlpha(hex: string, alpha: number) {
  const normalized = hex.replace("#", "");
  const value = Number.parseInt(normalized.length === 3
    ? normalized.split("").map((char) => `${char}${char}`).join("")
    : normalized, 16);
  return `rgba(${(value >> 16) & 255}, ${(value >> 8) & 255}, ${value & 255}, ${clamp(alpha, 0, 1)})`;
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
            <div><dt>Self Memory</dt><dd>{text(finding.selfPersistence?.status, "none")}</dd></div>
          </dl>
          {finding.selfPersistence?.reasons?.length ? (
            <p>{finding.selfPersistence.reasons.join("; ")}</p>
          ) : null}
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
