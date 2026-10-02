import { useAuthStore } from '@/stores';

/**
 * 审计日志服务端导出：走 /admin/v1/audit-logs:export（XLSX/CSV，突破客户端
 * 聚合导出的 1 万行上限，服务端上限 50 万）。query 与列表页同源（contains 条件），
 * 导出的即当前搜索看到的。
 */
export async function exportAuditLogsServer(
  type: 'api' | 'data_access' | 'login' | 'operation' | 'permission',
  queryJson: string,
  format: 'csv' | 'xlsx' = 'xlsx',
): Promise<void> {
  const token = useAuthStore.getState().accessToken;
  const params = new URLSearchParams({ format, type });
  if (queryJson) {
    params.set('query', queryJson);
  }
  const res = await fetch(`/admin/v1/audit-logs:export?${params.toString()}`, {
    headers: token ? { authorization: `Bearer ${token}` } : undefined,
  });
  if (!res.ok) {
    throw new Error(`export failed (${res.status})`);
  }
  const blob = await res.blob();
  const disposition = res.headers.get('content-disposition') || '';
  const filename = disposition.match(/filename="(.+)"/)?.[1] ?? `audit-logs-${Date.now()}.${format}`;
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
  URL.revokeObjectURL(a.href);
}
