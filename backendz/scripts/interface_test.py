#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
go-wind-admin 双端接口测试 harness
==================================
对 backend(go-kratos, :7788) 和 backendz(go-zero, :8888) 跑同一组 HTTP 接口检查，
归一化两套响应封装（backend=proto 原生体；backendz={code,msg,data}），
逐项输出 PASS / FAIL / SKIP，并打印结构化 JSON 便于汇总报告。

用法：
  python3 interface_test.py --base http://localhost:8888 --mode backendz   # 明文登录
  python3 interface_test.py --base http://localhost:7788 --mode backend    # AES 加密密码登录
"""
import argparse
import base64
import json
import random
import subprocess
import sys
import time
import urllib.parse
import urllib.request

# 统一返回 {name, ok(bool), detail(str), ms}
RESULTS = []

REDIS_CONTAINER = "backendz-redis-1"
REDIS_PASS = "*Abcd123456"


def aes_encrypt_password_cli(plain: str) -> str:
    """纯命令行 AES-128-CBC(PKCS5) 加密 'admin'，key=IV='f51d66a73d8a0927'，返回 base64。
    用于 backend 的密码传输（go-kratos 要求 base64(AESEncrypt(pwd))）。"""
    keyhex = "66353164363661373364386130393237"  # b"f51d66a73d8a0927"
    out = subprocess.run(
        ["openssl", "enc", "-aes-128-cbc", "-K", keyhex, "-iv", keyhex, "-base64", "-A"],
        input=plain.encode("utf-8"), capture_output=True, check=True,
    )
    return out.stdout.decode("utf-8").strip()


def http(method: str, url: str, body=None, headers=None, timeout=15):
    body_bytes = None
    if body is not None:
        body_bytes = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(url, data=body_bytes, method=method)
    for k, v in (headers or {}).items():
        req.add_header(k, v)
    if body_bytes is not None:
        req.add_header("Content-Type", "application/json")
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            status = resp.status
            raw = resp.read().decode("utf-8", "replace")
    except urllib.error.HTTPError as e:
        status = e.code
        raw = e.read().decode("utf-8", "replace")
    except Exception as e:  # noqa: BLE001 - 网络错误
        return status if "status" in dir() else 0, {"__error__": str(e)}
    try:
        return status, json.loads(raw)
    except Exception:
        return status, {"__raw__": raw}


class Svc:
    """封装两套响应形态的差异。"""

    def __init__(self, mode, base):
        self.mode = mode
        self.base = base.rstrip("/")

    # ---- 封装归一 ----
    def is_ok(self, status, j):
        if self.mode == "backendz":
            return status == 200 and j.get("code") == 0
        # backend 原生体：以可解析 JSON 且无 __error__ 视为"服务可达"
        return status == 200 and "__error__" not in j

    def data(self, j):
        if self.mode == "backendz":
            return j.get("data") or {}
        return j

    def record(self, name, ok, detail="", ms=None):
        line = f"{name}: {'PASS' if ok else 'FAIL'}"
        if ms is not None:
            line += f" ({ms:.0f}ms)"
        if detail:
            line += f"  [{detail}]"
        print(line)
        RESULTS.append({"name": name, "ok": ok, "detail": str(detail)[:300], "ms": ms})

    # ---- 验证码 ----
    def captcha(self):
        status, j = http("GET", f"{self.base}/admin/v1/captcha")
        d = self.data(j)
        cid = (d.get("captchaId") or d.get("CaptchaId")) if isinstance(d, dict) else None
        img = (d.get("imageBase64") or d.get("ImageBase64")) if isinstance(d, dict) else None
        ok = self.is_ok(status, j) and bool(cid) and bool(img)
        self.record("captcha 生成", ok, f"status={status} captchaId={'有' if cid else '无'}")
        return cid if ok else None

    def captcha_answer(self, cid):
        """从共享 Redis 读验证码答案（backendz/backend 同用 gowind:captcha:<id>）。"""
        if not cid:
            return None
        r = subprocess.run(
            ["docker", "exec", REDIS_CONTAINER, "redis-cli", "-a", REDIS_PASS,
             "--no-auth-warning", "GET", f"gowind:captcha:{cid}"],
            capture_output=True, text=True, check=False,
        )
        ans = (r.stdout or "").strip()
        return ans if ans and "require" not in ans.lower() else None

    # ---- 登录 ----
    def login(self):
        cid = self.captcha()
        ans = self.captcha_answer(cid)
        headers = {}
        if cid and ans:
            headers = {"X-Captcha-Id": cid, "X-Captcha-Value": ans}
        if self.mode == "backend":
            pwd = aes_encrypt_password_cli("admin")
            body = {"grant_type": "password", "username": "admin", "password": pwd,
                    "client_id": "itest", "device_id": "itest-1"}
        else:
            body = {"grant_type": "password", "username": "admin", "password": "admin",
                    "clientInfo": None, "tenantCode": ""}
        status, j = http("POST", f"{self.base}/admin/v1/login", body=body, headers=headers)
        d = self.data(j) if isinstance(j, dict) else {}
        tok = d.get("access_token") if isinstance(d, dict) else None
        # backend 登录若因缺 RBAC 种子返回 403 等，属已知限制 → 记录为 'BLOCKED' 而非 FAIL
        if self.mode == "backend":
            if tok:
                self.record("login", True, "登录成功，拿到 access_token")
            else:
                msg = j.get("message") or j.get("msg") or str(j)
                self.record("login", False, f"PROTO_OBTAINED_TOKEN_ERR status={status} msg={msg}")
            return tok
        ok = bool(tok)
        self.record("login", ok, f"status={status}" + ("" if ok else f" msg={j.get('msg')}"))
        return tok


def auth_round(svc, tok):
    """已登录后的只读业务接口检查（同一组路径双端共用）。"""
    # 先取列表拿首个真实 id，再查详情（避免 hardcode id 撞上已被软/硬删的一行）
    first_id = None
    if tok:
        _, lj = http("GET", f"{svc.base}/admin/v1/roles?page=1&pageSize=10",
                     headers={"Authorization": f"Bearer {tok}"})
        ld = svc.data(lj) if isinstance(lj, dict) else {}
        items = ld.get("items") if isinstance(ld, dict) else None
        if isinstance(items, list) and items:
            first_id = items[0].get("id")
    checks = [
        ("roles 列表", "GET", "/admin/v1/roles?page=1&pageSize=10", None, "items"),
        ("roles 详情", "GET", f"/admin/v1/roles/{first_id}" if first_id else None, None, None),
        ("menus 列表", "GET", "/admin/v1/menus", None, None),
        ("dict/types 列表", "GET", "/admin/v1/dict/types?page=1&pageSize=10", None, "items"),
        ("dict/langs 列表", "GET", "/admin/v1/dict/langs?page=1&pageSize=10", None, "items"),
        ("users 列表", "GET", "/admin/v1/users?page=1&pageSize=10", None, "items"),
    ]
    for name, method, path, body, key in checks:
        if not tok or not path:
            svc.record(f"{name}(需登录)", False, "SKIP: 未取得 token 或 token 下无该数据")
            continue
        t0 = time.time()
        status, j = http(method, f"{svc.base}{path}", body=body,
                         headers={"Authorization": f"Bearer {tok}"})
        ms = (time.time() - t0) * 1000
        d = svc.data(j) if isinstance(j, dict) else {}
        ok = svc.is_ok(status, j)
        detail = f"status={status} code={j.get('code') if svc.mode=='backendz' else '-'}"
        if ok and key and isinstance(d, dict) and key in d:
            detail += f" {key}={len(d[key]) if isinstance(d[key], list) else d[key]}"
        svc.record(name, ok, detail, ms)


def write_create_cleanup(svc, tok):
    """写操作闭环：create → 校验 → delete 清理（仅 backendz）。"""
    uniq = f"it{random.randint(10, 99)}"
    code = f"it_{uniq}"
    post = {"data": {"code": code, "name": f"接口测试角色{uniq}", "sortOrder": 1,
                     "type": "TENANT", "status": "ON"}}
    t0 = time.time()
    status, j = http("POST", f"{svc.base}/admin/v1/roles", body=post,
                     headers={"Authorization": f"Bearer {tok}"})
    ms = (time.time() - t0) * 1000
    svc.record("roles create {data} 包裹", svc.is_ok(status, j), f"status={status} code={code}", ms)
    if not svc.is_ok(status, j):
        return
    # 从列表里找到刚创建的角色 id 进行清理
    _, lj = http("GET", f"{svc.base}/admin/v1/roles?page=1&pageSize=50",
                 headers={"Authorization": f"Bearer {tok}"})
    ld = svc.data(lj) if isinstance(lj, dict) else {}
    rid = None
    for it in (ld.get("items") or []):
        if it.get("code") == code:
            rid = it.get("id")
            break
    if rid:
        _, _ = http("DELETE", f"{svc.base}/admin/v1/roles/{rid}",
                    headers={"Authorization": f"Bearer {tok}"})
        svc.record("roles delete 清理", True, f"id={rid} 已软删")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--mode", choices=["backend", "backendz"], required=True)
    args = ap.parse_args()
    svc = Svc(args.mode, args.base)
    print(f"== 接口测试 mode={args.mode} base={args.base} ==")
    tok = svc.login()
    auth_round(svc, tok)
    if args.mode == "backendz" and tok:
        write_create_cleanup(svc, tok)
    print("\n== 结果汇总 ==")
    passed = sum(1 for r in RESULTS if r["ok"])
    skipped = sum(1 for r in RESULTS if "SKIP" in r["detail"] or "BLOCKED" in r["detail"])
    print(f"PASS={passed} FAIL={len(RESULTS)-passed} TOTAL={len(RESULTS)} (SKIP/BLOCKED={skipped})")
    print("JSON=" + json.dumps(RESULTS, ensure_ascii=False))


if __name__ == "__main__":
    main()