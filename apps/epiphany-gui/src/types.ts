export interface StatusRequest {
  threadId?: string;
  cwd?: string;
  codexHome?: string;
  appServer?: string;
}

export type OperatorAction =
  | "statusSnapshot"
  | "coordinatorPlan"
  | "inspectUnity"
  | "prepareCheckpoint"
  | "continueImplementation"
  | "launchModeling"
  | "readModelingResult"
  | "acceptModeling"
  | "launchVerification"
  | "readVerificationResult"
  | "acceptVerification"
  | "launchReorient"
  | "readReorientResult"
  | "acceptReorient";

export interface OperatorActionResult {
  action: OperatorAction;
  artifactPath: string;
  summary: string;
  threadId?: string;
}

export interface ArtifactBundle {
  name: string;
  path: string;
  files: string[];
  summaryPath?: string;
  finalStatusPath?: string;
  comparisonPath?: string;
  implementationAudit?: {
    resultPath: string;
    workspaceChanged: boolean;
    trackedDiffPresent: boolean;
    changedFiles: string[];
  };
  runtimeAudit?: {
    resultPath: string;
    status: string;
    projectVersion?: string;
    editorPath?: string;
  };
  modifiedMillis?: number;
}

export interface OperatorSnapshot {
  generatedAt: string;
  repoRoot: string;
  status: any;
  artifacts: ArtifactBundle[];
}
