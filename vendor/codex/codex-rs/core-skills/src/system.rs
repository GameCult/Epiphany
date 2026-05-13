use codex_utils_absolute_path::AbsolutePathBuf;

const SYSTEM_SKILLS_DIR_NAME: &str = ".system";
const SKILLS_DIR_NAME: &str = "skills";

/// Returns the legacy on-disk cache location for previously embedded system skills.
pub(crate) fn system_cache_root_dir(codex_home: &AbsolutePathBuf) -> AbsolutePathBuf {
    codex_home
        .join(SKILLS_DIR_NAME)
        .join(SYSTEM_SKILLS_DIR_NAME)
}

pub(crate) fn uninstall_system_skills(codex_home: &AbsolutePathBuf) {
    let _ = std::fs::remove_dir_all(system_cache_root_dir(codex_home));
}
