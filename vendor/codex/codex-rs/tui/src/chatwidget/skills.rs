use std::collections::HashMap;
use std::collections::HashSet;

use super::ChatWidget;
use codex_core_skills::model::SkillMetadata;
use codex_protocol::parse_command::ParsedCommand;

const TOOL_MENTION_SIGIL: char = '$';

impl ChatWidget {
    pub(crate) fn annotate_skill_reads_in_parsed_cmd(
        &self,
        mut parsed_cmd: Vec<ParsedCommand>,
    ) -> Vec<ParsedCommand> {
        if self.skills_all.is_empty() {
            return parsed_cmd;
        }

        for parsed in &mut parsed_cmd {
            let ParsedCommand::Read { name, path, .. } = parsed else {
                continue;
            };
            if name != "SKILL.md" {
                continue;
            }

            // Best effort only: annotate exact SKILL.md path matches from the loaded skills list.
            if let Some(skill) = self
                .skills_all
                .iter()
                .find(|skill| skill.path.as_path() == path)
            {
                *name = format!("{name} ({} skill)", skill.name);
            }
        }

        parsed_cmd
    }
}

pub(crate) fn collect_tool_mentions(
    text: &str,
    mention_paths: &HashMap<String, String>,
) -> ToolMentions {
    let mut mentions = extract_tool_mentions_from_text(text);
    for (name, path) in mention_paths {
        if mentions.names.contains(name) {
            mentions.linked_paths.insert(name.clone(), path.clone());
        }
    }
    mentions
}

pub(crate) fn find_skill_mentions_with_tool_mentions(
    mentions: &ToolMentions,
    skills: &[SkillMetadata],
) -> Vec<SkillMetadata> {
    let mention_skill_paths: HashSet<&str> = mentions
        .linked_paths
        .values()
        .filter(|path| is_skill_path(path))
        .map(|path| normalize_skill_path(path))
        .collect();

    let mut seen_names = HashSet::new();
    let mut seen_paths = HashSet::new();
    let mut matches: Vec<SkillMetadata> = Vec::new();

    for skill in skills {
        if seen_paths.contains(&skill.path_to_skills_md) {
            continue;
        }
        let path_str = skill.path_to_skills_md.to_string_lossy();
        if mention_skill_paths.contains(path_str.as_ref()) {
            seen_paths.insert(skill.path_to_skills_md.clone());
            seen_names.insert(skill.name.clone());
            matches.push(skill.clone());
        }
    }

    for skill in skills {
        if seen_paths.contains(&skill.path_to_skills_md) {
            continue;
        }
        if mentions.names.contains(&skill.name) && seen_names.insert(skill.name.clone()) {
            seen_paths.insert(skill.path_to_skills_md.clone());
            matches.push(skill.clone());
        }
    }

    matches
}

pub(crate) struct ToolMentions {
    names: HashSet<String>,
    linked_paths: HashMap<String, String>,
}

fn extract_tool_mentions_from_text(text: &str) -> ToolMentions {
    extract_tool_mentions_from_text_with_sigil(text, TOOL_MENTION_SIGIL)
}

fn extract_tool_mentions_from_text_with_sigil(text: &str, sigil: char) -> ToolMentions {
    let text_bytes = text.as_bytes();
    let mut names: HashSet<String> = HashSet::new();
    let mut linked_paths: HashMap<String, String> = HashMap::new();

    let mut index = 0;
    while index < text_bytes.len() {
        let byte = text_bytes[index];
        if byte == b'['
            && let Some((name, path, end_index)) =
                parse_linked_tool_mention(text, text_bytes, index, sigil)
        {
            if !is_common_env_var(name) {
                if is_skill_path(path) {
                    names.insert(name.to_string());
                }
                linked_paths
                    .entry(name.to_string())
                    .or_insert(path.to_string());
            }
            index = end_index;
            continue;
        }

        if byte != sigil as u8 {
            index += 1;
            continue;
        }

        let name_start = index + 1;
        let Some(first_name_byte) = text_bytes.get(name_start) else {
            index += 1;
            continue;
        };
        if !is_mention_name_char(*first_name_byte) {
            index += 1;
            continue;
        }

        let mut name_end = name_start + 1;
        while let Some(next_byte) = text_bytes.get(name_end)
            && is_mention_name_char(*next_byte)
        {
            name_end += 1;
        }

        let name = &text[name_start..name_end];
        if !is_common_env_var(name) {
            names.insert(name.to_string());
        }
        index = name_end;
    }

    ToolMentions {
        names,
        linked_paths,
    }
}

fn parse_linked_tool_mention<'a>(
    text: &'a str,
    text_bytes: &[u8],
    start: usize,
    sigil: char,
) -> Option<(&'a str, &'a str, usize)> {
    let sigil_index = start + 1;
    if text_bytes.get(sigil_index) != Some(&(sigil as u8)) {
        return None;
    }

    let name_start = sigil_index + 1;
    let first_name_byte = text_bytes.get(name_start)?;
    if !is_mention_name_char(*first_name_byte) {
        return None;
    }

    let mut name_end = name_start + 1;
    while let Some(next_byte) = text_bytes.get(name_end)
        && is_mention_name_char(*next_byte)
    {
        name_end += 1;
    }

    if text_bytes.get(name_end) != Some(&b']') {
        return None;
    }

    let mut path_start = name_end + 1;
    while let Some(next_byte) = text_bytes.get(path_start)
        && next_byte.is_ascii_whitespace()
    {
        path_start += 1;
    }
    if text_bytes.get(path_start) != Some(&b'(') {
        return None;
    }

    let mut path_end = path_start + 1;
    while let Some(next_byte) = text_bytes.get(path_end)
        && *next_byte != b')'
    {
        path_end += 1;
    }
    if text_bytes.get(path_end) != Some(&b')') {
        return None;
    }

    let path = text[path_start + 1..path_end].trim();
    if path.is_empty() {
        return None;
    }

    let name = &text[name_start..name_end];
    Some((name, path, path_end + 1))
}

fn is_common_env_var(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "PATH"
            | "HOME"
            | "USER"
            | "SHELL"
            | "PWD"
            | "TMPDIR"
            | "TEMP"
            | "TMP"
            | "LANG"
            | "TERM"
            | "XDG_CONFIG_HOME"
    )
}

fn is_mention_name_char(byte: u8) -> bool {
    matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-')
}

fn is_skill_path(path: &str) -> bool {
    path.starts_with("skill://")
        || (!path.contains("://")
            && path
                .rsplit(['/', '\\'])
                .next()
                .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md")))
}

fn normalize_skill_path(path: &str) -> &str {
    path.strip_prefix("skill://").unwrap_or(path)
}
