package service

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/tx7do/go-utils/timeutil"
	"github.com/tx7do/go-utils/trans"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"github.com/xuri/excelize/v2"
	"google.golang.org/protobuf/types/known/timestamppb"

	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"

	auditV1 "go-wind-admin/api/gen/go/audit/service/v1"

	"go-wind-admin/app/admin/service/internal/data"
	"go-wind-admin/pkg/middleware/auth"

	"github.com/tx7do/go-crud/viewer"
	appViewer "go-wind-admin/pkg/entgo/viewer"
)

// 审计日志服务端导出的单次行数上限。
// 全量导出的意义是突破前端客户端聚合的 1 万行上限，但无上限会把一次点击变成
// 一次无界查询（OOM 风险）；50 万行约等于等保场景下几个月的量，真要更多走归档 JSONL。
const auditExportMaxRows = 500000

// auditExportColumn 一列的定义：表头 + 从行 DTO 取值的取值器（返回已格式化的字符串）。
type auditExportColumn[T any] struct {
	header string
	value  func(row T) string
}

// AuditExportService 审计日志服务端导出（XLSX/CSV，流式二进制响应）。
// 五类审计日志共用一个端点，按 type 分派；条件与列表页同源（query JSON 透传），
// 因此"导出的就是当前搜索看到的"。
type AuditExportService struct {
	loginRepo      *data.LoginAuditLogRepo
	apiRepo        *data.ApiAuditLogRepo
	operationRepo  *data.OperationAuditLogRepo
	dataAccessRepo *data.DataAccessAuditLogRepo
	permissionRepo *data.PermissionAuditLogRepo
	log            *bLogger.Helper

	// tokenChecker 手动路由的鉴权锚：导出端点未设 Operation，auth 中间件的
	// selector 不会应用（也就不会注入操作人），须自行验 Bearer token。
	tokenChecker auth.AccessTokenChecker
}

func NewAuditExportService(
	ctx *bootstrap.Context,
	loginRepo *data.LoginAuditLogRepo,
	apiRepo *data.ApiAuditLogRepo,
	operationRepo *data.OperationAuditLogRepo,
	dataAccessRepo *data.DataAccessAuditLogRepo,
	permissionRepo *data.PermissionAuditLogRepo,
	tokenChecker auth.AccessTokenChecker,
) *AuditExportService {
	return &AuditExportService{
		loginRepo:      loginRepo,
		apiRepo:        apiRepo,
		operationRepo:  operationRepo,
		dataAccessRepo: dataAccessRepo,
		permissionRepo: permissionRepo,
		log:            ctx.NewLoggerHelper("audit-export/service/admin-service"),
		tokenChecker:   tokenChecker,
	}
}

