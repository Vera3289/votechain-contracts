# Dependency Vulnerability Audit

**Date:** 2026-04-23  
**Tool:** `cargo audit`  
**Result:** ✅ Zero high/critical vulnerabilities

## Audit Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High     | 0 |
| Medium   | 0 |
| Low      | 0 |

## Dependency Versions

| Crate | Version | Notes |
|-------|---------|-------|
| soroban-sdk | 22.0.11 | Latest Protocol-22 compatible release |
| soroban-env-common | 22.1.3 | Pinned to Protocol 22 |
| soroban-env-host | 22.1.3 | Pinned to Protocol 22 |
| stellar-xdr | 0.0.9 | Pinned to Protocol 22 |

## How to Run

```bash
cargo install cargo-audit
cargo audit
```

## CI

Automated audit runs on every push and pull request via `.github/workflows/audit.yml`.
