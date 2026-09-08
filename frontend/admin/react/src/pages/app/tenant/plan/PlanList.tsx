import { useRef, useState } from 'react';
import type { ProColumns, ActionType } from '@ant-design/pro-components';
import { ProTable } from '@ant-design/pro-components';
import { Button, Popconfirm, Tag, App } from 'antd';
import { EditOutlined, DeleteOutlined, PlusOutlined } from '@ant-design/icons';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

import { PaginationQuery } from '@/core';
import { TABLE } from '@/config/constants';
import { useProTableScrollY } from '@/hooks/useProTableScrollY';
import { fetchListPlans, useDeletePlan } from '@/api/hooks/plan';
import {
  getPlanVersionMap,
  getPlanVersionOptions,
  getExpiryPolicyMap,
  getExpiryPolicyOptions,
} from './constants';
import PlanDrawer from './components/PlanDrawer';

interface PlanListProps {
  currentPlanId: number | null;
  onPlanSelect: (planId: number) => void;
}

/**
 * 套餐目录列表（左侧）
 */
const PlanList: React.FC<PlanListProps> = ({ currentPlanId, onPlanSelect }) => {
  const { t } = useTranslation('plan');
  const actionRef = useRef<ActionType>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const tableScrollY = useProTableScrollY(containerRef);
  const queryClient = useQueryClient();
  const { message } = App.useApp();

  const [drawerOpen, setDrawerOpen] = useState(false);
  const [drawerMode, setDrawerMode] = useState<'create' | 'edit'>('create');
  const [editingPlan, setEditingPlan] = useState<any>(null);

  const versionMap = getPlanVersionMap(t);
  const expiryPolicyMap = getExpiryPolicyMap(t);

  // 删除 mutation
  const deleteMutation = useDeletePlan({
    onSuccess: () => {
      message.success(t('deleteSuccess'));
      actionRef.current?.reload();
      queryClient.invalidateQueries({ queryKey: ['listPlans'] });
    },
    onError: (error: Error) => {
      message.error(error.message || t('deleteFailed'));
    },
  });

  // 列配置
  const columns: ProColumns<any>[] = [
    {
      title: t('name'),
      dataIndex: 'name',
      width: 150,
      fixed: 'left',
    },
    {
      title: t('versionLabel'),
      dataIndex: 'version',
      width: 95,
      valueType: 'select',
      fieldProps: {
        options: getPlanVersionOptions(t),
      },
      render: (_, record) => {
        const item = (versionMap as Record<string, { text: string; color: string }>)[record.version as string];
        return item ? <Tag color={item.color}>{item.text}</Tag> : '-';
      },
    },
    {
      title: t('expiryPolicyLabel'),
      dataIndex: 'expiryPolicy',
      width: 95,
      valueType: 'select',
      fieldProps: {
        options: getExpiryPolicyOptions(t),
      },
      render: (_, record) => {
        const item = (expiryPolicyMap as Record<string, { text: string; color: string }>)[record.expiryPolicy as string];
        return item ? <Tag color={item.color}>{item.text}</Tag> : '-';
      },
    },
    {
      title: t('dataRetentionDays'),
      dataIndex: 'dataRetentionDays',
      width: 110,
      hideInSearch: true,
    },
    {
      title: t('description'),
      dataIndex: 'description',
      width: 150,
      hideInSearch: true,
      ellipsis: true,
    },
    {
      title: t('remark'),
      dataIndex: 'remark',
      width: 150,
      hideInSearch: true,
      ellipsis: true,
    },
    {
      title: t('createdAt'),
      dataIndex: 'createdAt',
      valueType: 'dateTime',
      width: 150,
      hideInSearch: true,
      sorter: true,
    },
    {
      title: t('action'),
      valueType: 'option',
      width: 90,
      render: (_, record) => [
        <a
          key="edit"
          onClick={(e) => {
            e.stopPropagation();
            setEditingPlan(record);
            setDrawerMode('edit');
            setDrawerOpen(true);
          }}
        >
          <EditOutlined />
        </a>,
        <Popconfirm
          key="delete"
          title={t('deleteConfirmTitle')}
          description={t('deleteConfirmDesc', { moduleName: t('moduleName') })}
          onConfirm={(e) => {
            e?.stopPropagation();
            record.id && deleteMutation.mutate({ id: record.id });
          }}
          onCancel={(e) => e?.stopPropagation()}
          okText={t('common:button.ok')}
          cancelText={t('common:button.cancel')}
        >
          <a style={{ color: 'var(--ant-color-error)' }} onClick={(e) => e.stopPropagation()}>
            <DeleteOutlined />
          </a>
        </Popconfirm>,
      ],
    },
  ];

  return (
    <>
      <div ref={containerRef} className="page-container-content" style={{ padding: '0 8px', height: '100%' }}>
        <ProTable<any>
          actionRef={actionRef}
          columns={columns}
          headerTitle={t('planList')}
          request={async (params, sorter) => {
            try {
              const orderBy: string[] = [];
              if (sorter && Object.keys(sorter).length > 0) {
                for (const key in sorter) {
                  orderBy.push((sorter[key] === 'ascend' ? '' : '-') + key);
                }
              }
              const query = new PaginationQuery({
                paging: {
                  page: params.current || 1,
                  pageSize: params.pageSize || TABLE.DEFAULT_PAGE_SIZE,
                },
                formValues: Object.fromEntries(
                  Object.entries(params).filter(
                    ([key]) => !['current', 'pageSize'].includes(key),
                  ),
                ),
                orderBy,
              });

              const response = await fetchListPlans(query);

              return {
                data: response.items || [],
                total: response.total || 0,
                success: true,
              };
            } catch (error: any) {
              message.error(error.message || t('fetchFailed'));
              return { data: [], total: 0, success: false };
            }
          }}
          rowKey="id"
          search={{
            labelWidth: 'auto',
            defaultCollapsed: false,
            span: 24,
          }}
          pagination={{
            defaultPageSize: TABLE.DEFAULT_PAGE_SIZE,
            showSizeChanger: true,
            showQuickJumper: true,
          }}
          toolBarRender={() => [
            <Button
              key="create"
              type="primary"
              icon={<PlusOutlined />}
              size="small"
              onClick={() => {
                setEditingPlan(null);
                setDrawerMode('create');
                setDrawerOpen(true);
              }}
            >
              {t('create')}
            </Button>,
          ]}
          options={{
            density: false,
            fullScreen: false,
            setting: false,
            reload: true,
          }}
          size="small"
          bordered
          cardBordered={false}
          scroll={{ y: tableScrollY }}
          onRow={(record) => ({
            onClick: () => {
              onPlanSelect(record.id);
            },
            style: {
              cursor: 'pointer',
              outline: record.id === currentPlanId ? '2px solid var(--ant-color-primary)' : undefined,
              outlineOffset: '-2px',
            },
          })}
        />
      </div>

      <PlanDrawer
        open={drawerOpen}
        mode={drawerMode}
        data={editingPlan}
        onClose={() => {
          setDrawerOpen(false);
          setEditingPlan(null);
        }}
        onSuccess={() => {
          actionRef.current?.reload();
        }}
      />
    </>
  );
};

export default PlanList;
