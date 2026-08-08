//! Locally managed authentication policy for the direct ChatGPT Codex adapter.
//!
//! Only root/system and MDM requirements layers are trusted here. The user
//! requirements layer may be cloud-served, so it cannot loosen or establish a
//! local credential policy. Every independently managed allowlist narrows the
//! preceding one by intersection.

use std::collections::BTreeSet;

use toml::Value;

use super::CodexAuthError;

const ALLOWED_LOGIN_METHODS: &str = "allowed_login_methods";
const ALLOWED_CHATGPT_WORKSPACES: &str = "allowed_chatgpt_workspaces";
const CHATGPT_LOGIN_METHOD: &str = "chatgpt";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CodexManagedAuthPolicy {
    chatgpt_allowed: bool,
    allowed_workspaces: Option<Vec<String>>,
}

impl CodexManagedAuthPolicy {
    /// Resolve the process-local policy before any provider credential is read.
    pub(super) fn load() -> Result<Self, CodexAuthError> {
        let layers = xai_grok_config::requirements_layers();
        Self::from_layers(layers.iter().map(|layer| (layer.is_system, &layer.value)))
    }

    /// Unrestricted policy for hermetic tests that provide their own store.
    #[cfg(test)]
    pub(super) fn unrestricted() -> Self {
        Self {
            chatgpt_allowed: true,
            allowed_workspaces: None,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_workspaces(workspaces: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            chatgpt_allowed: true,
            allowed_workspaces: Some(workspaces.into_iter().map(str::to_owned).collect()),
        }
    }

    #[cfg(test)]
    pub(super) fn for_test_denied() -> Self {
        Self {
            chatgpt_allowed: false,
            allowed_workspaces: None,
        }
    }

    fn from_layers<'a>(
        layers: impl IntoIterator<Item = (bool, &'a Value)>,
    ) -> Result<Self, CodexAuthError> {
        let mut login_methods: Option<BTreeSet<String>> = None;
        let mut workspaces: Option<BTreeSet<String>> = None;

        for (is_system, value) in layers {
            if !is_system {
                continue;
            }
            if let Some(methods) = string_allowlist(value, ALLOWED_LOGIN_METHODS)? {
                intersect(&mut login_methods, methods);
            }
            if let Some(layer_workspaces) = string_allowlist(value, ALLOWED_CHATGPT_WORKSPACES)? {
                intersect(&mut workspaces, layer_workspaces);
            }
        }

        let chatgpt_allowed = login_methods
            .as_ref()
            .is_none_or(|methods| methods.contains(CHATGPT_LOGIN_METHOD));
        let allowed_workspaces = workspaces.map(|items| items.into_iter().collect::<Vec<_>>());
        let policy = Self {
            chatgpt_allowed,
            allowed_workspaces,
        };
        policy.ensure_usable()?;
        Ok(policy)
    }

    /// The Enhanced direct adapter supports ChatGPT subscription login only.
    /// An API-only/empty method intersection, or an empty workspace
    /// intersection, therefore leaves no usable login method and fails closed.
    pub(super) fn ensure_usable(&self) -> Result<(), CodexAuthError> {
        if !self.chatgpt_allowed || self.allowed_workspaces.as_ref().is_some_and(Vec::is_empty) {
            return Err(CodexAuthError::ManagedPolicyDenied);
        }
        Ok(())
    }

    pub(super) fn allowed_workspaces(&self) -> Option<&[String]> {
        self.allowed_workspaces.as_deref()
    }

    pub(super) fn allows_workspace(&self, account_id: &str) -> bool {
        self.allowed_workspaces
            .as_ref()
            .is_none_or(|allowed| allowed.iter().any(|id| id == account_id))
    }
}

fn intersect(target: &mut Option<BTreeSet<String>>, layer: BTreeSet<String>) {
    match target {
        Some(current) => current.retain(|item| layer.contains(item)),
        None => *target = Some(layer),
    }
}

/// `None` means the field is absent (unrestricted by this layer). A present but
/// malformed field is a managed-policy failure rather than a permissive skip.
fn string_allowlist(
    value: &Value,
    field: &'static str,
) -> Result<Option<BTreeSet<String>>, CodexAuthError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let array = raw.as_array().ok_or(CodexAuthError::ManagedPolicyDenied)?;
    let mut values = BTreeSet::new();
    for item in array {
        let item = item
            .as_str()
            .ok_or(CodexAuthError::ManagedPolicyDenied)?
            .trim();
        if !item.is_empty() {
            values.insert(item.to_owned());
        }
    }
    Ok(Some(values))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirements(source: &str) -> Value {
        toml::from_str(source).expect("valid test requirements")
    }

    #[test]
    fn only_system_layers_can_establish_auth_policy() {
        let user = requirements("allowed_login_methods = []\nallowed_chatgpt_workspaces = []\n");
        let policy = CodexManagedAuthPolicy::from_layers([(false, &user)]).unwrap();
        assert_eq!(policy, CodexManagedAuthPolicy::unrestricted());
    }

    #[test]
    fn system_login_allowlists_intersect_and_api_only_fails_closed() {
        let broad = requirements("allowed_login_methods = [\"chatgpt\", \"api\"]\n");
        let api_only = requirements("allowed_login_methods = [\"api\"]\n");
        assert!(matches!(
            CodexManagedAuthPolicy::from_layers([(true, &broad), (true, &api_only)]),
            Err(CodexAuthError::ManagedPolicyDenied)
        ));
    }

    #[test]
    fn workspace_allowlists_intersect() {
        let first = requirements(
            "allowed_login_methods = [\"chatgpt\"]\nallowed_chatgpt_workspaces = [\"alpha\", \"shared\"]\n",
        );
        let second = requirements("allowed_chatgpt_workspaces = [\"shared\", \"omega\"]\n");
        let policy = CodexManagedAuthPolicy::from_layers([(true, &first), (true, &second)])
            .expect("shared workspace remains usable");
        assert_eq!(
            policy.allowed_workspaces(),
            Some(["shared".to_owned()].as_slice())
        );
        assert!(policy.allows_workspace("shared"));
        assert!(!policy.allows_workspace("alpha"));
    }

    #[test]
    fn empty_workspace_intersection_fails_closed() {
        let first = requirements("allowed_chatgpt_workspaces = [\"alpha\"]\n");
        let second = requirements("allowed_chatgpt_workspaces = [\"omega\"]\n");
        assert!(matches!(
            CodexManagedAuthPolicy::from_layers([(true, &first), (true, &second)]),
            Err(CodexAuthError::ManagedPolicyDenied)
        ));
    }

    #[test]
    fn untrusted_cloud_or_user_layer_cannot_loosen_system_policy() {
        let system = requirements("allowed_chatgpt_workspaces = [\"managed\"]\n");
        let untrusted = requirements(
            "allowed_login_methods = [\"chatgpt\", \"api\"]\nallowed_chatgpt_workspaces = [\"managed\", \"personal\"]\n",
        );
        let policy =
            CodexManagedAuthPolicy::from_layers([(true, &system), (false, &untrusted)]).unwrap();
        assert!(policy.allows_workspace("managed"));
        assert!(!policy.allows_workspace("personal"));
    }

    #[test]
    fn malformed_managed_allowlists_do_not_fail_open() {
        for source in [
            "allowed_login_methods = \"chatgpt\"\n",
            "allowed_chatgpt_workspaces = [\"managed\", 7]\n",
        ] {
            let layer = requirements(source);
            assert!(matches!(
                CodexManagedAuthPolicy::from_layers([(true, &layer)]),
                Err(CodexAuthError::ManagedPolicyDenied)
            ));
        }
    }
}
