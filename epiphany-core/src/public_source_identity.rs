use anyhow::{Result, anyhow};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImmutableGithubSource {
    owner: String,
    repository: String,
    revision: String,
    path: String,
}

impl ImmutableGithubSource {
    pub fn from_components(
        owner: &str,
        repository: &str,
        revision: &str,
        path: &str,
    ) -> Result<Self> {
        validate_component(owner, "owner")?;
        validate_component(repository, "repository")?;
        if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(anyhow!(
                "GitHub revision must be an immutable 40-hex commit id"
            ));
        }
        validate_path(path)?;
        Ok(Self {
            owner: owner.into(),
            repository: repository.into(),
            revision: revision.to_ascii_lowercase(),
            path: path.into(),
        })
    }

    pub fn parse(source_ref: &str) -> Result<Self> {
        let rest = source_ref
            .strip_prefix("github://")
            .ok_or_else(|| anyhow!("immutable GitHub source must use github://"))?;
        let (repository_identity, target) = rest
            .split_once('@')
            .ok_or_else(|| anyhow!("immutable GitHub source requires an exact revision"))?;
        if target.contains('@') {
            return Err(anyhow!(
                "immutable GitHub source contains multiple revisions"
            ));
        }
        let (owner, repository) = repository_identity
            .split_once('/')
            .ok_or_else(|| anyhow!("immutable GitHub source requires owner/repository"))?;
        if repository.contains('/') {
            return Err(anyhow!("immutable GitHub repository identity is not exact"));
        }
        let (revision, path) = target
            .split_once('/')
            .ok_or_else(|| anyhow!("immutable GitHub source requires a repository path"))?;
        Self::from_components(owner, repository, revision, path)
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn repository_name(&self) -> &str {
        &self.repository
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn repository_ref(&self) -> String {
        format!("github://{}/{}", self.owner, self.repository)
    }
}

impl Display for ImmutableGithubSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}@{}/{}",
            self.repository_ref(),
            self.revision,
            self.path
        )
    }
}

fn validate_component(value: &str, name: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 100
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(anyhow!("GitHub {name} is invalid"));
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 512
        || path.starts_with('/')
        || path.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || !segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
    {
        return Err(anyhow!("GitHub repository path is invalid"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_and_round_trips() -> Result<()> {
        let source = ImmutableGithubSource::from_components(
            "GameCult",
            "Epiphany",
            "ABCDEF0123456789ABCDEF0123456789ABCDEF01",
            "docs/source_file.rs",
        )?;
        assert_eq!(
            source.revision(),
            "abcdef0123456789abcdef0123456789abcdef01"
        );
        assert_eq!(ImmutableGithubSource::parse(&source.to_string())?, source);
        Ok(())
    }

    #[test]
    fn refuses_mutable_or_ambiguous_identity() {
        for source_ref in [
            "https://github.com/GameCult/Epiphany",
            "github://GameCult/Epiphany@main/README.md",
            "github://GameCult/Epiphany@0123456789abcdef0123456789abcdef01234567/../secret",
            "github://GameCult/Epiphany@0123456789abcdef0123456789abcdef01234567//secret",
            "github://GameCult/other/repo@0123456789abcdef0123456789abcdef01234567/file",
        ] {
            assert!(
                ImmutableGithubSource::parse(source_ref).is_err(),
                "{source_ref}"
            );
        }
    }
}
