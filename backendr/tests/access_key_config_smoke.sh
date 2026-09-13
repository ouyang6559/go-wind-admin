#!/bin/bash
# access-keys + configs 模块 API 测试：CRUD 全链 + 令牌交换正/负例 + 内置参数禁删
R=${E2E_BASE:-http://127.0.0.1:7666}/admin/v1
REDIS_CONTAINER=${E2E_REDIS_CONTAINER:-backendr-redis-1}
REDIS_PASSWORD=${REDIS_PASSWORD:-}
KEYHEX=66353164363661373364386130393237
pass=0; fail=0
chk() { if [ "$2" = "$3" ]; then pass=$((pass+1)); echo "PASS $1 [$3] ${4:0:120}"; else fail=$((fail+1)); echo "FAIL $1 expected=$2 got=$3 ${4:0:200}"; fi }
captcha_ans() { docker exec "$REDIS_CONTAINER" redis-cli ${REDIS_PASSWORD:+-a "$REDIS_PASSWORD"} --no-auth-warning GET "gowind:captcha:$1" | tr -d '\r\n'; }
login() {
  local CAP CID ANS ENC
  CAP=$(curl -s $R/captcha); CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
  ANS=$(captcha_ans "$CID")
  ENC=$(printf 'Abcd@1234' | openssl enc -aes-128-cbc -K $KEYHEX -iv $KEYHEX -base64 -A)
  curl -s -X POST $R/login -H 'Content-Type: application/json' -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" \
    -d "{\"username\":\"admin\",\"password\":\"$ENC\",\"grant_type\":\"password\",\"client_type\":\"admin\"}" \
    | python3 -c "import sys,json;print(json.load(sys.stdin).get('access_token',''))"
}
TOKEN=$(login)
[ -n "$TOKEN" ] && chk login 0 0 "token len=${#TOKEN}" || { echo "FATAL: login failed"; exit 1; }
A="Authorization: Bearer $TOKEN"
rt() { curl -s -w "\n%{http_code}" -H "$A" "$@"; }

# 重跑幂等：清理上次残留的测试数据（软删也占 key 唯一约束，须硬删）
docker exec "$REDIS_CONTAINER" true 2>/dev/null
PG_CONTAINER=${PG_CONTAINER:-backendr-postgres-1}
docker exec -e PGPASSWORD='*Abcd123456' "$PG_CONTAINER" psql -U postgres -d gwa -c \
  "delete from sys_configs where key in ('ci.flag','ci.builtin'); delete from sys_access_keys where name like 'ci-key%'" >/dev/null 2>&1 || true

echo "== access-keys CRUD =="
RSP=$(rt -X POST $R/access-keys -H 'Content-Type: application/json' -d '{"data":{"name":"ci-key","status":"ON"}}'); CODE=$(echo "$RSP"|tail -1); BODY=$(echo "$RSP"|sed '$d')
chk ak-create 200 "$CODE" "$BODY"
AK=$(echo "$BODY" | python3 -c "import sys,json;print(json.load(sys.stdin)['data']['accessKey'])")
SK=$(echo "$BODY" | python3 -c "import sys,json;print(json.load(sys.stdin)['secret'])")
AKID=$(echo "$BODY" | python3 -c "import sys,json;print(json.load(sys.stdin)['data']['id'])")
chk ak-create-shape 0 0 "ak=$AK sk_len=${#SK} id=$AKID"
# SK 形状：sk- 前缀 + 64 hex（32 字节）
[[ "$SK" == sk-* && ${#SK} -eq 67 ]] && chk ak-sk-format 0 0 "sk- 64hex ok" || chk ak-sk-format 0 1 "sk=${SK:0:10} len=${#SK}"

RSP=$(rt "$R/access-keys?page=1&pageSize=10"); chk ak-list 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 150)"
echo "$RSP" | grep -q "\"accessKey\":\"$AK\"" && chk ak-list-contains 0 0 "found" || chk ak-list-contains 0 1 "not found"
RSP=$(rt "$R/access-keys/$AKID"); chk ak-get 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 150)"
RSP=$(rt "$R/access-keys/999999"); chk ak-get-404 404 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 80)"
RSP=$(rt -X PUT $R/access-keys/$AKID -H 'Content-Type: application/json' -d '{"data":{"name":"ci-key-renamed","status":"OFF"}}'); chk ak-update 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 80)"
RSP=$(rt "$R/access-keys/$AKID"); echo "$RSP" | grep -q '"OFF"' && chk ak-update-effect 0 0 "status OFF" || chk ak-update-effect 0 1 "$(echo "$RSP"|head -c 150)"

echo "== 令牌交换（IssueToken，免鉴权）=="
# SK 正确但 AK 停用 → 400 disabled（不进 token 判定）
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d "{\"accessKey\":\"$AK\",\"secret\":\"$SK\"}")
chk token-disabled-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
# 重新启用
rt -X PUT $R/access-keys/$AKID -H 'Content-Type: application/json' -d '{"data":{"status":"ON"}}' > /dev/null
# 正确 AK/SK → 200 + accessToken
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d "{\"accessKey\":\"$AK\",\"secret\":\"$SK\"}")
CODE=$(echo "$RSP"|tail -1); BODY=$(echo "$RSP"|sed '$d')
chk token-ok 200 "$CODE" "$(echo "$BODY"|head -c 120)"
MTOKEN=$(echo "$BODY" | python3 -c "import sys,json;print(json.load(sys.stdin).get('accessToken',''))" 2>/dev/null)
[ -n "$MTOKEN" ] && chk token-shape 0 0 "mtoken len=${#MTOKEN}" || chk token-shape 0 1 "no token: $BODY"
# 机器令牌可用性：带机器令牌调只读端点（uid=0 平台视角）
RSP=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $MTOKEN" "$R/access-keys/$AKID")
chk machine-token-usable 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 100)"
# 错误 SK → 400 invalid（防枚举同文案）
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d "{\"accessKey\":\"$AK\",\"secret\":\"sk-wrong\"}")
chk token-bad-secret 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
# 不存在的 AK → 400 同文案（不泄露存在性）
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d '{"accessKey":"ak-notexist","secret":"sk-x"}')
BAD1=$(echo "$RSP"|sed '$d')
chk token-bad-ak 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
# 空参数 → 400
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d '{}')
chk token-empty-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"

