import {
  AlertTriangle,
  BriefcaseBusiness,
  CheckCircle2,
  ClipboardCheck,
  Database,
  Eye,
  FileText,
  GitBranch,
  Play,
  RefreshCw,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { loadOperatorSnapshot, runOperatorAction } from "./operatorApi";
import type { ArtifactBundle, OperatorAction, OperatorActionResult, OperatorSnapshot, StatusRequest } from "./types";

const roleOrder = ["implementation", "modeling", "verification", "reorientation"];
const actionButtons: Array<{
  action: OperatorAction;
  label: string;
  runningLabel: string;
  title: string;
  requiresThread?: boolean;
  requiresReadyState?: boolean;
  requiresReorientResult?: boolean;
  icon: "file" | "check" | "play" | "eye" | "accept";
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
    action: "prepareCheckpoint",
    label: "Prepare Checkpoint",
    runningLabel: "Preparing",
    title: "Seed durable Epiphany state for this GUI operator thread",
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

function statusClass(value: unknown): string {
  const lower = text(value).toLowerCase();
  if (lower.includes("blocked") || lower.includes("critical") || lower.includes("regather")) return "danger";
  if (lower.includes("needed") || lower.includes("review") || lower.includes("prepare") || lower.includes("high")) return "warn";
  if (lower.includes("completed") || lower.includes("ready") || lower.includes("continue") || lower.includes("pass")) return "ok";
  return "neutral";
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
  const roleResults = status?.roleResults ?? {};
  const reorientResult = status?.reorientResult ?? {};
  const readyState = scene.stateStatus === "ready";
  const currentThreadId = request.threadId;
  const canAcceptReorient = text(reorientResult?.status).toLowerCase() === "completed";

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
          const needsReorient = button.requiresReorientResult && !canAcceptReorient;
          const disabled = runningAction !== null || needsThread || needsState || needsReorient;
          const title = needsThread
            ? "Prepare a checkpoint or enter a persisted thread id first"
            : needsState
              ? "Prepare Epiphany state before launching this lane"
              : needsReorient
                ? "Read a completed reorient result before accepting it"
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
            <span>Files</span>
            <span>Path</span>
          </div>
          {(snapshot?.artifacts ?? []).map((artifact: ArtifactBundle) => (
            <div className="artifactRow" role="row" key={artifact.path}>
              <strong>{artifact.name}</strong>
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

function ActionIcon({ icon }: { icon: "file" | "check" | "play" | "eye" | "accept" }) {
  if (icon === "file") return <FileText size={16} aria-hidden="true" />;
  if (icon === "check") return <ClipboardCheck size={16} aria-hidden="true" />;
  if (icon === "play") return <Play size={16} aria-hidden="true" />;
  if (icon === "eye") return <Eye size={16} aria-hidden="true" />;
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
          </dl>
        </>
      ) : (
        <p>{text(result?.note, "No finding available.")}</p>
      )}
    </article>
  );
}

function EmptyState({ label }: { label: string }) {
  return <p className="emptyState">{label}</p>;
}
