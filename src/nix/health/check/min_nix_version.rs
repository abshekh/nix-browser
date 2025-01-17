use leptos::*;
use serde::{Deserialize, Serialize};

use crate::nix::{
    health::{
        report::{Report, WithDetails},
        traits::Check,
    },
    info,
    version::NixVersion,
};

/// Check that [crate::nix::version::NixVersion] is set to a good value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinNixVersion(NixVersion);

impl Check for MinNixVersion {
    fn check(info: &info::NixInfo) -> Self {
        MinNixVersion(info.nix_version.clone())
    }
    fn name(&self) -> &'static str {
        "Minimum Nix Version"
    }
    fn report(&self) -> Report<WithDetails> {
        let min_required = NixVersion {
            major: 2,
            minor: 13,
            patch: 0,
        };
        if self.0 >= min_required {
            Report::Green
        } else {
            Report::Red(WithDetails {
                msg: format!("Your Nix version ({}) is too old; we require at least {}", self.0, min_required),
                suggestion: "See https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-upgrade-nix.html".into(),
            })
        }
    }
}

impl IntoView for MinNixVersion {
    fn into_view(self, cx: Scope) -> View {
        view! { cx, <span>"Nix version: " {self.0}</span> }.into_view(cx)
    }
}
