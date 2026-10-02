import { useAccessStore } from "@/stores";

/** 服务端审计导出支持的日志类型（与后端 ExportService 的 type 参数一致） */
export type AuditExportLogType =
  | "api"
  | "data_access"
  | "login"
  | "operation"
  | "permission";

/**
 * 审计日志服务端导出：走 /admin/v1/audit-logs:export（XLSX/CSV，突破 ProPage
 * 导出弹窗客户端聚合的 1 万行上限，服务端上限 50 万）。
 * query 与列表页同源（contains 条件），导出的即当前搜索看到的。
 *
 * 手动 fetch 而非走 RequestClient：需要 blob 响应 + 触发浏览器下载，
 * axios 拦截器的 JSON 错误处理对二进制响应不适用。
 */
export async function exportAuditLogsServer(
  type: AuditExportLogType,
  queryJson: string,
  format: "csv" | "xlsx" = "xlsx"
): Promise<void> {
  const accessStore = useAccessStore();
  const token = accessStore.accessToken;
  const params = new URLSearchParams({ format, type });
  if (queryJson) {
    params.set("query", queryJson);
  }
  const res = await fetch(`/admin/v1/audit-logs:export?${params.toString()}`, {
    headers: token ? { Authorization: `Bearer ${token}` } : undefined,
  });
  if (!res.ok) {
    throw new Error(`export failed (${res.status})`);
  }
  const blob = await res.blob();
  const disposition = res.headers.get("Content-Disposition") || "";
  const filename =
    disposition.match(/filename="(.+)"/)?.[1] ??
    `audit-logs-${Date.now()}.${format}`;
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
  URL.revokeObjectURL(a.href);
}
