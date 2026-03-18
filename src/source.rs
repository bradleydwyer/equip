use std::path::PathBuf;

#[derive(Debug)]
pub enum SkillSource {
    GitHub {
        owner: String,
        repo: String,
        subpath: Option<String>,
    },
    GitUrl {
        url: String,
    },
    Local {
        path: PathBuf,
    },
}

impl SkillSource {
    pub fn parse(source: &str) -> Result<Self, String> {
        // Local path: starts with /, ./, ../, or ~
        if source.starts_with('/')
            || source.starts_with("./")
            || source.starts_with("../")
            || source.starts_with('~')
            || source == "."
        {
            let path = if let Some(rest) = source.strip_prefix('~') {
                let home = std::env::var("HOME")
                    .map_err(|_| "HOME environment variable not set".to_string())?;
                PathBuf::from(home).join(rest.strip_prefix('/').unwrap_or(rest))
            } else {
                PathBuf::from(source)
                    .canonicalize()
                    .map_err(|e| format!("Invalid path '{}': {}", source, e))?
            };
            return Ok(SkillSource::Local { path });
        }

        // Git URL: starts with git@, git://, http://, https://, or ends with .git
        if source.starts_with("git@")
            || source.starts_with("git://")
            || source.starts_with("http://")
            || source.starts_with("https://")
            || source.ends_with(".git")
        {
            return Ok(SkillSource::GitUrl {
                url: source.to_string(),
            });
        }

        // GitHub shorthand: owner/repo or owner/repo/subpath
        let parts: Vec<&str> = source.splitn(3, '/').collect();
        match parts.len() {
            2 | 3 => {
                let owner = parts[0];
                let repo = parts[1];
                if owner.is_empty() || repo.is_empty() {
                    return Err(format!(
                        "Invalid source '{}': owner and repo must not be empty.",
                        source
                    ));
                }
                let subpath = if parts.len() == 3 {
                    let sp = parts[2];
                    if sp.is_empty() {
                        None
                    } else {
                        Some(sp.to_string())
                    }
                } else {
                    None
                };
                Ok(SkillSource::GitHub {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    subpath,
                })
            }
            _ => Err(format!(
                "Invalid source '{}'. Expected: owner/repo, a git URL, or a local path.",
                source
            )),
        }
    }

    pub fn repo_url(&self) -> Option<String> {
        match self {
            SkillSource::GitHub { owner, repo, .. } => {
                Some(format!("https://github.com/{}/{}", owner, repo))
            }
            SkillSource::GitUrl { url } => Some(url.clone()),
            SkillSource::Local { .. } => None,
        }
    }

    pub fn git_clone_url(&self) -> Option<String> {
        match self {
            SkillSource::GitHub { owner, repo, .. } => {
                Some(format!("https://github.com/{}/{}.git", owner, repo))
            }
            SkillSource::GitUrl { url } => Some(url.clone()),
            SkillSource::Local { .. } => None,
        }
    }

    pub fn subpath(&self) -> Option<&str> {
        match self {
            SkillSource::GitHub { subpath, .. } => subpath.as_deref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_shorthand() {
        let source = SkillSource::parse("anthropics/skills").unwrap();
        match source {
            SkillSource::GitHub {
                owner,
                repo,
                subpath,
            } => {
                assert_eq!(owner, "anthropics");
                assert_eq!(repo, "skills");
                assert!(subpath.is_none());
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn parse_github_with_subpath() {
        let source = SkillSource::parse("anthropics/skills/pdf").unwrap();
        match source {
            SkillSource::GitHub {
                owner,
                repo,
                subpath,
            } => {
                assert_eq!(owner, "anthropics");
                assert_eq!(repo, "skills");
                assert_eq!(subpath.as_deref(), Some("pdf"));
            }
            _ => panic!("Expected GitHub source"),
        }
    }

    #[test]
    fn parse_git_url_https() {
        let source = SkillSource::parse("https://github.com/foo/bar.git").unwrap();
        match source {
            SkillSource::GitUrl { url } => {
                assert_eq!(url, "https://github.com/foo/bar.git");
            }
            _ => panic!("Expected GitUrl source"),
        }
    }

    #[test]
    fn parse_git_url_ssh() {
        let source = SkillSource::parse("git@github.com:foo/bar.git").unwrap();
        match source {
            SkillSource::GitUrl { url } => {
                assert_eq!(url, "git@github.com:foo/bar.git");
            }
            _ => panic!("Expected GitUrl source"),
        }
    }

    #[test]
    fn parse_local_relative() {
        // Can't easily test canonicalize without a real path, so test the detection
        let source = SkillSource::parse("./my-skill");
        // Will fail due to path not existing, but should try Local path
        assert!(source.is_err()); // canonicalize fails on non-existent path
    }

    #[test]
    fn parse_local_absolute() {
        let source = SkillSource::parse("/tmp").unwrap();
        match source {
            SkillSource::Local { path } => {
                // canonicalize resolves symlinks, so /tmp may become /private/tmp on macOS
                assert!(path.ends_with("tmp"));
            }
            _ => panic!("Expected Local source"),
        }
    }

    #[test]
    fn repo_url_github() {
        let source = SkillSource::GitHub {
            owner: "anthropics".to_string(),
            repo: "skills".to_string(),
            subpath: None,
        };
        assert_eq!(
            source.repo_url(),
            Some("https://github.com/anthropics/skills".to_string())
        );
    }
}
