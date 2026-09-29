import { useMemo } from 'react';
import { Skeleton, Table } from 'antd';
import type { TableColumnsType, TableProps } from 'antd';
import type { ProColumns } from '@ant-design/pro-components';

/** 单元格占位条宽度轮换表：整列等长会读成「数据已到位、内容恰好相同」，按 (行+列) 打散 */
const BAR_WIDTHS = ['78%', '62%', '84%', '54%', '70%'];

const SKELETON_ROW_KEY = '__listTableSkeletonRow__';

export interface TableSkeletonProps<RecordType extends Record<string, any>> {
  columns?: ProColumns<RecordType>[];
  rows: number;
  size?: TableProps<RecordType>['size'];
  bordered?: boolean;
  /** 只传横向 scroll：scroll.y 会让骨架多出一层限高 body，而这套页面渲染出的真实表格没有那层，两态结构不一致 */
  scrollX?: number | string | true;
  pagination?: TableProps<RecordType>['pagination'];
}

/**
 * 列表首屏骨架。
 *
 * 直接用 antd Table 当量具：表头底色、列宽、边框、字号、暗色 token 全部由页面自己的
 * columns 与组件主题供给，占位条只替换单元格 render——自画一份灰块会跟真实表格对不齐，
 * 尤其在有 fixed 列 / scroll.x / bordered 的页面上。
 */
export function TableSkeleton<RecordType extends Record<string, any>>({
  columns = [],
  rows,
  size,
  bordered,
  scrollX,
  pagination,
}: TableSkeletonProps<RecordType>) {
  const skeletonColumns = useMemo<TableColumnsType<RecordType>>(
    () =>
      columns
        .filter((column) => !column.hideInTable)
        .map((column, columnIndex) => ({
          ...(column as unknown as TableColumnsType<RecordType>[number]),
          key: String(column.key ?? column.dataIndex ?? columnIndex),
          render: (_dom: unknown, _record: unknown, rowIndex: number) => (
            <Skeleton.Button
              active
              size="small"
              style={{ width: BAR_WIDTHS[(rowIndex + columnIndex) % BAR_WIDTHS.length] }}
            />
          ),
        })),
    [columns],
  );

  const dataSource = useMemo(
    () =>
      Array.from({ length: rows }, (_, index) => ({
        [SKELETON_ROW_KEY]: index,
      }) as unknown as RecordType),
    [rows],
  );

  return (
    <Table<RecordType>
      rowKey={SKELETON_ROW_KEY}
      columns={skeletonColumns}
      dataSource={dataSource}
      size={size}
      bordered={bordered}
      scroll={scrollX === undefined ? undefined : { x: scrollX }}
      pagination={pagination}
    />
  );
}

export default TableSkeleton;
