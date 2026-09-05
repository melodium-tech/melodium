//! AI-facing guides bundled into the server: the Mélodium language/runtime
//! model, and CI/CD migration references. These are the same reference
//! documents shipped as the `melodium` Claude Code skill, embedded here so
//! any MCP client (not just Claude Code) can retrieve them.

const LANGUAGE_GUIDE_RAW: &str = include_str!("../../skills/melodium/SKILL.md");
const GITHUB_MIGRATION_GUIDE: &str =
    include_str!("../../skills/melodium/references/github-migration.md");
const GITLAB_MIGRATION_GUIDE: &str =
    include_str!("../../skills/melodium/references/gitlab-migration.md");

/// The language/runtime guide, with its Claude Code-specific YAML
/// frontmatter (name/description/allowed-tools/...) stripped.
pub fn language_guide() -> &'static str {
    strip_frontmatter(LANGUAGE_GUIDE_RAW)
}

pub fn cicd_migration_guide(source: CicdSource) -> &'static str {
    match source {
        CicdSource::Github => GITHUB_MIGRATION_GUIDE,
        CicdSource::Gitlab => GITLAB_MIGRATION_GUIDE,
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CicdSource {
    Github,
    Gitlab,
}

fn strip_frontmatter(content: &str) -> &str {
    let Some(rest) = content.strip_prefix("---\n") else {
        return content;
    };
    match rest.find("\n---\n") {
        Some(end) => rest[end + 5..].trim_start_matches('\n'),
        None => content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_guide_has_no_frontmatter() {
        let guide = language_guide();
        assert!(!guide.starts_with("---"));
        assert!(guide.contains("Mélodium Language and Technology"));
    }

    #[test]
    fn migration_guides_are_not_empty() {
        assert!(cicd_migration_guide(CicdSource::Github).contains("GitHub Actions"));
        assert!(cicd_migration_guide(CicdSource::Gitlab).contains("GitLab CI"));
    }
}
