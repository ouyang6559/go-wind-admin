#!/usr/bin/env python3
"""Go(:7788) vs Rust(:7666) wire-shape comparator.

Usage: python3 compare_wire.py <go_token> <rust_token>
Compares JSON key structure (and scalar values for identity) of paired endpoints.
"""
import json
import os
import sys
import urllib.request

GO = "http://localhost:7788"
RUST = os.environ.get("RUST_BASE", "http://localhost:7666")

PATHS = [
    "/admin/v1/users?page=1&pageSize=3",
    "/admin/v1/users/1",
    "/admin/v1/me",
    "/admin/v1/permissions?page=1&pageSize=3",
    "/admin/v1/permissions/1",
    "/admin/v1/menus?page=1&pageSize=3",
    "/admin/v1/menus/1",
    "/admin/v1/permission-groups?page=1&pageSize=3",
    "/admin/v1/permission-groups/1",
    "/admin/v1/routes",
    "/admin/v1/mfa/status",
    "/admin/v1/mfa/methods",
    "/admin/v1/roles?page=1&pageSize=3",
    "/admin/v1/roles/1",
    "/admin/v1/org-units?page=1&pageSize=3",
    "/admin/v1/positions?page=1&pageSize=3",
    "/admin/v1/dict/types?page=1&pageSize=3",
    "/admin/v1/dict/langs?page=1&pageSize=3",
    "/admin/v1/tenants?page=1&pageSize=3",
    "/admin/v1/plans?page=1&pageSize=3",
    "/admin/v1/dashboard/overview",
    "/admin/v1/dashboard/login-trend",
    "/admin/v1/initial-context",
    "/admin/v1/online-session/sessions?page=1&pageSize=3",
    "/admin/v1/online-session/my-sessions?page=1&pageSize=3",
    "/admin/v1/login-audit-logs?page=1&pageSize=3",
    "/admin/v1/operation-audit-logs?page=1&pageSize=3",
]


def fetch(base, token, path):
    req = urllib.request.Request(base + path, headers={"Authorization": "Bearer " + token})
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            return r.status, json.loads(r.read().decode())
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        try:
            return e.code, json.loads(body)
        except Exception:
            return e.code, {"__raw__": body[:200]}
    except Exception as e:  # noqa: BLE001
        return -1, {"__err__": str(e)}


IDENTITY_KEYS = ("id", "name", "code", "username", "path")


def keyed_children(v):
    """If list of dicts with an identity key, map identity->dict, else None."""
    if isinstance(v, list) and v and all(isinstance(x, dict) for x in v):
        for k in IDENTITY_KEYS:
            if all(k in x for x in v):
                return {x[k]: x for x in v}
    return None


def diff(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        ka, kb = set(a.keys()), set(b.keys())
        for k in sorted(ka - kb):
            out.append(f"{path}.{k}: GO-ONLY")
        for k in sorted(kb - ka):
            out.append(f"{path}.{k}: RUST-ONLY")
        for k in sorted(ka & kb):
            diff(a[k], b[k], f"{path}.{k}", out)
    elif isinstance(a, list) and isinstance(b, list):
        kc_a, kc_b = keyed_children(a), keyed_children(b)
        if kc_a is not None and kc_b is not None:
            for k in sorted(set(kc_a) & set(kc_b), key=str):
                diff(kc_a[k], kc_b[k], f"{path}[{k}]", out)
        else:
            if a:
                diff(a[0], b[0] if b else None, path + "[0]", out)
    else:
        if type(a) is not type(b):
            out.append(f"{path}: TYPE {type(a).__name__} vs {type(b).__name__}")


def main():
    go_tok, rust_tok = sys.argv[1], sys.argv[2]
    fails = 0
    for p in PATHS:
        gs, g = fetch(GO, go_tok, p)
        rs, r = fetch(RUST, rust_tok, p)
        if gs != rs:
            print(f"[HTTP-DIFF] {p}: go={gs} rust={rs}")
            fails += 1
            continue
        if gs >= 400:
            print(f"[both-{gs}] {p}: (error parity)")
            continue
        out = []
        diff(g, r, "$", out)
        if out:
            fails += 1
            print(f"[DIFF] {p}")
            for line in out[:40]:
                print("   ", line)
            if len(out) > 40:
                print(f"    ... {len(out)-40} more")
        else:
            print(f"[OK] {p}")
    print(f"\n== {fails} endpoint(s) with diffs ==")
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
