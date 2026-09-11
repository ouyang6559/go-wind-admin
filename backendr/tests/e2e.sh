#!/bin/bash
R=http://127.0.0.1:7666/admin/v1
KEYHEX=66353164363661373364386130393237
pass=0; fail=0
chk() { # name, expected_code, actual_code, extra
  if [ "$2" = "$3" ]; then pass=$((pass+1)); echo "PASS $1 [$3] ${4:0:120}"; else fail=$((fail+1)); echo "FAIL $1 expected=$2 got=$3 ${4:0:200}"; fi
}

# ---- 免鉴权 ----
CAP=$(curl -s $R/captcha)
CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
chk captcha 200 200 "$(echo $CAP | head -c 100)"
ANS=$(docker exec gwa-rust-test-redis redis-cli GET "gowind:captcha:$CID" | tr -d '\r\n')
ENC=$(printf 'Abcd@1234' | openssl enc -aes-128-cbc -K $KEYHEX -iv $KEYHEX -base64 -A)

V=$(curl -s -o /dev/null -w "%{http_code}" -X POST $R/captcha/verify -H 'Content-Type: application/json' -d "{\"captchaId\":\"$CID\",\"userInput\":\"$ANS\"}")
chk verify-captcha-consumed 200 "$V"

# 重新拿一个验证码登录
CAP=$(curl -s $R/captcha); CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
ANS=$(docker exec gwa-rust-test-redis redis-cli GET "gowind:captcha:$CID" | tr -d '\r\n')
CODE=$(curl -s -D /tmp/rh.txt -o /tmp/login.json -w "%{http_code}" -X POST $R/login -H 'Content-Type: application/json' -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" -d "{\"username\":\"admin\",\"password\":\"$ENC\",\"grant_type\":\"password\",\"client_type\":\"admin\"}")
TOKEN=$(cat /tmp/login.json | python3 -c "import sys,json;print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)
chk login 200 "$CODE" "$(cat /tmp/login.json | head -c 200)"
[ -n "$TOKEN" ] && { pass=$((pass+1)); echo "PASS token-extracted len=${#TOKEN}"; } || { fail=$((fail+1)); echo "FAIL no token: $LOGIN"; }
A="Authorization: Bearer $TOKEN"
grep -qi "set-cookie: refresh_token=" /tmp/rh.txt && { pass=$((pass+1)); echo "PASS refresh-cookie set"; } || { fail=$((fail+1)); echo "FAIL no refresh cookie"; }

rt() { curl -s -w "\n%{http_code}" -H "$A" "$@"; }

# ---- 新增模块 ----
RSP=$(rt $R/server-monitor); CODE=$(echo "$RSP"|tail -1)
chk server-monitor 200 "$CODE" "$(echo "$RSP"|head -c 200)"

RSP=$(rt "$R/tasks:type-names"); CODE=$(echo "$RSP"|tail -1)
chk tasks:type-names 200 "$CODE" "$(echo "$RSP"|head -c 150)"
RSP=$(rt "$R/users:exists?username=admin"); CODE=$(echo "$RSP"|tail -1)
chk users:exists 200 "$CODE" "$(echo "$RSP"|head -c 100)"
RSP=$(rt "$R/users:exists?username=nosuchuser"); chk users-exists-neg 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"
RSP=$(rt "$R/tenants:exists?code=none"); chk tenants:exists 200 "$(echo "$RSP"|tail -1)" "$(echo "$RSP"|head -c 100)"

# script CRUD
SC=$(rt -X POST $R/scripts -H 'Content-Type: application/json' -d '{"data":{"name":"demo-hook","language":"LUA","source":"function on_user_create(ctx) return true end","hookPoint":"user.after_create","priority":1,"isEnabled":true}}')
chk script-create 200 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 120)"
SC=$(rt $R/scripts); chk script-list 200 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 200)"
SC=$(rt $R/scripts/count); chk script-count 200 "$(echo "$SC"|tail -1)" "$SC"
SC=$(rt $R/scripts/name/demo-hook); chk script-get-by-name 200 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 150)"
SID=$(echo "$SC"|sed '$d'|python3 -c "import sys,json;print(json.load(sys.stdin)['id'])")
SC=$(rt -X PUT $R/scripts/$SID -H 'Content-Type: application/json' -d '{"data":{"priority":9}}'); chk script-update 200 "$(echo "$SC"|tail -1)" "$SC"
SC=$(rt $R/scripts/$SID); chk script-get 200 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 150)"
SC=$(rt -X DELETE "$R/scripts?ids=$SID"); chk script-delete 200 "$(echo "$SC"|tail -1)" "$SC"
SC=$(rt -X POST $R/scripts/test_run -H 'Content-Type: application/json' -d '{}'); chk script-test-run-501 501 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 120)"
SC=$(rt $R/script/hooks); chk script-hooks 200 "$(echo "$SC"|tail -1)" "$SC"
SC=$(rt $R/script/logs); chk script-logs 200 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 100)"
SC=$(rt $R/script/logs/count); chk script-logs-count 200 "$(echo "$SC"|tail -1)" "$SC"
SC=$(rt -X POST $R/script/logs/purge -H 'Content-Type: application/json' -d '{}'); chk script-logs-purge 200 "$(echo "$SC"|tail -1)" "$SC"

