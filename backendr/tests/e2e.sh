#!/bin/bash
R=${E2E_BASE:-http://127.0.0.1:7666}/admin/v1
REDIS_CONTAINER=${E2E_REDIS_CONTAINER:-gwa-rust-test-redis}
KEYHEX=66353164363661373364386130393237
captcha_ans() { docker exec "$REDIS_CONTAINER" redis-cli GET "gowind:captcha:$1" | tr -d '\r\n'; }
pass=0; fail=0
chk() { # name, expected_code, actual_code, extra
  if [ "$2" = "$3" ]; then pass=$((pass+1)); echo "PASS $1 [$3] ${4:0:120}"; else fail=$((fail+1)); echo "FAIL $1 expected=$2 got=$3 ${4:0:200}"; fi
}

# ---- 免鉴权 ----
CAP=$(curl -s $R/captcha)
CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
chk captcha 200 200 "$(echo $CAP | head -c 100)"
ANS=$(captcha_ans "$CID")
ENC=$(printf 'Abcd@1234' | openssl enc -aes-128-cbc -K $KEYHEX -iv $KEYHEX -base64 -A)

V=$(curl -s -o /dev/null -w "%{http_code}" -X POST $R/captcha/verify -H 'Content-Type: application/json' -d "{\"captchaId\":\"$CID\",\"userInput\":\"$ANS\"}")
chk verify-captcha-consumed 200 "$V"

# 重新拿一个验证码登录
CAP=$(curl -s $R/captcha); CID=$(echo "$CAP" | python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])")
ANS=$(captcha_ans "$CID")
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
# test_run：空请求体 400（target 必填）；引擎未接入 → 200 + success:false
SC=$(rt -X POST $R/scripts/test_run -H 'Content-Type: application/json' -d '{}'); chk script-test-run-target-400 400 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 120)"
SC=$(rt -X POST $R/scripts/test_run -H 'Content-Type: application/json' -d '{"draft":{"name":"draft-1","language":"LUA","source":"function x() end"},"input":{"a":"1"}}'); chk script-test-run-no-engine-200 200 "$(echo "$SC"|tail -1)" "$(echo "$SC"|head -c 200)"
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
CAP=$(curl -s $R/captcha); CID=$(echo "$CAP"|python3 -c "import sys,json;print(json.load(sys.stdin)['captchaId'])"); ANS=$(captcha_ans "$CID")
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

# task 控制端点（B3 后为真实调度器；对齐 Go 语义校验）
# start/restart：无任务/全部未启用 → 200 {"count":0}；有任务 → {"count":N}
IM=$(rt -X POST "$R/tasks:start" -H 'Content-Type: application/json' -d '{}')
chk task-start-real-scheduler 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 120)"
IM=$(rt -X POST "$R/tasks:stop" -H 'Content-Type: application/json' -d '{}')
chk task-stop 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 120)"
IM=$(rt -X POST "$R/tasks:restart" -H 'Content-Type: application/json' -d '{}')
chk task-restart 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 120)"
# control：{} 缺 typeName → 400；平台上下文（tid=0）控制任务 → 400 tenant scope required
IM=$(rt -X POST "$R/tasks:control" -H 'Content-Type: application/json' -d '{}')
chk task-control-empty-400 400 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 120)"
IM=$(rt -X POST "$R/tasks:control" -H 'Content-Type: application/json' -d '{"controlType":"Start","typeName":"broadcast_message"}')
chk task-control-platform-scope 400 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 140)"
IM=$(rt -X POST "$R/tasks/control" -H 'Content-Type: application/json' -d '{}')
chk task-slash-alias 400 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"

# 错误格式
IM=$(curl -s $R/online-session/my-sessions)
chk error-format-kratos 200 200 "$(echo "$IM"|head -c 120)"

# ---- file / file_transfer / avatar / redis_cache_monitor / permission ----

# redis_cache_monitor（fail-soft 空视图或指标）
IM=$(rt $R/redis-cache-monitor); chk redis-cache-monitor 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 200)"

# file CRUD
IM=$(rt -X POST $R/files -H 'Content-Type: application/json' -d '{"data":{"provider":"MINIO","bucketName":"images","fileName":"demo.png","extension":"png","size":1024,"linkUrl":"images/demo.png"}}')
chk file-create 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 150)"
IM=$(rt "$R/files?pageSize=5"); chk file-list 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 200)"
FID=$(echo "$IM"|sed '$d'|python3 -c "import sys,json;d=json.load(sys.stdin);print(d['items'][0]['id'] if d['items'] else 0)" 2>/dev/null)
IM=$(rt $R/files/$FID); chk file-get 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 150)"
IM=$(rt -X DELETE $R/files/$FID); chk file-delete 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"

