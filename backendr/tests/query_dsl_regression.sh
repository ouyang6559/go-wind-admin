#!/bin/bash
# 查询 DSL 修复回归：2026-09-14 修复项的 API 级验证。
#   1. type__not 语义（曾反转成 type = 'TEMPLATE'，vue 两端在用）
#   2. created_at__gte/__lte 时间过滤（曾 400 unknown query field + TEXT 绑参 500）
#   3. 未知操作符 fail-closed 400（曾静默降级 exact）
#   4. regex 操作符（新增）
# 用法：E2E_BASE=http://127.0.0.1:7666 [E2E_REDIS_CONTAINER=… REDIS_PASSWORD=…] bash tests/query_dsl_regression.sh
R=${E2E_BASE:-http://127.0.0.1:7666}/admin/v1
REDIS_CONTAINER=${E2E_REDIS_CONTAINER:-gwa-rust-test-redis}
KEYHEX=66353164363661373364386130393237
captcha_ans() { docker exec "$REDIS_CONTAINER" redis-cli ${REDIS_PASSWORD:+-a "$REDIS_PASSWORD"} --no-auth-warning GET "gowind:captcha:$1" | tr -d '\r\n'; }
pass=0; fail=0
chk() {
  if [ "$2" = "$3" ]; then pass=$((pass+1)); echo "PASS $1 [$3] ${4:0:140}"; else fail=$((fail+1)); echo "FAIL $1 expected=$2 got=$3 ${4:0:220}"; fi
}
jget() { echo "$1" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d$2)" 2>/dev/null; }

# 登录
CAP=$(curl -s $R/captcha); CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
ANS=$(captcha_ans "$CID")
ENC=$(printf 'Abcd@1234' | openssl enc -aes-128-cbc -K $KEYHEX -iv $KEYHEX -base64 -A)
LOGIN=$(curl -s -X POST $R/login -H 'Content-Type: application/json' -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" -d "{\"username\":\"admin\",\"password\":\"$ENC\",\"grant_type\":\"password\"}")
TOKEN=$(echo "$LOGIN" | python3 -c "import sys,json;print(json.load(sys.stdin).get('access_token',''))")
[ -n "$TOKEN" ] && { pass=$((pass+1)); echo "PASS login"; } || { fail=$((fail+1)); echo "FAIL login: $(echo "$LOGIN"|head -c 200)"; }
A="Authorization: Bearer $TOKEN"
rt() { curl -s -w "\n%{http_code}" -H "$A" "$@"; }

# ---- 1. type__not：排除 TEMPLATE（曾反转成只取 TEMPLATE）----
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"type__not":"TEMPLATE"}' --data-urlencode 'pageSize=100')
CODE=$(echo "$RSP"|tail -1); BODY=$(echo "$RSP"|sed '$d')
chk role-type-not-200 200 "$CODE" "$(echo "$BODY"|head -c 160)"
N_TPL=$(echo "$BODY" | python3 -c "import sys,json;d=json.load(sys.stdin);print(sum(1 for i in d.get('items',[]) if i.get('type')=='TEMPLATE'))" 2>/dev/null || echo -1)
chk role-type-not-excludes-template 0 "$N_TPL" "items 中 TEMPLATE 数量应为 0，实际 $N_TPL"
N_OTHER=$(echo "$BODY" | python3 -c "import sys,json;d=json.load(sys.stdin);print(len(d.get('items',[])))" 2>/dev/null || echo -1)
[ "$N_OTHER" -gt 0 ] && { pass=$((pass+1)); echo "PASS role-type-not-keeps-others ($N_OTHER rows)"; } || { fail=$((fail+1)); echo "FAIL role-type-not-keeps-others: 空列表？$N_OTHER"; }

# 对照：裸 type 等值（语义未受影响）
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"type":"SYSTEM"}' --data-urlencode 'pageSize=100')
BODY=$(echo "$RSP"|sed '$d')
N_SYS=$(echo "$BODY" | python3 -c "import sys,json;d=json.load(sys.stdin);print(all(i.get('type')=='SYSTEM' for i in d.get('items',[])) and len(d.get('items',[]))>0)" 2>/dev/null || echo False)
chk role-bare-type-eq True "$N_SYS" "裸键 EQ"