// ServeExport 手动注册的 HTTP handler（要写二进制响应头，走不了 proto 生成路由）。
// 鉴权：手动路由不带 Operation，auth 中间件的 selector 不会应用（也就不会注入
// 操作人），在此自行验 Bearer token；平台管理员闸与五类审计读接口同一道。
func (s *AuditExportService) ServeExport(w http.ResponseWriter, r *http.Request) error {
	// 手动路由未经 auth 中间件，自行验 token 并把操作人塞进 context
	// （下游的租户谓词/审计链路依赖它）。
	authzHeader := r.Header.Get("Authorization")
	token := strings.TrimPrefix(authzHeader, "Bearer ")
	if token == "" || token == authzHeader {
		http.Error(w, "unauthorized: bearer token required", http.StatusUnauthorized)
		return nil
	}
	valid, operator := s.tokenChecker.IsValidAccessToken(r.Context(), token, false)
	if !valid || operator == nil {
		http.Error(w, "unauthorized: invalid token", http.StatusUnauthorized)
		return nil
	}
	ctx := auth.NewContext(r.Context(), operator)
	if !operator.GetIsPlatformAdmin() {
		http.Error(w, "forbidden: platform admin only", http.StatusForbidden)
		return nil
	}

	// ent viewer 注入：auth 中间件的 WithInjectEnt(true) 对手动路由不生效，
	// 而 TenantPrivacy 在 viewer 缺失时直接拒绝查询（与 asynq handler 的
	// viewer 重建同款——平台管理员 SystemViewer，租户用户按其租户建 viewer）。
	if operator.GetTenantId() == 0 {
		ctx = appViewer.NewSystemViewerContext(ctx)
	} else {
		ctx = viewer.WithContext(ctx, appViewer.NewUserViewer(
			uint64(operator.GetUserId()), uint64(operator.GetTenantId()), 0, "", nil,
		))
	}

	q := r.URL.Query()
	logType := q.Get("type")
	format := strings.ToLower(q.Get("format"))
	if format == "" {
		format = "xlsx"
	}
	if format != "xlsx" && format != "csv" {
		http.Error(w, "format must be xlsx or csv", http.StatusBadRequest)
		return nil
	}
	maxRows := auditExportMaxRows
	if v := q.Get("maxRows"); v != "" {
		if parsed, err := strconv.Atoi(v); err == nil && parsed > 0 && parsed < auditExportMaxRows {
			maxRows = parsed
		}
	}

	req := &paginationV1.PagingRequest{
		NoPaging: trans.Ptr(true),
		Limit:    trans.Ptr(uint32(maxRows)),
	}
	if query := strings.TrimSpace(q.Get("query")); query != "" {
		req.FilteringType = &paginationV1.PagingRequest_Query{Query: query}
	}

	var (
		fileName string
		sheet    string
		headers  []string
		rows     [][]string
	)

	switch logType {
	case "login":
		resp, listErr := s.loginRepo.List(ctx, req)
		if listErr != nil {
			// 原始错误必须留日志：这里只有"query failed"回给客户端，不含原因
			s.log.Errorf(ctx, "audit export [%s] query failed: %s", logType, listErr.Error())
			http.Error(w, "query failed", http.StatusInternalServerError)
			return nil
		}
		fileName, sheet = "login-audit-logs", "login"
		cols := loginExportColumns()
		headers = colHeaders(cols)
		for _, row := range resp.GetItems() {
			rows = append(rows, colValues(cols, row))
		}
	case "api":
		resp, listErr := s.apiRepo.List(ctx, req)
		if listErr != nil {
			// 原始错误必须留日志：这里只有"query failed"回给客户端，不含原因
			s.log.Errorf(ctx, "audit export [%s] query failed: %s", logType, listErr.Error())
			http.Error(w, "query failed", http.StatusInternalServerError)
			return nil
		}
		fileName, sheet = "api-audit-logs", "api"
		cols := apiExportColumns()
		headers = colHeaders(cols)
		for _, row := range resp.GetItems() {
			rows = append(rows, colValues(cols, row))
		}
	case "operation":
		resp, listErr := s.operationRepo.List(ctx, req)
		if listErr != nil {
			// 原始错误必须留日志：这里只有"query failed"回给客户端，不含原因
			s.log.Errorf(ctx, "audit export [%s] query failed: %s", logType, listErr.Error())
			http.Error(w, "query failed", http.StatusInternalServerError)
			return nil
		}
		fileName, sheet = "operation-audit-logs", "operation"
		cols := operationExportColumns()
		headers = colHeaders(cols)
		for _, row := range resp.GetItems() {
			rows = append(rows, colValues(cols, row))
		}
	case "data_access":
		resp, listErr := s.dataAccessRepo.List(ctx, req)
		if listErr != nil {
			http.Error(w, "query failed", http.StatusInternalServerError)
			return nil
		}
		fileName, sheet = "data-access-audit-logs", "data_access"
		cols := dataAccessExportColumns()
		headers = colHeaders(cols)
		for _, row := range resp.GetItems() {
			rows = append(rows, colValues(cols, row))
		}
	case "permission":
		resp, listErr := s.permissionRepo.List(ctx, req)
		if listErr != nil {
			// 原始错误必须留日志：这里只有"query failed"回给客户端，不含原因
			s.log.Errorf(ctx, "audit export [%s] query failed: %s", logType, listErr.Error())
			http.Error(w, "query failed", http.StatusInternalServerError)
			return nil
		}
		fileName, sheet = "permission-audit-logs", "permission"
		cols := permissionExportColumns()
		headers = colHeaders(cols)
		for _, row := range resp.GetItems() {
			rows = append(rows, colValues(cols, row))
		}
	default:
		http.Error(w, "type must be one of login/api/operation/data_access/permission", http.StatusBadRequest)
		return nil
	}

	stamp := time.Now().Format("20060102_150405")

	switch format {
	case "csv":
		w.Header().Set("Content-Type", "text/csv; charset=utf-8")
		w.Header().Set("Content-Disposition", fmt.Sprintf(`attachment; filename="%s-%s.csv"`, fileName, stamp))
		if err := writeCSV(w, headers, rows); err != nil {
			s.log.Errorf(ctx, "audit export [%s] csv write failed: %s", logType, err.Error())
			return nil
		}
	default:
		data, err := buildXLSX(sheet, headers, rows)
		if err != nil {
			s.log.Errorf(ctx, "audit export [%s] xlsx build failed: %s", logType, err.Error())
			http.Error(w, "build xlsx failed", http.StatusInternalServerError)
			return nil
		}
		w.Header().Set("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
		w.Header().Set("Content-Disposition", fmt.Sprintf(`attachment; filename="%s-%s.xlsx"`, fileName, stamp))
		_, _ = w.Write(data)
	}
	return nil
}

func colHeaders[T any](cols []auditExportColumn[T]) []string {
	headers := make([]string, 0, len(cols))
	for _, c := range cols {
		headers = append(headers, c.header)
	}
	return headers
}

func colValues[T any](cols []auditExportColumn[*T], row *T) []string {
	values := make([]string, 0, len(cols))
	for _, c := range cols {
		values = append(values, c.value(row))
	}
	return values
}

func writeCSV(w http.ResponseWriter, headers []string, rows [][]string) error {
	// 手写 CSV（RFC 4180）：含引号/逗号/换行的字段加引号转义；UTF-8 BOM 让 Excel 正确识别中文
	if _, err := w.Write([]byte{0xEF, 0xBB, 0xBF}); err != nil {
		return err
	}
	escape := func(v string) string {
		if strings.ContainsAny(v, ",\"\n\r") {
			return `"` + strings.ReplaceAll(v, `"`, `""`) + `"`
		}
		return v
	}
	lines := make([]string, 0, len(rows)+1)
	lines = append(lines, joinCSV(headers, escape))
	for _, row := range rows {
		lines = append(lines, joinCSV(row, escape))
	}
	_, err := w.Write([]byte(strings.Join(lines, "\r\n")))
	return err
}

func joinCSV(fields []string, escape func(string) string) string {
	out := make([]byte, 0, 32*len(fields))
	for i, f := range fields {
		if i > 0 {
			out = append(out, ',')
		}
		out = append(out, escape(f)...)
	}
	return string(out)
}

func buildXLSX(sheet string, headers []string, rows [][]string) ([]byte, error) {
	f := excelize.NewFile()
	defer func() { _ = f.Close() }()
	if _, err := f.NewSheet(sheet); err != nil {
		return nil, err
	}
	if err := f.SetSheetName("Sheet1", sheet); err != nil {
		return nil, err
	}
	cell, _ := excelize.CoordinatesToCellName(1, 1)
	if err := f.SetSheetRow(sheet, cell, &headers); err != nil {
		return nil, err
	}
	if len(rows) > 0 {
		cell, _ := excelize.CoordinatesToCellName(1, 2)
		if err := f.SetSheetRow(sheet, cell, &rows); err != nil {
			return nil, err
		}
	}
	buf, err := f.WriteToBuffer()
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// ==== 各类型列定义（表头用英文字段名：导出文件没有运行时语言上下文，
// 后端文案不进 i18n 是仓库铁律的边界）====

func fmtTime(ts *timestamppb.Timestamp) string {
	t := timeutil.TimestamppbToTime(ts)
	if t == nil {
		return ""
	}
	return t.Format("2006-01-02 15:04:05")
}

func boolStr(v *bool) string {
	if v == nil {
		return ""
	}
	return strconv.FormatBool(*v)
}

func loginExportColumns() []auditExportColumn[*auditV1.LoginAuditLog] {
	return []auditExportColumn[*auditV1.LoginAuditLog]{
		{header: "id", value: func(r *auditV1.LoginAuditLog) string { return strconv.FormatUint(uint64(r.GetId()), 10) }},
		{header: "createdAt", value: func(r *auditV1.LoginAuditLog) string { return fmtTime(r.GetCreatedAt()) }},
		{header: "username", value: func(r *auditV1.LoginAuditLog) string { return r.GetUsername() }},
		{header: "tenantName", value: func(r *auditV1.LoginAuditLog) string { return r.GetTenantName() }},
		{header: "actionType", value: func(r *auditV1.LoginAuditLog) string { return r.GetActionType().String() }},
		{header: "status", value: func(r *auditV1.LoginAuditLog) string { return r.GetStatus().String() }},
		{header: "loginMethod", value: func(r *auditV1.LoginAuditLog) string { return r.GetLoginMethod().String() }},
		{header: "mfaStatus", value: func(r *auditV1.LoginAuditLog) string { return r.GetMfaStatus() }},
		{header: "failureReason", value: func(r *auditV1.LoginAuditLog) string { return r.GetFailureReason() }},
		{header: "ipAddress", value: func(r *auditV1.LoginAuditLog) string { return r.GetIpAddress() }},
		{header: "deviceInfo", value: func(r *auditV1.LoginAuditLog) string { return r.GetDeviceInfo().String() }},
		{header: "requestId", value: func(r *auditV1.LoginAuditLog) string { return r.GetRequestId() }},
		{header: "traceId", value: func(r *auditV1.LoginAuditLog) string { return r.GetTraceId() }},
	}
}

func apiExportColumns() []auditExportColumn[*auditV1.ApiAuditLog] {
	return []auditExportColumn[*auditV1.ApiAuditLog]{
		{header: "id", value: func(r *auditV1.ApiAuditLog) string { return strconv.FormatUint(uint64(r.GetId()), 10) }},
		{header: "createdAt", value: func(r *auditV1.ApiAuditLog) string { return fmtTime(r.GetCreatedAt()) }},
		{header: "username", value: func(r *auditV1.ApiAuditLog) string { return r.GetUsername() }},
		{header: "tenantName", value: func(r *auditV1.ApiAuditLog) string { return r.GetTenantName() }},
		{header: "httpMethod", value: func(r *auditV1.ApiAuditLog) string { return r.GetHttpMethod() }},
		{header: "path", value: func(r *auditV1.ApiAuditLog) string { return r.GetPath() }},
		{header: "apiModule", value: func(r *auditV1.ApiAuditLog) string { return r.GetApiModule() }},
		{header: "ipAddress", value: func(r *auditV1.ApiAuditLog) string { return r.GetIpAddress() }},
		{header: "requestId", value: func(r *auditV1.ApiAuditLog) string { return r.GetRequestId() }},
	}
}

func operationExportColumns() []auditExportColumn[*auditV1.OperationAuditLog] {
	return []auditExportColumn[*auditV1.OperationAuditLog]{
		{header: "id", value: func(r *auditV1.OperationAuditLog) string { return strconv.FormatUint(uint64(r.GetId()), 10) }},
		{header: "createdAt", value: func(r *auditV1.OperationAuditLog) string { return fmtTime(r.GetCreatedAt()) }},
		{header: "username", value: func(r *auditV1.OperationAuditLog) string { return r.GetUsername() }},
		{header: "tenantName", value: func(r *auditV1.OperationAuditLog) string { return r.GetTenantName() }},
		{header: "resourceType", value: func(r *auditV1.OperationAuditLog) string { return r.GetResourceType() }},
		{header: "resourceId", value: func(r *auditV1.OperationAuditLog) string { return r.GetResourceId() }},
		{header: "action", value: func(r *auditV1.OperationAuditLog) string { return r.GetAction().String() }},
		{header: "success", value: func(r *auditV1.OperationAuditLog) string { return boolStr(r.Success) }},
		{header: "sensitiveLevel", value: func(r *auditV1.OperationAuditLog) string { return r.GetSensitiveLevel().String() }},
		{header: "requestId", value: func(r *auditV1.OperationAuditLog) string { return r.GetRequestId() }},
	}
}

func dataAccessExportColumns() []auditExportColumn[*auditV1.DataAccessAuditLog] {
	return []auditExportColumn[*auditV1.DataAccessAuditLog]{
		{header: "id", value: func(r *auditV1.DataAccessAuditLog) string { return strconv.FormatUint(uint64(r.GetId()), 10) }},
		{header: "createdAt", value: func(r *auditV1.DataAccessAuditLog) string { return fmtTime(r.GetCreatedAt()) }},
		{header: "username", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetUsername() }},
		{header: "tenantName", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetTenantName() }},
		{header: "dataSource", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetDataSource() }},
		{header: "tableName", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetTableName() }},
		{header: "dataId", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetDataId() }},
		{header: "accessType", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetAccessType().String() }},
		{header: "affectedRows", value: func(r *auditV1.DataAccessAuditLog) string { return strconv.FormatInt(int64(r.GetAffectedRows()), 10) }},
		{header: "sqlDigest", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetSqlDigest() }},
		{header: "ipAddress", value: func(r *auditV1.DataAccessAuditLog) string { return r.GetIpAddress() }},
	}
}

func permissionExportColumns() []auditExportColumn[*auditV1.PermissionAuditLog] {
	return []auditExportColumn[*auditV1.PermissionAuditLog]{
		{header: "id", value: func(r *auditV1.PermissionAuditLog) string { return strconv.FormatUint(uint64(r.GetId()), 10) }},
		{header: "createdAt", value: func(r *auditV1.PermissionAuditLog) string { return fmtTime(r.GetCreatedAt()) }},
		{header: "operatorName", value: func(r *auditV1.PermissionAuditLog) string { return r.GetOperatorName() }},
		{header: "targetType", value: func(r *auditV1.PermissionAuditLog) string { return r.GetTargetType() }},
		{header: "targetName", value: func(r *auditV1.PermissionAuditLog) string { return r.GetTargetName() }},
		{header: "action", value: func(r *auditV1.PermissionAuditLog) string { return r.GetAction().String() }},
		{header: "reason", value: func(r *auditV1.PermissionAuditLog) string { return r.GetReason() }},
		{header: "ipAddress", value: func(r *auditV1.PermissionAuditLog) string { return r.GetIpAddress() }},
	}
}
