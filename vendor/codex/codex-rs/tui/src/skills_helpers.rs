use codex_core_skills::model::SkillMetadata;

pub(crate) fn skill_display_name(skill: &SkillMetadata) -> String {
    if let Some(display_name) = skill
        .interface
        .as_ref()
        .and_then(|interface| interface.display_name.as_deref())
    {
        return display_name.to_string();
    }

    if let Some((plugin_name, skill_name)) = skill.name.split_once(':')
        && !plugin_name.is_empty()
        && !skill_name.is_empty()
    {
        return format!("{skill_name} ({plugin_name})");
    }

    skill.name.clone()
}
