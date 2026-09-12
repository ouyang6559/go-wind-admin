#!/usr/bin/env python3
"""Static route diff: Go proto HTTP annotations vs Rust axum .route() registrations.

Param names are canonicalized (`{user_id}` and `{userId}` both -> `{userid}`)
so proto/Rust naming style differences don't cause false diffs.
"""
import re, os, glob

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PROTO_DIR = os.path.join(ROOT, 'backend', 'api', 'protos')
ROUTES_DIR = os.path.join(ROOT, 'backendr', 'src', 'routes')

VERBS = ('get', 'post', 'put', 'delete', 'patch')

def canon_path(p: str) -> str:
    def norm(mm):
        return '{' + re.sub(r'[^a-z0-9]', '', mm.group(1).lower()) + '}'
    p = re.sub(r'\{([^}]+)\}', norm, p)
    if not p.startswith('/admin/v1'):
        p = '/admin/v1' + p
    return p

def extract_proto_endpoints():
    eps = set()
    for f in glob.glob(os.path.join(PROTO_DIR, '**', '*.proto'), recursive=True):
        text = open(f, encoding='utf-8').read()
        for m in re.finditer(r'\b(get|post|put|delete|patch)\s*:\s*"([^"]+)"', text):
            eps.add((m.group(1).upper(), canon_path(m.group(2))))
    return eps

def extract_rust_routes():
    eps = set()
    for f in glob.glob(os.path.join(ROUTES_DIR, '*.rs')):
        text = open(f, encoding='utf-8').read()
        for m in re.finditer(r'\.route\(\s*"([^"]+)"\s*,\s*([\w:]+)\(', text):
            verb = m.group(2).split('::')[-1].lower()
            if verb not in VERBS:
                continue
            eps.add((verb.upper(), canon_path(m.group(1))))
    return eps

go_eps = extract_proto_endpoints()
rs_eps = extract_rust_routes()

missing = sorted(go_eps - rs_eps)
extra = sorted(rs_eps - go_eps)

print(f"Go proto endpoints: {len(go_eps)}")
print(f"Rust registered routes: {len(rs_eps)}")
print(f"Matched: {len(go_eps & rs_eps)}")
print(f"\n=== Missing in Rust ({len(missing)}) ===")
for v, p in missing:
    print(f"  {v:6} {p}")
print(f"\n=== Extra in Rust / aliases ({len(extra)}) ===")
for v, p in extra:
    print(f"  {v:6} {p}")