# ---- 2. created_at__gte/__lte 时间过滤（曾 400；且 cast 缺失会 500）----
NOW=$(python3 -c "from datetime import datetime,timezone;print(datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S+00:00'))")
OLD=$(python3 -c "from datetime import datetime,timezone;print((datetime.now(timezone.utc).replace(year=2000)).strftime('%Y-%m-%dT%H:%M:%S+00:00'))")
for M in login-audit-logs operation-audit-logs api-audit-logs permission-audit-logs data-access-audit-logs policy-evaluation-logs; do
  RSP=$(rt --get "$R/$M" --data-urlencode "query={\"created_at__gte\":\"$OLD\",\"created_at__lte\":\"$NOW\"}" --data-urlencode 'pageSize=5')
  CODE=$(echo "$RSP"|tail -1)
  chk "audit-$M-created-at-range" 200 "$CODE" "$(echo "$RSP"|sed '$d'|head -c 140)"
done
# 时间过滤确实生效：gte 取未来 → total=0（若过滤被忽略会 >0）
FUT=$(python3 -c "from datetime import datetime,timezone;print((datetime.now(timezone.utc).replace(year=2100)).strftime('%Y-%m-%dT%H:%M:%S+00:00'))")
RSP=$(rt --get "$R/login-audit-logs" --data-urlencode "query={\"created_at__gte\":\"$FUT\"}" --data-urlencode 'pageSize=5')
CODE=$(echo "$RSP"|tail -1); BODY=$(echo "$RSP"|sed '$d')
TOTAL=$(jget "$BODY" "['total']")
chk audit-gte-future-code 200 "$CODE"
chk audit-gte-future-total-0 0 "$TOTAL" "未来时间下界 total 应为 0，实际 $TOTAL"

# orderBy=-created_at（同白名单路径，曾一并 400）
RSP=$(rt --get "$R/login-audit-logs" --data-urlencode 'orderBy=["-created_at"]' --data-urlencode 'pageSize=2')
chk audit-orderBy-created-at 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 120)"

# ---- 3. 未知操作符 fail-closed 400（date/year 未实现，不可静默降级）----
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"name__year":"2026"}')
chk unknown-op-year-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 140)"
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"name__week_day":"1"}')
chk unknown-op-weekday-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 140)"
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"name__bogus_op":"x"}')
chk unknown-op-bogus-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 140)"

# ---- 4. regex（新增，postgres ~）----
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"name__regex":"^.*$"}' --data-urlencode 'pageSize=3')
chk regex-op-200 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 140)"
RSP=$(rt --get "$R/roles" --data-urlencode 'query={"name__iregex":"^A.*$"}' --data-urlencode 'pageSize=3')
chk iregex-op-200 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 140)"

# ---- 5. 常用子集回归（e2e 之外的关键操作符各抽一）----
for Q in '{"name__contains":"a"}' '{"name__ne":"__no_such__"}' '{"name__like":"%"}' '{"name__in":["x1","x2"]}' '{"name__nin":["__none__"]}'; do
  RSP=$(rt --get "$R/roles" --data-urlencode "query=$Q" --data-urlencode 'pageSize=3')
  chk "op-${Q:0:24}..." 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 100)"
done

# ---- 6. server-monitor in_use ≠ idle ----
RSP=$(rt $R/server-monitor); BODY=$(echo "$RSP"|sed '$d')
INUSE=$(jget "$BODY" "['database']['inUseConnections']")
IDLE=$(jget "$BODY" "['database']['idleConnections']")
OPEN=$(jget "$BODY" "['database']['openConnections']")
chk server-monitor-inuse+idle=open True "$(python3 -c "print(int('$INUSE')+int('$IDLE')==int('$OPEN'))")" "inUse=$INUSE idle=$IDLE open=$OPEN"

echo; echo "RESULT: pass=$pass fail=$fail"
[ "$fail" -eq 0 ] || exit 1