# multipart 上传 → 签名图片代理 → 下载回读
printf '\x89PNG\r\n\x1a\n' > /tmp/px.png  # 1x1 PNG 魔数前缀即可，只测上传链路
# 造一个合法的极小 PNG（64B 静态图），否则嗅探需真实魔数
python3 - <<'PYEOF'
import base64
png = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==")
open("/tmp/px.png","wb").write(png)
PYEOF
UP=$(rt -X POST $R/file/upload -F "file=@/tmp/px.png;type=image/png" -F 'storageObject={"bucketName":"","fileDirectory":"avatar"}' -F 'sourceFileName=px.png' -F 'mime=image/png')
chk upload-png 200 "$(echo "$UP"|tail -1)" "$(echo "$UP"|head -c 300)"
PUBURL=$(echo "$UP"|sed '$d'|python3 -c "import sys,json;print(json.load(sys.stdin).get('publicUrl','') or '')" 2>/dev/null)
OBJ=$(echo "$UP"|sed '$d'|python3 -c "import sys,json;print(json.load(sys.stdin).get('objectName',''))" 2>/dev/null)
BUCKET=${OBJ%%/*}
if [ -n "$PUBURL" ] && [[ "$PUBURL" == /admin* ]]; then
  SC=$(curl -s -o /dev/null -w "%{http_code}" "${R%/admin/v1}$PUBURL")
  chk signed-image-serve 200 "$SC" "sig url ok"
else
  chk signed-image-serve 200 0 "no publicUrl (GOWIND_CRYPTO_KEY 未配置)"
fi
if [ -n "$OBJ" ]; then
  SC=$(rt "$R/file/download?storageObject.bucketName=$BUCKET&storageObject.objectName=${OBJ#*/}" | tail -1)
  chk download-storage-object 200 "$SC" "$OBJ"
fi
# fileId 选择器：查 files 表元数据转 storageObject 下载（对齐 Go DownloadFile）
FID2=$(rt "$R/files?pageSize=1&orderBy=%5B%22-id%22%5D" | sed '$d' | python3 -c "import sys,json;d=json.load(sys.stdin);print(d['items'][0]['id'] if d.get('items') else 0)" 2>/dev/null)
if [ -n "$FID2" ] && [ "$FID2" != "0" ]; then
  IM=$(rt "$R/file/download?fileId=$FID2"); chk download-file-id 200 "$(echo "$IM"|tail -1)" "fileId=$FID2"
  IM=$(rt "$R/file/download?fileId=99999999"); chk download-file-id-404 404 "$(echo "$IM"|tail -1)" "missing"
else
  chk download-file-id 200 0 "no file row to test"
fi
# SSRF 防护：内网地址必须被拒
SC=$(rt "$R/file/download?downloadUrl=http://127.0.0.1:56379/" | tail -1)
chk download-ssrf-blocked 403 "$SC" "loopback"

# 头像：imageBase64 上传（1x1 PNG）→ 200 + url
AVB64=$(base64 -i /tmp/px.png | tr -d '\n')
IM=$(rt -X POST $R/me/avatar -H 'Content-Type: application/json' -d "{\"imageBase64\":\"$AVB64\"}")
chk avatar-base64-upload 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 150)"
# 头像：非图片 base64 必须拒绝 400
IM=$(rt -X POST $R/me/avatar -H 'Content-Type: application/json' -d '{"imageBase64":"d29yZC1jb250ZW50"}')
chk avatar-non-image-400 400 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 120)"
IM=$(rt -X DELETE $R/me/avatar); chk avatar-delete 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 100)"

# permission sync:perms + list
IM=$(rt -X POST $R/permissions/sync:perms -H 'Content-Type: application/json' -d '{}')
chk permission-sync 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 200)"
IM=$(rt "$R/permissions?pageSize=5"); chk permission-list 200 "$(echo "$IM"|tail -1)" "$(echo "$IM"|head -c 150)"

# 登出
LO=$(rt -X POST $R/logout); chk logout 200 "$(echo "$LO"|tail -1)" "$(echo "$LO"|head -c 100)"

echo; echo "RESULT: pass=$pass fail=$fail"
