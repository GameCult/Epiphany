use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::RuntimeSpineEventOptions;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::RuntimeSpineJobOptions;
use epiphany_core::RuntimeSpineJobResultOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::append_runtime_event;
use epiphany_core::complete_runtime_job;
use epiphany_core::create_runtime_job;
use epiphany_core::create_runtime_session;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::runtime_spine_status;
use epiphany_core::write_runtime_hello_frame;
use std::env;
use std::path::PathBuf;
use uuid::Uuid;

const DEFAULT_STORE: &str = "state/runtime-spine.msgpack";

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command {
        Command::Init {
            runtime_id,
            display_name,
        } => {
            let identity = initialize_runtime_spine(
                &args.store,
                RuntimeSpineInitOptions {
                    runtime_id,
                    display_name,
                    created_at: now(),
                },
            )?;
            println!("runtime spine initialized");
            println!("store: {}", args.store.display());
            println!("runtime: {}", identity.runtime_id);
            println!(
                "documents: {}",
                identity.supported_document_types.join(", ")
            );
        }
        Command::Status => {
            let status = runtime_spine_status(&args.store)?;
            println!("runtime spine status");
            println!("store: {}", status.store);
            println!("present: {}", status.present);
            println!(
                "runtime: {}",
                status.runtime_id.unwrap_or_else(|| "missing".to_string())
            );
            println!(
                "display: {}",
                status.display_name.unwrap_or_else(|| "missing".to_string())
            );
            println!(
                "sessions: {} active: {}",
                status.sessions, status.active_sessions
            );
            println!("jobs: {} open: {}", status.jobs, status.open_jobs);
            println!("job results: {}", status.job_results);
            println!("events: {}", status.events);
            if !status.supported_document_types.is_empty() {
                println!("documents: {}", status.supported_document_types.join(", "));
            }
        }
        Command::OpenSession {
            session_id,
            objective,
            coordinator_note,
        } => {
            let session = create_runtime_session(
                &args.store,
                RuntimeSpineSessionOptions {
                    session_id,
                    objective,
                    created_at: now(),
                    coordinator_note,
                },
            )?;
            println!("runtime session opened");
            println!("session: {}", session.session_id);
            println!("objective: {}", session.objective);
        }
        Command::RecordEvent {
            event_id,
            event_type,
            source,
            session_id,
            job_id,
            summary,
        } => {
            let event = append_runtime_event(
                &args.store,
                RuntimeSpineEventOptions {
                    event_id,
                    occurred_at: now(),
                    event_type,
                    source,
                    session_id,
                    job_id,
                    summary,
                },
            )?;
            println!("runtime event recorded");
            println!("event: {}", event.event_id);
            println!("type: {}", event.event_type);
        }
        Command::OpenJob {
            job_id,
            session_id,
            role,
            summary,
            artifact_refs,
        } => {
            let job = create_runtime_job(
                &args.store,
                RuntimeSpineJobOptions {
                    job_id,
                    session_id,
                    role,
                    created_at: now(),
                    summary,
                    artifact_refs,
                },
            )?;
            println!("runtime job opened");
            println!("job: {}", job.job_id);
            println!("session: {}", job.session_id);
            println!("role: {}", job.role);
        }
        Command::CompleteJob {
            result_id,
            job_id,
            verdict,
            summary,
            next_safe_move,
            evidence_refs,
            artifact_refs,
        } => {
            let result = complete_runtime_job(
                &args.store,
                RuntimeSpineJobResultOptions {
                    result_id,
                    job_id,
                    completed_at: now(),
                    verdict,
                    summary,
                    next_safe_move,
                    evidence_refs,
                    artifact_refs,
                },
            )?;
            println!("runtime job completed");
            println!("result: {}", result.result_id);
            println!("job: {}", result.job_id);
            println!("verdict: {}", result.verdict);
        }
        Command::HelloFrame { output } => {
            let bytes = write_runtime_hello_frame(&args.store, &output)?;
            println!("cultnet hello frame written");
            println!("path: {}", output.display());
            println!("bytes: {bytes}");
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Args {
    store: PathBuf,
    command: Command,
}

#[derive(Debug)]
enum Command {
    Init {
        runtime_id: String,
        display_name: String,
    },
    Status,
    OpenSession {
        session_id: String,
        objective: String,
        coordinator_note: String,
    },
    RecordEvent {
        event_id: String,
        event_type: String,
        source: String,
        session_id: Option<String>,
        job_id: Option<String>,
        summary: String,
    },
    OpenJob {
        job_id: String,
        session_id: String,
        role: String,
        summary: String,
        artifact_refs: Vec<String>,
    },
    CompleteJob {
        result_id: String,
        job_id: String,
        verdict: String,
        summary: String,
        next_safe_move: String,
        evidence_refs: Vec<String>,
        artifact_refs: Vec<String>,
    },
    HelloFrame {
        output: PathBuf,
    },
}

impl Args {
    fn parse() -> Result<Self> {
        let mut store = PathBuf::from(DEFAULT_STORE);
        let mut positional = Vec::new();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--store" => store = take_path(&mut args, "--store")?,
                _ => positional.push(arg),
            }
        }
        let command = parse_command(positional)?;
        Ok(Self { store, command })
    }
}

