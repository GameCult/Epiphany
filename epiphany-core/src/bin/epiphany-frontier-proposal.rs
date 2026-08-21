use anyhow::{Context, Result, anyhow, bail};
use epiphany_core::{
    RepoFrontierUserProposalInput, intake_user_repo_frontier_proposal,
    select_repo_frontier_work_proposal_for_modeling,
};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};

fn main() -> Result<()> {
    let args = Args::parse(std::env::args().skip(1))?;
    let proposal = intake_user_repo_frontier_proposal(
        &args.store,
        RepoFrontierUserProposalInput {
            proposal_id: args.required("--proposal-id")?,
            source_actor: args.required("--source-actor")?,
            source_ref: args.required("--source-ref")?,
            repository: args.required("--repository")?,
            workspace: args.required("--workspace")?,
            thread_id: args.required("--thread-id")?,
            runtime_id: args.required("--runtime-id")?,
            title: args.required("--title")?,
            body: args.required("--body")?,
            desired_outcome: args.required("--desired-outcome")?,
            constraints: args.constraints.clone(),
            scope_hints: args.scope_hints.clone(),
            evidence_refs: args.evidence_refs.clone(),
            public_source_refs: args.public_source_refs.clone(),
            proposed_at: args.required("--proposed-at")?,
            private_state_included: false,
        },
    )?;
    let selection = select_repo_frontier_work_proposal_for_modeling(
        &args.store,
        &proposal.proposal_id,
        &args.required("--selected-at")?,
    )?;
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schemaVersion": selection.schema_version,
            "status": "selected-for-modeling",
            "proposalId": proposal.proposal_id,
            "proposalPayloadSha256": proposal.payload_sha256,
            "proposalModelingRequestId": selection.request_id,
            "runtimeId": selection.runtime_id,
            "threadId": selection.thread_id,
            "repository": selection.repository,
            "workspace": selection.workspace,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

struct Args {
    store: PathBuf,
    fields: BTreeMap<String, String>,
    constraints: Vec<String>,
    scope_hints: Vec<String>,
    evidence_refs: Vec<String>,
    public_source_refs: Vec<String>,
}

impl Args {
    fn parse(values: impl Iterator<Item = String>) -> Result<Self> {
        let mut values = values;
        let command = values
            .next()
            .context("missing command: expected intake-select")?;
        if command != "intake-select" {
            bail!("unknown command {command:?}; expected intake-select");
        }
        let mut fields = BTreeMap::new();
        let mut constraints = Vec::new();
        let mut scope_hints = Vec::new();
        let mut evidence_refs = Vec::new();
        let mut public_source_refs = Vec::new();
        while let Some(flag) = values.next() {
            let value = values
                .next()
                .with_context(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--constraint" => constraints.push(value),
                "--scope-hint" => scope_hints.push(value),
                "--evidence-ref" => evidence_refs.push(value),
                "--public-source-ref" => public_source_refs.push(value),
                "--store" | "--proposal-id" | "--source-actor" | "--source-ref"
                | "--repository" | "--workspace" | "--thread-id" | "--runtime-id" | "--title"
                | "--body" | "--desired-outcome" | "--proposed-at" | "--selected-at" => {
                    if fields.insert(flag.clone(), value).is_some() {
                        bail!("duplicate proposal argument {flag}");
                    }
                }
                _ => bail!("unknown proposal argument {flag}"),
            }
        }
        let store = PathBuf::from(fields.remove("--store").context("missing --store")?);
        if !store.is_absolute() {
            return Err(anyhow!("proposal store must be absolute"));
        }
        Ok(Self {
            store,
            fields,
            constraints,
            scope_hints,
            evidence_refs,
            public_source_refs,
        })
    }

    fn required(&self, name: &str) -> Result<String> {
        self.fields
            .get(name)
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .ok_or_else(|| anyhow!("missing {name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_cli_keeps_repeated_typed_lists_and_forbids_private_state() {
        let args = Args::parse(
            [
                "intake-select",
                "--store",
                "C:/state/runtime.cc",
                "--constraint",
                "one",
                "--constraint",
                "two",
                "--scope-hint",
                "src/lib.rs",
                "--evidence-ref",
                "operator:request",
                "--public-source-ref",
                "github://GameCult/Epiphany@0123456789abcdef0123456789abcdef01234567/README.md",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect("typed proposal args");

        assert_eq!(args.constraints, ["one", "two"]);
        assert_eq!(args.scope_hints, ["src/lib.rs"]);
        assert_eq!(args.evidence_refs, ["operator:request"]);
        assert_eq!(args.public_source_refs.len(), 1);
    }

    #[test]
    fn proposal_cli_rejects_relative_store_and_unknown_authority_flags() {
        assert!(
            Args::parse(
                ["intake-select", "--store", "state/runtime.cc"]
                    .into_iter()
                    .map(str::to_string)
            )
            .is_err()
        );
        assert!(
            Args::parse(
                [
                    "intake-select",
                    "--store",
                    "C:/state/runtime.cc",
                    "--grant-hands",
                    "yes",
                ]
                .into_iter()
                .map(str::to_string)
            )
            .is_err()
        );
    }
}
