// SPDX-License-Identifier: AGPL-3.0-only
//! `wutong-networkd`: the optional Rust background service of Wutong Network (NET-001).
//!
//! Principles from the architecture baseline:
//! * strictly **opt-in** — refuses to start unless explicitly enabled; no silent auto-enable of
//!   sync, federation or agents;
//! * the Qt GUI (`wutong-network`) can start/stop/connect but never depends on it for local browsing;
//! * local IPC is a versioned JSON-lines protocol over a Unix domain socket / named pipe with
//!   OS ACLs (`0600` socket, owner-only directory).

pub mod config;
pub mod ipc;

pub use config::Config;

/// IPC protocol version, negotiated in `hello`.
pub const IPC_VERSION: u32 = 1;
