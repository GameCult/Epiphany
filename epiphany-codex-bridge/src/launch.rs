//! Compatibility facade for older bridge callers.
//!
//! Epiphany prompt doctrine, worker launch policy, role binding ids, and output
//! schemas live in `epiphany-core`. The bridge may translate those typed launch
//! requests to Codex JSON-RPC, but it does not own the agent instructions.

pub use epiphany_core::EPIPHANY_IMAGINATION_OWNER_ROLE;
pub use epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID;
pub use epiphany_core::EPIPHANY_IMPLEMENTATION_OWNER_ROLE;
pub use epiphany_core::EPIPHANY_IMPLEMENTATION_ROLE_BINDING_ID;
pub use epiphany_core::EPIPHANY_MODELING_OWNER_ROLE;
pub use epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID;
pub use epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
pub use epiphany_core::EPIPHANY_REORIENT_OWNER_ROLE;
pub use epiphany_core::EPIPHANY_RESEARCH_OWNER_ROLE;
pub use epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID;
pub use epiphany_core::EPIPHANY_VERIFICATION_OWNER_ROLE;
pub use epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID;
pub use epiphany_core::EpiphanyCoordinatorPromptConfig;
pub use epiphany_core::EpiphanyCrrcPromptConfig;
pub use epiphany_core::EpiphanyImplementationPromptConfig;
pub use epiphany_core::EpiphanyReorientationPromptConfig;
pub use epiphany_core::EpiphanyRolePromptConfig;
pub use epiphany_core::EpiphanySharedPromptConfig;
pub use epiphany_core::EpiphanySpecialistPromptConfig;
pub use epiphany_core::build_epiphany_job_launch_request;
pub use epiphany_core::build_epiphany_reorient_launch_instruction;
pub use epiphany_core::build_epiphany_reorient_launch_request;
pub use epiphany_core::build_epiphany_role_launch_request;
pub use epiphany_core::epiphany_agent_prompt_with_memory;
pub use epiphany_core::epiphany_reorient_launch_output_schema;
pub use epiphany_core::epiphany_role_binding_id;
pub use epiphany_core::epiphany_role_label;
pub use epiphany_core::epiphany_role_launch_output_schema;
pub use epiphany_core::epiphany_role_owner;
pub use epiphany_core::epiphany_specialist_prompt_config;
pub use epiphany_core::epiphany_worker_prompt;
pub use epiphany_core::render_epiphany_coordinator_note;
pub use epiphany_core::render_epiphany_pre_compaction_checkpoint_intervention;
pub use epiphany_core::unique_strings;
