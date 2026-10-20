# wutong-networkd · 梧桐网络后台服务

[![CI](https://github.com/project-wutong/wutong-networkd/actions/workflows/ci.yml/badge.svg)](https://github.com/project-wutong/wutong-networkd/actions/workflows/ci.yml)
Rust 2024 · `AGPL-3.0-only`

## 中文

Wutong Network 的**用户可选** Rust 后台（NET-001）：身份与密钥、同步、联邦解析/订阅、受保护调用与签名回执。
**默认关闭**，必须在配置中显式 `enabled = true`；各项网络能力（sync/federation/agents）也默认关闭。
Qt GUI [`wutong-network`](https://github.com/project-wutong/wutong-network) 通过带版本协商的本地 IPC（Unix 套接字 `0600` / Windows 命名管道）与其通信；
后台关闭时，Writer/Knowledge/Publisher 与 Network GUI 仍可完整浏览本地内容。

## English

Optional, user-controlled background daemon for Wutong Network. Opt-in only; versioned JSON-lines
IPC (`hello`, `status`, `validate` today). Sync, federation and agent execution land behind
explicit feature switches.