echo "== ResetSecret 轮换 =="
RSP=$(rt -X PUT $R/access-keys/$AKID/secret -H 'Content-Type: application/json' -d '{}'); CODE=$(echo "$RSP"|tail -1); BODY=$(echo "$RSP"|sed '$d')
chk ak-reset-secret 200 "$CODE" "$(echo "$BODY"|head -c 120)"
SK2=$(echo "$BODY" | python3 -c "import sys,json;print(json.load(sys.stdin)['secret'])")
# 旧 SK 立即失效
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d "{\"accessKey\":\"$AK\",\"secret\":\"$SK\"}")
chk token-old-sk-rejected 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 80)"
# 新 SK 可用
RSP=$(curl -s -w "\n%{http_code}" -X POST $R/access-keys/token -H 'Content-Type: application/json' -d "{\"accessKey\":\"$AK\",\"secret\":\"$SK2\"}")
chk token-new-sk-ok 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|sed '$d'|head -c 80)"

echo "== configs CRUD =="
RSP=$(rt -X POST $R/configs -H 'Content-Type: application/json' -d '{"data":{"name":"CI配置","key":"ci.flag","value":"true","valueType":"BOOL"}}'); chk cfg-create 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 80)"
RSP=$(rt "$R/configs?page=1&pageSize=10"); chk cfg-list 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 150)"
echo "$RSP" | grep -q '"key":"ci.flag"' && chk cfg-list-contains 0 0 "found" || chk cfg-list-contains 0 1 "missing"
CFGID=$(echo "$RSP"|sed '$d'|python3 -c "import sys,json;items=json.load(sys.stdin)['items'];print(next(i['id'] for i in items if i.get('key')=='ci.flag'))")
RSP=$(rt "$R/configs/$CFGID"); chk cfg-get 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 120)"
RSP=$(rt -X PUT $R/configs/$CFGID -H 'Content-Type: application/json' -d '{"data":{"value":"false"}}'); chk cfg-update 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 80)"
RSP=$(rt "$R/configs/$CFGID"); echo "$RSP"|grep -q '"value":"false"' && chk cfg-update-effect 0 0 "value=false" || chk cfg-update-effect 0 1 "$(echo "$RSP"|head -c 120)"
# key 唯一冲突
RSP=$(rt -X POST $R/configs -H 'Content-Type: application/json' -d '{"data":{"key":"ci.flag","value":"dup"}}'); chk cfg-dup-key-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
# 非法 valueType
RSP=$(rt -X POST $R/configs -H 'Content-Type: application/json' -d '{"data":{"key":"ci.bad","valueType":"FLOAT"}}'); chk cfg-bad-type-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
# 内置参数禁删：建一行 is_built_in=true
RSP=$(rt -X POST $R/configs -H 'Content-Type: application/json' -d '{"data":{"key":"ci.builtin","value":"1","valueType":"INT","isBuiltIn":true}}'); chk cfg-create-builtin 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 60)"
BID=$(rt "$R/configs?page=1&pageSize=20"|sed '$d'|python3 -c "import sys,json;items=json.load(sys.stdin)['items'];print(next(i['id'] for i in items if i.get('key')=='ci.builtin'))")
RSP=$(rt -X DELETE $R/configs/$BID); chk cfg-builtin-delete-400 400 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
# 普通行可删
RSP=$(rt -X DELETE $R/configs/$CFGID); chk cfg-delete 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 60)"
# 幂等删除
RSP=$(rt -X DELETE $R/configs/$CFGID); chk cfg-delete-idempotent 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 60)"

echo "== access-keys 删除收尾 =="
RSP=$(rt -X DELETE $R/access-keys/$AKID); chk ak-delete 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 60)"

echo; echo "==== access_key/config smoke: pass=$pass fail=$fail ===="
exit $fail
