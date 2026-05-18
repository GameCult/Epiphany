use super::EpiphanyMemoryProfile;
use sha1::Digest;
use sha1::Sha1;

pub fn memory_graph_domain_id(
    profile: EpiphanyMemoryProfile,
    kind: impl AsRef<str>,
    path_or_name: impl AsRef<str>,
) -> String {
    let normalized = normalized_key(path_or_name.as_ref());
    stable_memory_graph_id(
        "memdom",
        [profile_key(profile), kind.as_ref(), normalized.as_str()],
    )
}

pub fn memory_graph_node_id(
    domain_id: impl AsRef<str>,
    kind: impl AsRef<str>,
    path: impl AsRef<str>,
    symbol: Option<&str>,
) -> String {
    let normalized = normalized_key(path.as_ref());
    stable_memory_graph_id(
        "memnode",
        [
            domain_id.as_ref(),
            kind.as_ref(),
            normalized.as_str(),
            symbol.unwrap_or_default(),
        ],
    )
}

pub fn memory_graph_edge_id(
    source_id: impl AsRef<str>,
    target_id: impl AsRef<str>,
    kind: impl AsRef<str>,
    anchor_keys: impl IntoIterator<Item = impl AsRef<str>>,
) -> String {
    let mut parts = vec![
        source_id.as_ref().to_string(),
        target_id.as_ref().to_string(),
        kind.as_ref().to_string(),
    ];
    let mut anchors = anchor_keys
        .into_iter()
        .map(|key| normalized_key(key.as_ref()))
        .collect::<Vec<_>>();
    anchors.sort();
    parts.extend(anchors);
    stable_memory_graph_id("memedge", parts.iter().map(String::as_str))
}

pub(crate) fn stable_memory_graph_id<'a>(
    prefix: &str,
    parts: impl IntoIterator<Item = &'a str>,
) -> String {
    let mut hasher = Sha1::new();
    for part in parts {
        hasher.update(part.trim().to_lowercase().as_bytes());
        hasher.update([0]);
    }
    format!("{prefix}-{:x}", hasher.finalize())
}

fn profile_key(profile: EpiphanyMemoryProfile) -> &'static str {
    match profile {
        EpiphanyMemoryProfile::RepoArchitecture => "repo_architecture",
        EpiphanyMemoryProfile::RepoDataflow => "repo_dataflow",
        EpiphanyMemoryProfile::RoleSelf => "role_self",
        EpiphanyMemoryProfile::ShortTerm => "short_term",
        EpiphanyMemoryProfile::Incubation => "incubation",
        EpiphanyMemoryProfile::AgencyPressure => "agency_pressure",
        EpiphanyMemoryProfile::CandidateIntervention => "candidate_intervention",
        EpiphanyMemoryProfile::Identity => "identity",
        EpiphanyMemoryProfile::Evidence => "evidence",
    }
}

pub(crate) fn normalized_key(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_lowercase()
}
