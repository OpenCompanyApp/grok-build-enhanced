//! `/release-notes` -- view official upstream release notes for the base version.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

pub const OFFICIAL_RELEASE_NOTES_TITLE: &str = "Official xAI / Upstream Release Notes";
const OFFICIAL_RELEASE_NOTES_DESCRIPTION: &str =
    "View official xAI/upstream notes for the base version";
const OFFICIAL_RELEASE_NOTES_OFFLINE: &str =
    "No official xAI/upstream release notes available (offline).";

/// Show xAI's official upstream notes for the compiled base version.
pub struct ReleaseNotesCommand;

impl SlashCommand for ReleaseNotesCommand {
    fn name(&self) -> &str {
        "release-notes"
    }

    fn aliases(&self) -> &[&str] {
        &["changelog"]
    }

    fn description(&self) -> &str {
        OFFICIAL_RELEASE_NOTES_DESCRIPTION
    }

    fn usage(&self) -> &str {
        "/release-notes"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        let changelog = xai_grok_shell::util::changelog::ChangelogManager::new().fetch();
        match changelog.markdown {
            Some(content) => CommandResult::Action(Action::ShowReleaseNotes {
                title: OFFICIAL_RELEASE_NOTES_TITLE.to_string(),
                content: content.trim().to_string(),
            }),
            None => CommandResult::Error(OFFICIAL_RELEASE_NOTES_OFFLINE.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_notes_metadata() {
        let cmd = ReleaseNotesCommand;
        assert_eq!(cmd.name(), "release-notes");
        assert_eq!(cmd.aliases(), &["changelog"]);
        assert!(!cmd.takes_args());
    }

    #[test]
    fn release_notes_returns_action_or_error() {
        let models = crate::acp::model_state::ModelState::default();
        let mut ctx = super::super::tests::make_ctx(&models);
        let result = ReleaseNotesCommand.run(&mut ctx, "");
        assert!(
            matches!(result, CommandResult::Action(_) | CommandResult::Error(_)),
            "expected Action or Error, got {result:?}"
        );
    }

    #[test]
    fn release_notes_surfaces_identify_official_upstream_ownership() {
        let command = ReleaseNotesCommand;
        assert!(command.description().contains("official xAI/upstream"));
        assert_eq!(
            OFFICIAL_RELEASE_NOTES_TITLE,
            "Official xAI / Upstream Release Notes"
        );
        assert!(OFFICIAL_RELEASE_NOTES_OFFLINE.contains("official xAI/upstream"));
        assert!(!OFFICIAL_RELEASE_NOTES_TITLE.contains("Enhanced"));
    }
}
