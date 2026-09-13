#!/bin/bash
# SSE 网关冒烟：鉴权负例 + 实时推送链路（发送站内信 → /events 收到 notification 帧）
R=${E2E_BASE:-http://127.0.0.1:7667}/admin/v1
SSE=${SSE_BASE:-http://127.0.0.1:7787}/events
REDIS_CONTAINER=${E2E_REDIS_CONTAINER:-gwa-rust-test-redis}
KEYHEX=66353164363661373364386130393237
pass=0; fail=0
chk() { if [ "$2" = "$3" ]; then pass=$((pass+1)); echo "PASS $1 [$3] ${4:0:140}"; else fail=$((fail+1)); echo "FAIL $1 expected=$2 got=$3 ${4:0:220}"; fi }
captcha_ans() { docker exec "$REDIS_CONTAINER" redis-cli ${REDIS_PASSWORD:+-a "$REDIS_PASSWORD"} --no-auth-warning GET "gowind:captcha:$1" | tr -d '\r\n'; }

# 登录
CAP=$(curl -s $R/captcha); CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
ANS=$(captcha_ans "$CID")
ENC=$(printf 'Abcd@1234' | openssl enc -aes-128-cbc -K $KEYHEX -iv $KEYHEX -base64 -A)
CODE=$(curl -s -o /tmp/sse_login.json -w "%{http_code}" -X POST $R/login -H 'Content-Type: application/json' -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" -d "{\"username\":\"admin\",\"password\":\"$ENC\",\"grant_type\":\"password\",\"client_type\":\"admin\"}")
TOKEN=$(cat /tmp/sse_login.json | python3 -c "import sys,json;print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
chk login 200 "$CODE" "token_len=${#TOKEN}"

# 1. OPTIONS 预检 → 204 + CORS 头
OPT=$(curl -s -o /dev/null -D /tmp/sse_opt.h -w "%{http_code}" -X OPTIONS "$SSE?stream=1")
chk sse-options-204 204 "$OPT" "$(grep -i 'access-control-allow-methods' /tmp/sse_opt.h)"
grep -qi "access-control-allow-origin: \*" /tmp/sse_opt.h && chk sse-cors-origin 0 0 "origin * ok" || chk sse-cors-origin x 0 "origin missing"

# 2. 无 token → 401（Go: token empty → Authenticate 失败 → 401）
NT=$(curl -s -o /tmp/sse_notok.txt -w "%{http_code}" "$SSE?stream=1")
chk sse-no-token-401 401 "$NT" "$(cat /tmp/sse_notok.txt)"

# 3. 坏 token → 401
BT=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer bad.token.here" "$SSE?stream=1")
chk sse-bad-token-401 401 "$BT"

# 4. 越权：stream != uid → 401（Go HandleAuthorize: stream user mismatch → Forbidden）
MM=$(curl -s -o /tmp/sse_mm.txt -w "%{http_code}" -H "Authorization: Bearer $TOKEN" "$SSE?stream=999")
chk sse-stream-mismatch-401 401 "$MM" "$(cat /tmp/sse_mm.txt)"

# 5. 缺 stream 参数 → 500（Go: Please specify a stream!）
NS=$(curl -s -o /tmp/sse_ns.txt -w "%{http_code}" -H "Authorization: Bearer $TOKEN" "$SSE")
chk sse-no-stream-500 500 "$NS" "$(cat /tmp/sse_ns.txt)"

# 6. 实时推送链路：先订阅（后台 curl，max-time 自动断开），再发站内信，期待收到 notification 帧
(curl -sN --max-time 12 -H "Authorization: Bearer $TOKEN" "$SSE?stream=1" > /tmp/sse_stream.txt) &
SUB_PID=$!
sleep 1
SEND_CODE=$(curl -s -o /tmp/sse_send.json -w "%{http_code}" -X POST $R/internal-message/send -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"type":"NOTIFICATION","recipientUserId":1,"title":"sse-smoke-title","content":"sse-smoke-content"}')
chk sse-send-message 200 "$SEND_CODE" "$(cat /tmp/sse_send.json | head -c 120)"
# 等待帧到达
for i in $(seq 1 20); do grep -q "event: notification" /tmp/sse_stream.txt 2>/dev/null && break; sleep 0.5; done
wait $SUB_PID 2>/dev/null
if grep -q "event: notification" /tmp/sse_stream.txt; then
  chk sse-frame-received 0 0 "$(tr '\n' '|' < /tmp/sse_stream.txt | head -c 200)"
  # 帧结构：id → data → event
  grep -q "^id: " /tmp/sse_stream.txt && chk sse-frame-id 0 0 "id line ok" || chk sse-frame-id x 0 "id line missing"
  grep -q '^data: .*"messageId"' /tmp/sse_stream.txt && chk sse-frame-data-json 0 0 "$(grep '^data:' /tmp/sse_stream.txt | head -c 200)" || chk sse-frame-data-json x 0 "data json missing"
  grep -q '"title":"sse-smoke-title"' /tmp/sse_stream.txt && chk sse-frame-payload 0 0 "payload ok" || chk sse-frame-payload x 0 "payload mismatch"
  grep -q '"recipientUserId":1' /tmp/sse_stream.txt && chk sse-frame-uid 0 0 "uid ok" || chk sse-frame-uid x 0 "uid mismatch"
else
  chk sse-frame-received 0 1 "NO notification frame: $(tr '\n' '|' < /tmp/sse_stream.txt | head -c 200)"
fi

# 7. 离线用户：无订阅者时发送仍成功（try_publish 跳过，不报错）
OFF=$(curl -s -o /dev/null -w "%{http_code}" -X POST $R/internal-message/send -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"type":"NOTIFICATION","recipientUserId":1,"title":"offline-ok","content":"x"}')
chk sse-offline-send-200 200 "$OFF"

# 8. X-Token 头取 token（Go DefaultTokenExtractor 第二优先级）
XT=$(curl -s -o /dev/null -w "%{http_code}" -H "X-Token: $TOKEN" "$SSE?stream=wrong-uid")
chk sse-x-token-authz 401 "$XT" "X-Token 被解析并执行了越权校验（401=mismatch）"

echo; echo "==== SSE smoke: pass=$pass fail=$fail ===="
exit $fail