fn parse_command(mut args: Vec<String>) -> Result<Command> {
    if args.is_empty() {
        return Err(anyhow!(usage()));
    }
    let command = args.remove(0);
    match command.as_str() {
        "init" => {
            let mut runtime_id = "epiphany-local".to_string();
            let mut display_name = "Epiphany Local".to_string();
            parse_options(args, |name, value| match name {
                "--runtime-id" => {
                    runtime_id = value;
                    Ok(())
                }
                "--display-name" => {
                    display_name = value;
                    Ok(())
                }
                _ => Err(anyhow!("unknown init argument: {name}")),
            })?;
            Ok(Command::Init {
                runtime_id,
                display_name,
            })
        }
        "status" => Ok(Command::Status),
        "open-session" => {
            let mut session_id = format!("session-{}", Uuid::new_v4());
            let mut objective = String::new();
            let mut coordinator_note = String::new();
            parse_options(args, |name, value| match name {
                "--session-id" => {
                    session_id = value;
                    Ok(())
                }
                "--objective" => {
                    objective = value;
                    Ok(())
                }
                "--coordinator-note" => {
                    coordinator_note = value;
                    Ok(())
                }
                _ => Err(anyhow!("unknown open-session argument: {name}")),
            })?;
            if objective.trim().is_empty() {
                return Err(anyhow!("open-session requires --objective"));
            }
            Ok(Command::OpenSession {
                session_id,
                objective,
                coordinator_note,
            })
        }
        "record-event" => {
            let mut event_id = format!("event-{}", Uuid::new_v4());
            let mut event_type = String::new();
            let mut source = "operator".to_string();
            let mut session_id = None;
            let mut job_id = None;
            let mut summary = String::new();
            parse_options(args, |name, value| match name {
                "--event-id" => {
                    event_id = value;
                    Ok(())
                }
                "--type" => {
                    event_type = value;
                    Ok(())
                }
                "--source" => {
                    source = value;
                    Ok(())
                }
                "--session-id" => {
                    session_id = Some(value);
                    Ok(())
                }
                "--job-id" => {
                    job_id = Some(value);
                    Ok(())
                }
                "--summary" => {
                    summary = value;
                    Ok(())
                }
                _ => Err(anyhow!("unknown record-event argument: {name}")),
            })?;
            if event_type.trim().is_empty() {
                return Err(anyhow!("record-event requires --type"));
            }
            Ok(Command::RecordEvent {
                event_id,
                event_type,
                source,
                session_id,
                job_id,
                summary,
            })
        }
        "open-job" => {
            let mut job_id = format!("job-{}", Uuid::new_v4());
            let mut session_id = String::new();
            let mut role = String::new();
            let mut summary = String::new();
            let mut artifact_refs = Vec::new();
            parse_options(args, |name, value| match name {
                "--job-id" => {
                    job_id = value;
                    Ok(())
                }
                "--session-id" => {
                    session_id = value;
                    Ok(())
                }
                "--role" => {
                    role = value;
                    Ok(())
                }
                "--summary" => {
                    summary = value;
                    Ok(())
                }
                "--artifact-ref" => {
                    artifact_refs.push(value);
                    Ok(())
                }
                _ => Err(anyhow!("unknown open-job argument: {name}")),
            })?;
            if session_id.trim().is_empty() {
                return Err(anyhow!("open-job requires --session-id"));
            }
            if role.trim().is_empty() {
                return Err(anyhow!("open-job requires --role"));
            }
            Ok(Command::OpenJob {
                job_id,
                session_id,
                role,
                summary,
                artifact_refs,
            })
        }
        "complete-job" => {
            let mut result_id = format!("result-{}", Uuid::new_v4());
            let mut job_id = String::new();
            let mut verdict = String::new();
            let mut summary = String::new();
            let mut next_safe_move = String::new();
            let mut evidence_refs = Vec::new();
            let mut artifact_refs = Vec::new();
            parse_options(args, |name, value| match name {
                "--result-id" => {
                    result_id = value;
                    Ok(())
                }
                "--job-id" => {
                    job_id = value;
                    Ok(())
                }
                "--verdict" => {
                    verdict = value;
                    Ok(())
                }
                "--summary" => {
                    summary = value;
                    Ok(())
                }
                "--next-safe-move" => {
                    next_safe_move = value;
                    Ok(())
                }
                "--evidence-ref" => {
                    evidence_refs.push(value);
                    Ok(())
                }
                "--artifact-ref" => {
                    artifact_refs.push(value);
                    Ok(())
                }
                _ => Err(anyhow!("unknown complete-job argument: {name}")),
            })?;
            if job_id.trim().is_empty() {
                return Err(anyhow!("complete-job requires --job-id"));
            }
            if verdict.trim().is_empty() {
                return Err(anyhow!("complete-job requires --verdict"));
            }
            if summary.trim().is_empty() {
                return Err(anyhow!("complete-job requires --summary"));
            }
            Ok(Command::CompleteJob {
                result_id,
                job_id,
                verdict,
                summary,
                next_safe_move,
                evidence_refs,
                artifact_refs,
            })
        }
        "hello-frame" => {
            let mut output = PathBuf::from(".epiphany-dogfood/runtime-spine/hello.cultnet");
            parse_options(args, |name, value| match name {
                "--output" => {
                    output = PathBuf::from(value);
                    Ok(())
                }
                _ => Err(anyhow!("unknown hello-frame argument: {name}")),
            })?;
            Ok(Command::HelloFrame { output })
        }
        _ => Err(anyhow!(usage())),
    }
}

fn parse_options(
    args: Vec<String>,
    mut on_option: impl FnMut(&str, String) -> Result<()>,
) -> Result<()> {
    let mut args = args.into_iter();
    while let Some(name) = args.next() {
        if !name.starts_with("--") {
            return Err(anyhow!("unexpected positional argument: {name}"));
        }
        let value = args
            .next()
            .with_context(|| format!("{name} requires a value"))?;
        on_option(&name, value)?;
    }
    Ok(())
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn usage() -> &'static str {
    "usage: epiphany-runtime-spine [--store path] <init|status|open-session|open-job|complete-job|record-event|hello-frame>"
}
