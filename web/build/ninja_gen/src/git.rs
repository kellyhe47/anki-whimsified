// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::process::Command;
use std::sync::LazyLock;

use anyhow::Result;
use itertools::Itertools;

use super::*;
use crate::action::BuildAction;
use crate::input::BuildInput;

pub struct SyncSubmodule {
    pub path: &'static str,
    pub offline_build: bool,
}

impl BuildAction for SyncSubmodule {
    fn command(&self) -> &str {
        if self.offline_build {
            "echo OFFLINE_BUILD is set, skipping git repository update for $path"
        } else {
            "git -c protocol.file.allow=always submodule update --checkout --init $path"
        }
    }

    fn files(&mut self, build: &mut impl build::FilesHandle) {
        if !self.offline_build {
            if let Some(head) = locate_git_head() {
                build.add_inputs("", head);
            } else {
                println!("Warning, .git/HEAD not found; submodules may be stale");
            }
        }

        build.add_variable("path", self.path);
        build.add_output_stamp(format!("git/{}", self.path));
    }

    fn on_first_instance(&self, build: &mut Build) -> Result<()> {
        build.pool("git", 1);
        Ok(())
    }

    fn concurrency_pool(&self) -> Option<&'static str> {
        Some("git")
    }
}

static GIT_HEAD: LazyLock<Option<BuildInput>> = LazyLock::new(locate_git_head_uncached);
static GIT_DIR: LazyLock<Option<BuildInput>> = LazyLock::new(locate_git_dir_uncached);

/// Locate the repository metadata directory used as a Ninja dependency.
pub(crate) fn locate_git_dir() -> Option<BuildInput> {
    GIT_DIR.clone()
}

fn locate_git_dir_uncached() -> Option<BuildInput> {
    let standard_path = Utf8Path::new(".git");
    if standard_path.exists() {
        return Some(inputs![standard_path.to_string()]);
    }

    git_path(&["rev-parse", "--git-dir"])
        .filter(|path| path.exists())
        .map(ninja_input)
}

/// Locate the repository HEAD used as a Ninja dependency.
///
/// Prefer the standard path, ask Git for worktree or monorepo layouts, then
/// fall back to scanning parent submodule metadata.
pub(crate) fn locate_git_head() -> Option<BuildInput> {
    GIT_HEAD.clone()
}

fn locate_git_head_uncached() -> Option<BuildInput> {
    let standard_path = Utf8Path::new(".git/HEAD");
    if standard_path.exists() {
        return Some(inputs![standard_path.to_string()]);
    }

    if let Some(path) = git_path(&["rev-parse", "--git-path", "HEAD"]) {
        if path.exists() {
            return Some(ninja_input(path));
        }
    }

    let mut folder = Utf8PathBuf::from_path_buf(
        dunce::canonicalize(Utf8Path::new(".").canonicalize().unwrap()).unwrap(),
    )
    .unwrap();
    loop {
        let path = folder.join(".git").join("modules");
        if path.exists() {
            let heads = path
                .read_dir_utf8()
                .unwrap()
                .filter_map(|p| {
                    let head = p.unwrap().path().join("HEAD");
                    if head.exists() {
                        Some(head.as_str().replace(':', "$:"))
                    } else {
                        None
                    }
                })
                .collect_vec();
            return Some(inputs![heads]);
        }
        if let Some(parent) = folder.parent() {
            folder = parent.to_owned();
        } else {
            return None;
        }
    }
}

fn git_path(args: &[&str]) -> Option<Utf8PathBuf> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_git_path(&output.stdout).map(Utf8Path::to_owned)
}

fn parse_git_path(output: &[u8]) -> Option<&Utf8Path> {
    let path = std::str::from_utf8(output).ok()?.trim();
    (!path.is_empty()).then(|| Utf8Path::new(path))
}

fn ninja_input(path: Utf8PathBuf) -> BuildInput {
    inputs![path.as_str().replace(':', "$:")]
}

#[cfg(test)]
mod tests {
    use super::parse_git_path;

    #[test]
    fn parses_git_head_path() {
        assert_eq!(
            parse_git_path(b"../.git/HEAD\n"),
            Some(camino::Utf8Path::new("../.git/HEAD"))
        );
    }

    #[test]
    fn rejects_empty_git_head_path() {
        assert_eq!(parse_git_path(b" \n"), None);
    }

    #[test]
    fn rejects_non_utf8_git_head_path() {
        assert_eq!(parse_git_path(&[0xff]), None);
    }
}
