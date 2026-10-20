// SPDX-License-Identifier: AGPL-3.0-only
//! Versioned JSON-lines local IPC. Requests: `{"id":1,"method":"hello"|"status"|"validate","params":{...}}`.

use crate::{Config, IPC_VERSION};
use serde_json::{Value, json};

/// Handle one request line and produce one response value (transport independent, unit-testable).
pub fn handle(line: &str, cfg: &Config) -> Value {
    let req: Value = match wutong_core::wire::strict::parse(line.as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            return json!({"id": null, "error": {"code": e.code.as_str(), "message": e.detail}});
        }
    };
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let result = match req.get("method").and_then(Value::as_str) {
        Some("hello") => Ok(json!({
            "ipc_version": IPC_VERSION,
            "implementation": wutong_core::IMPLEMENTATION,
            "daemon": concat!("wutong-networkd/", env!("CARGO_PKG_VERSION")),
        })),
        Some("status") => Ok(json!({
            "enabled": cfg.enabled,
            "features": {"sync": cfg.features.sync, "federation": cfg.features.federation, "agents": cfg.features.agents},
        })),
        Some("validate") => {
            let env = req
                .pointer("/params/envelope")
                .cloned()
                .unwrap_or(Value::Null);
            let bytes = serde_json::to_vec(&env).unwrap_or_default();
            wutong_core::parse_and_validate(&bytes)
                .map(|e| json!({"kind": e.kind().as_str()}))
                .map_err(|e| (e.code.as_str(), e.detail))
        }
        _ => Err(("ERR_UNSUPPORTED", "unknown method".to_owned())),
    };
    match result {
        Ok(r) => json!({"id": id, "result": r}),
        Err((code, msg)) => json!({"id": id, "error": {"code": code, "message": msg}}),
    }
}

/// Serve on a Unix domain socket (owner-only permissions). Windows named pipes: tracked in issues.
#[cfg(unix)]
pub fn serve(cfg: &Config) -> anyhow::Result<()> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    cfg.require_enabled()?;
    let _ = std::fs::remove_file(&cfg.socket);
    let listener = UnixListener::bind(&cfg.socket)?;
    std::fs::set_permissions(&cfg.socket, std::fs::Permissions::from_mode(0o600))?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let reader = BufReader::new(stream.try_clone()?);
        for line in reader.lines() {
            let resp = handle(&line?, cfg);
            writeln!(stream, "{resp}")?;
        }
    }
    Ok(())
}

/// Non-Unix platforms: named-pipe transport not implemented yet.
#[cfg(not(unix))]
pub fn serve(cfg: &Config) -> anyhow::Result<()> {
    cfg.require_enabled()?;
    anyhow::bail!("named-pipe IPC is not implemented on this platform yet")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config::from_toml("enabled = true\nsocket = \"/tmp/x\"").unwrap()
    }

    #[test]
    fn hello_status_validate() {
        let r = handle(r#"{"id":1,"method":"hello"}"#, &cfg());
        assert_eq!(r["result"]["ipc_version"], 1);
        let r = handle(r#"{"id":2,"method":"status"}"#, &cfg());
        assert_eq!(r["result"]["features"]["sync"], false);
        let env = r#"{"protocol":"wutong/0.1","kind":"wutong.principal","id":"urn:uuid:019a1111-1111-7111-8111-000000000064","occurred_at":"2026-10-18T10:00:00Z","payload":{"principal_id":"urn:uuid:019a1111-1111-7111-8111-000000000065","principal_type":"human"}}"#;
        let r = handle(
            &format!(r#"{{"id":3,"method":"validate","params":{{"envelope":{env}}}}}"#),
            &cfg(),
        );
        assert_eq!(r["result"]["kind"], "wutong.principal");
        let r = handle(r#"{"id":4,"method":"nope"}"#, &cfg());
        assert_eq!(r["error"]["code"], "ERR_UNSUPPORTED");
        let r = handle(r#"{"id":5,"id":6}"#, &cfg());
        assert_eq!(r["error"]["code"], "ERR_SCHEMA");
    }

    #[cfg(unix)]
    #[test]
    fn socket_roundtrip() {
        use std::io::{BufRead, BufReader, Write};
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("n.sock");
        let c = Config::from_toml(&format!(
            "enabled = true\nsocket = {:?}",
            sock.to_str().unwrap()
        ))
        .unwrap();
        std::thread::spawn(move || serve(&c));
        for _ in 0..100 {
            if sock.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let mut s = std::os::unix::net::UnixStream::connect(&sock).unwrap();
        writeln!(s, r#"{{"id":1,"method":"hello"}}"#).unwrap();
        let mut line = String::new();
        BufReader::new(s).read_line(&mut line).unwrap();
        assert!(line.contains("ipc_version"));
    }
}
