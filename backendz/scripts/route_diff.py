#!/usr/bin/env python3
"""Accurate route diff: parse backendz routes.go AddRoutes blocks (WithPrefix-aware)
plus colon_routes.go; compare against backend kratos *_http.pb.go."""
import re, os, glob

ROOT = "/Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin"

def norm(p):
    p = re.sub(r'\{[^/]+\}', ':p', p)
    p = re.sub(r':[A-Za-z0-9_]+', ':p', p)
    if len(p) > 1:
        p = p.rstrip('/')
    return p

def key(m, p):
    return f"{m.upper()} {norm(p)}"

# ---------- backend (kratos) ----------
backend = {}
for f in glob.glob(f'{ROOT}/backend/api/gen/go/admin/service/v1/*_http.pb.go') + \
         glob.glob(f'{ROOT}/backend/app/admin/service/internal/server/rest_server.go'):
    src = open(f, encoding='utf8').read()
    for m, p in re.findall(r'(?:r|mux|router)\.(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\("([^"]+)"', src):
        backend.setdefault(key(m, p), os.path.basename(f))

# ---------- backendz: routes.go with WithPrefix blocks + colon_routes.go ----------
bz = {}
for f in [f'{ROOT}/backendz/internal/handler/routes.go', f'{ROOT}/backendz/internal/handler/colon_routes.go']:
    src = open(f, encoding='utf8').read()
    # split at AddRoutes( boundaries, track the WithPrefix in each block
    blocks = re.split(r'server\.AddRoutes\(', src)
    for blk in blocks[1:]:
        mpre = re.search(r'rest\.WithPrefix\("([^"]*)"\)', blk)
        prefix = mpre.group(1) if mpre else ''
        for hm, hp in re.findall(r'Method:\s*http\.Method(Get|Post|Put|Delete|Patch|Head|Options),\s*\n?\s*Path:\s*"([^"]+)"', blk):
            full = prefix + hp
            full = re.sub(r'//+', '/', full) if prefix else full
            bz.setdefault(key(hm, full), f"{os.path.basename(f)}:prefix={prefix or '-'}")
# also .api colon alternates already covered by slash routes; include desc/*.api for completeness (prefix-aware)
for f in glob.glob(f'{ROOT}/backendz/desc/*.api'):
    src = open(f, encoding='utf8').read()
    parts = re.split(r'(@server\s*\([^)]*\))', src)
    prefix = None
    for seg in parts:
        if seg.startswith('@server'):
            mp = re.search(r'prefix:\s*"([^"]+)"', seg)
            prefix = mp.group(1) if mp else ''
            continue
        for hm, hp in re.findall(r'^\s*(get|post|put|delete|patch)\s+(/[^\s(]*)', seg, re.M):
            full = (prefix or '') + hp
            k = key(hm, full)
            if k not in bz:
                bz[k] = f"{os.path.basename(f)}:.api"

only_backend = sorted(k for k in backend if k not in bz)
only_bz = sorted(k for k in bz if k not in backend)
common = len(backend) - len(only_backend)

print(f"backend: {len(backend)} | backendz: {len(bz)} | matched: {common}")
print(f"\n=== MISSING in backendz ({len(only_backend)}) ===")
for r in only_backend:
    print(f"  {r}   [{backend[r]}]")
print(f"\n=== EXTRA in backendz ({len(only_bz)}) ===")
for r in only_bz:
    print(f"  {r}   [{bz[r]}]")