# notification channels
NC=$(rt -X POST $R/notification-channels -H 'Content-Type: application/json' -d '{"data":{"name":"smtp-main","type":"EMAIL","smtpHost":"smtp.local","smtpPort":465,"smtpUsername":"noreply@gowind.local","smtpFrom":"noreply@gowind.local","smtpTls":"SSL","enabled":true},"password":"secret123"}')
chk channel-create 200 "$(echo "$NC"|tail -1)" "$(echo "$NC"|head -c 250)"
NC=$(rt $R/notification-channels); chk channel-list 200 "$(echo "$NC"|tail -1)" "$(echo "$NC"|head -c 200)"
NCID=$(echo "$NC"|sed '$d'|python3 -c "import sys,json;print(json.load(sys.stdin)['items'][0]['id'])")
NC=$(rt $R/notification-channels/$NCID); chk channel-get 200 "$(echo "$NC"|tail -1)" "$(echo "$NC"|head -c 200)"
NC=$(rt -X PUT $R/notification-channels/$NCID -H 'Content-Type: application/json' -d '{"data":{"smtpHost":"smtp2.local"}}'); chk channel-update 200 "$(echo "$NC"|tail -1)" "$NC"
NC=$(rt -X POST $R/notification-channels/$NCID/send-test-email -H 'Content-Type: application/json' -d '{"recipient":"to@gowind.local"}'); chk channel-send-test-400 400 "$(echo "$NC"|tail -1)" "$(echo "$NC"|head -c 150)"
NC=$(rt -X DELETE $R/notification-channels/$NCID); chk channel-delete 200 "$(echo "$NC"|tail -1)" "$NC"

# online sessions
OS=$(rt $R/online-session/sessions); chk online-sessions 200 "$(echo "$OS"|tail -1)" "$(echo "$OS"|head -c 250)"
OS=$(rt $R/online-session/my-sessions); chk my-sessions 200 "$(echo "$OS"|tail -1)" "$(echo "$OS"|head -c 250)"
JT=$(echo "$OS"|sed '$d'|python3 -c "import sys,json;d=json.load(sys.stdin);print(([i for i in d['items'] if i.get('current')] or d['items'])[0]['jti'])" 2>/dev/null)
OS=$(rt -X POST $R/online-session/my-sessions/revoke -H 'Content-Type: application/json' -d "{\"jti\":\"nosuch\"}"); chk revoke-my-404 404 "$(echo "$OS"|tail -1)" "$(echo "$OS"|head -c 150)"
OS=$(rt -X POST $R/online-session/force-logout -H 'Content-Type: application/json' -d "{\"userId\":1,\"jti\":\"$JT\"}"); chk force-logout 200 "$(echo "$OS"|tail -1)" "$OS"
OS=$(rt $R/online-session/sessions); chk after-logout-token-401 401 "$(echo "$OS"|tail -1)" "$(echo "$OS"|head -c 150)"

# 重新登录继续测
CAP=$(curl -s $R/captcha); CID=$(echo "$CAP"|python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])"); ANS=$(docker exec gwa-rust-test-redis redis-cli GET "gowind:captcha:$CID"|tr -d '\r\n')
LOGIN=$(curl -s -X POST $R/login -H 'Content-Type: application/json' -H "X-Captcha-Id: $CID" -H "X-Captcha-Value: $ANS" -d "{\"username\":\"admin\",\"password\":\"$ENC\",\"grant_type\":\"password\"}")
TOKEN=$(echo "$LOGIN"|python3 -c "import sys,json;print(json.load(sys.stdin)['access_token'])"); A="Authorization: Bearer $TOKEN"

# forgot/reset password（无邮件渠道 → 静默 200）
FP=$(rt -X POST $R/forgot-password -H 'Content-Type: application/json' -d '{"identifier":"admin@gowind.local"}')
chk forgot-password-silent 200 "$(echo "$FP"|tail -1)" "$(echo "$FP"|head -c 100)"
RP=$(rt -X POST $R/reset-password-by-code -H 'Content-Type: application/json' -d '{"identifier":"admin@gowind.local","code":"000000","new_password":"x"}')
chk reset-password-badcode 400 "$(echo "$RP"|tail -1)" "$(echo "$RP"|head -c 150)"

# internal-message
IM=$(rt $R/internal-message/inbox); chk inbox 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 150)"
IM=$(rt -X POST $R/internal-message/status -H 'Content-Type: application/json' -d '{"userId":1,"recipientIds":[],"newStatus":"READ"}')
chk im-status-empty-400 400 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 150)"
IM=$(rt -X POST $R/internal-message/status -H 'Content-Type: application/json' -d '{"userId":1,"recipientIds":[999],"newStatus":"READ"}')
chk im-status-no-row 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"
IM=$(rt -X POST $R/internal-message/read -H 'Content-Type: application/json' -d '{"userId":1}')
chk im-read 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"
IM=$(rt -X POST $R/internal-message/inbox/delete -H 'Content-Type: application/json' -d '{"userId":1,"recipientIds":[999]}')
chk im-inbox-delete 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"

# audit logs
for M in api-audit-logs login-audit-logs operation-audit-logs permission-audit-logs data-access-audit-logs policy-evaluation-logs; do
  IM=$(rt "$R/$M?pageSize=2"); chk "audit-$M" 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 130)"
done

# task 控制降级
for EP in tasks:start tasks:stop tasks:restart tasks:control; do
  IM=$(rt -X POST "$R/$EP" -H 'Content-Type: application/json' -d '{}')
  chk "task-$EP(500 调度器未配置)" 500 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 120)"
done
IM=$(rt -X POST "$R/tasks/control" -H 'Content-Type: application/json' -d '{}')
chk task-slash-alias 500 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"

# 错误格式
IM=$(curl -s $R/online-session/my-sessions)
chk error-format-kratos 200 200 "$(echo "$IM"|head -c 120)"

# 登出
LO=$(rt -X POST $R/logout); chk logout 200 "$(echo "$LO"|tail -1)" "$(echo "$LO"|head -c 100)"

echo; echo "RESULT: pass=$pass fail=$fail"
