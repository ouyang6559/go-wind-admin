/**
 * CSV 导出工具（vben 端）：按传入行/表头生成带 BOM 的 CSV 并触发下载。
 * 全量聚合由各页面组合 fetchList* + 分页循环实现（上限自控）。
 */

export interface CsvColumn {
  title: string;
  key: string;
}

function escapeCsvCell(value: unknown): string {
  if (value === null || value === undefined) return '';
  const str = String(value);
  if (/[",\n\r]/.test(str)) {
    return `"${str.replace(/"/g, '""')}"`;
  }
  return str;
}

export function downloadCsv(filename: string, columns: CsvColumn[], rows: any[]): void {
  const header = columns.map((c) => escapeCsvCell(c.title)).join(',');
  const lines = rows.map((row) =>
    columns.map((c) => escapeCsvCell((row as any)?.[c.key])).join(','),
  );
  // BOM：保证 Excel 打开中文不乱码
  const content = `\uFEFF${header}\n${lines.join('\n')}`;
  const blob = new Blob([content], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
