// SPDX-License-Identifier: AGPL-3.0-only
//! Daemon configuration (`networkd.toml`).

use serde::Deserialize;
use std::path::PathBuf;

/// Feature switches; every network capability defaults to **off**.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Features {
    /// Background sync (WTP-Sync, experimental).
    #[serde(default)]
    pub sync: bool,
    /// Federation resolve/subscribe (WTP-Federation, experimental).
    #[serde(default)]
    pub federation: bool,
    /// Executing agents (WTP-Agent).
    #[serde(default)]
    pub agents: bool,
}

/// Top-level configuration.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Must be `true` for the daemon to start (explicit user consent).
    #[serde(default)]
    pub enabled: bool,
    /// IPC socket path.
    pub socket: PathBuf,
    /// Feature switches.
    #[serde(default)]
    pub features: Features,
}

impl Config {
    /// Parse TOML.
    pub fn from_toml(s: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(s)?)
    }

    /// Ensure the user opted in.
    pub fn require_enabled(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.enabled,
            "wutong-networkd is disabled; set `enabled = true` in networkd.toml to opt in"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_off_and_unknown_keys_rejected() {
        let c = Config::from_toml("socket = \"/tmp/w.sock\"").unwrap();
        assert!(!c.enabled);
        assert_eq!(c.features, Features::default());
        assert!(c.require_enabled().is_err());
        assert!(Config::from_toml("socket = \"x\"\nauto_sync = true").is_err());
    }
}
