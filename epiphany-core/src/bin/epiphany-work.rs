use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use std::{env, path::PathBuf};

const HISTORICAL_REASON: &str = "canonical frontier workflow required";

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    let parsed = parse_legacy_args(args)?;
    let result = match command.as_str() {
        "overview" | "proof-bundle" | "status" => run_overview(parsed)?,
        "accept" | "derive-plan" | "imagine" | "plan-from-pressure" | "plan" | "run" | "adopt"
        | "promote" | "execute" | "exec" | "verify" | "soul-verify" | "close" | "closure"
        | "verify-close" | "readiness" | "readiness-report" | "mvp-readiness" | "export-proof"
        | "public-proof" | "tick" | "pulse" | "schedule" | "queue-run" | "run-queue"
        | "queue-tick" | "scheduler-run" | "serve" | "loop" | "daemon" => {
            return Err(canonical_frontier_required(&command));
        }
        other => return Err(anyhow!("unknown epiphany-work command {other:?}")),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Debug, Default)]
struct LegacyArgs {
    workspace: Option<PathBuf>,
    item: Option<String>,
}

fn parse_legacy_args(args: impl Iterator<Item = String>) -> Result<LegacyArgs> {
    let mut parsed = LegacyArgs::default();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => {
                parsed.workspace = Some(PathBuf::from(required_value(&mut args, &arg)?))
            }
            "--item" => parsed.item = Some(required_value(&mut args, &arg)?),
            flag if flag.starts_with("--") => {
                if args.peek().is_some_and(|v| !v.starts_with("--")) {
                    args.next();
                }
            }
            positional => return Err(anyhow!("unexpected positional argument {positional:?}")),
        }
    }
    Ok(parsed)
}

fn required_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<String> {
    args.next()
        .filter(|v| !v.starts_with("--"))
        .ok_or_else(|| anyhow!("{flag} requires a value"))
}

fn canonical_frontier_required(operation: &str) -> anyhow::Error {
    anyhow!("{operation} is historical-only: {HISTORICAL_REASON}")
}

fn run_overview(args: LegacyArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .ok_or_else(|| anyhow!("overview requires --workspace"))?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_historical_overview.v0",
        "status": "historical-only", "reason": HISTORICAL_REASON,
        "workspace": workspace, "item": args.item,
        "authority": { "durableStateAdmitted": false, "readinessApprovalAuthorized": false,
            "publicationAuthorized": false, "serviceLifecycleAuthority": false, "handsActionAuthorized": false },
        "privateStateExposed": false
    }))
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-work <legacy-command> [options]\nLegacy mutations are historical-only.\nRead history: epiphany-work overview --workspace <path> [--item <id>]"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutation_refusal_and_history_are_explicit() -> Result<()> {
        assert!(
            canonical_frontier_required("plan")
                .to_string()
                .contains(HISTORICAL_REASON)
        );
        let row = run_overview(LegacyArgs {
            workspace: Some(PathBuf::from("E:/test")),
            item: None,
        })?;
        assert_eq!(row["status"], "historical-only");
        assert_eq!(row["authority"]["handsActionAuthorized"], false);
        Ok(())
    }
}
