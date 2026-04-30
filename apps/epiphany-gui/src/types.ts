export interface StatusRequest {
  threadId?: string;
  cwd?: string;
  codexHome?: string;
  appServer?: string;
}

export type OperatorAction =
  | "statusSnapshot"
  | "coordinatorPlan"
  | "prepareCheckpoint"
  | "launchModeling"
  | "readModelingResult"
  | "acceptModeling"
  | "launchVerification"
  | "readVerificationResult"
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
  modifiedMillis?: number;
}

export interface OperatorSnapshot {
  generatedAt: string;
  repoRoot: string;
  status: any;
  artifacts: ArtifactBundle[];
}
