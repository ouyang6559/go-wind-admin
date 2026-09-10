import { useRef, useState, useEffect } from 'react';
import type { ProFormInstance } from '@ant-design/pro-components';
import {
  DrawerForm,
  ProFormText,
  ProFormDigit,
  ProFormTextArea,
  ProFormRadio,
} from '@ant-design/pro-components';
import { App, Tree, Spin } from 'antd';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { permissionservicev1_Role as Role } from '@/api/generated/admin/service/v1';
import { useCreateRole, useUpdateRole } from '@/api/hooks/role';
import { fetchListPermissionGroups } from '@/api/hooks/permission-group';
import { fetchListPermissions } from '@/api/hooks/permission';
import { PaginationQuery } from '@/core';
import { getStatusOptions, buildPermissionTree, extractLeafIds } from '../constants';

interface RoleDrawerProps {
  open: boolean;
  mode: 'create' | 'edit';
  data?: Role;
  onClose: () => void;
  onSuccess: () => void;
}

/**
 * 角色编辑/创建抽屉组件
 */
const RoleDrawer: React.FC<RoleDrawerProps> = ({ open, mode, data, onClose, onSuccess }) => {
  const { t } = useTranslation('role');
  const formRef = useRef<ProFormInstance>(null);
  const queryClient = useQueryClient();
  const { message } = App.useApp();

  const [confirmLoading, setConfirmLoading] = useState(false);
  const [treeData, setTreeData] = useState<any[]>([]);
  const [checkedKeys, setCheckedKeys] = useState<number[]>([]);
  const [treeVersion, setTreeVersion] = useState(0);
  const [treeLoading, setTreeLoading] = useState(false);

  // 加载权限树数据
  useEffect(() => {
    if (open) {
      setTreeLoading(true);
      Promise.all([
        fetchListPermissionGroups(new PaginationQuery({ formValues: { status: 'ON' } })),
        fetchListPermissions(new PaginationQuery({ formValues: { status: 'ON' } })),
      ])
        .then(([groupRes, permRes]) => {
          const groups = groupRes?.items || [];
          const permissions = permRes?.items || [];
          if (groups.length === 0 && permissions.length > 0) {
            // 无权限组时，直接罗列权限（平铺，不嵌套）
            setTreeData(permissions.map((p) => ({
              key: Number(p.id),
              title: p.name || p.code || String(p.id),
            })));
          } else {
            setTreeData(buildPermissionTree(groups, permissions));
          }
          setTreeVersion((v) => v + 1);
        })
        .catch(() => setTreeData([]))
        .finally(() => setTreeLoading(false));
    }
  }, [open]);

  // 编辑模式填充表单
  useEffect(() => {
    if (open && mode === 'edit' && data) {
      setTimeout(() => {
        formRef.current?.setFieldsValue({
          name: data.name || '',
          code: data.code || '',
          sortOrder: (data as any).sortOrder ?? 1,
          status: data.status || 'ON',
          description: (data as any).description || '',
        });
      }, 0);
      // 设置已勾选的权限
      const perms = (data as any).permissions;
      if (Array.isArray(perms)) {
        setCheckedKeys(perms.filter((v: any) => typeof v === 'number'));
      }
    }
  }, [open, mode, data]);

  const createMutation = useCreateRole({
    onSuccess: () => {
      message.success(t('createSuccess'));
      onSuccess();
      onClose();
      queryClient.invalidateQueries({ queryKey: ['listRoles'] });
    },
    onError: (error: Error) => {
      message.error(error.message || t('createFailed'));
    },
  });

  const updateMutation = useUpdateRole({
    onSuccess: () => {
      message.success(t('updateSuccess'));
      onSuccess();
      onClose();
      queryClient.invalidateQueries({ queryKey: ['listRoles'] });
    },
    onError: (error: Error) => {
      message.error(error.message || t('updateFailed'));
    },
  });

  const handleSubmit = async (values: any) => {
    setConfirmLoading(true);
    try {
      const payload = {
        ...values,
        // 用叶子交集剥离权限组父节点 ID（父子联动会自动勾选父节点，
        // 组 ID 混入提交会被后端当作权限 ID，可能越权绑定）。
        permissions: extractLeafIds(checkedKeys, treeData),
      };

      if (mode === 'create') {
        await createMutation.mutateAsync({ data: payload });
      } else if (data?.id) {
        await updateMutation.mutateAsync({ id: data.id, values: payload });
      }
    } finally {
      setConfirmLoading(false);
    }
  };

  return (
    <DrawerForm
      formRef={formRef}
      title={mode === 'create' ? t('create') : t('edit')}
      open={open}
      onOpenChange={(visible) => {
        if (!visible) {
          formRef.current?.resetFields();
          setCheckedKeys([]);
          onClose();
        }
      }}
      initialValues={{
        sortOrder: 1,
        status: 'ON',
      }}
      onFinish={handleSubmit}
      submitter={{
        searchConfig: {
          submitText: t('common:button.submit'),
          resetText: t('common:button.cancel'),
        },
        submitButtonProps: {
          loading: confirmLoading || createMutation.isPending || updateMutation.isPending,
        },
        resetButtonProps: { onClick: onClose },
      }}
      drawerProps={{ destroyOnHidden: true, onClose, size: 600 }}
    >
      <ProFormText
        name="name"
        label={t('name')}
        placeholder={t('namePlaceholder')}
        rules={[{ required: true, message: t('requiredName') }]}
        fieldProps={{ allowClear: true }}
      />

      <ProFormText
        name="code"
        label={t('code')}
        placeholder={t('codePlaceholder')}
        rules={[{ required: true, message: t('requiredCode') }]}
        fieldProps={{ allowClear: true }}
      />

      <ProFormDigit
        name="sortOrder"
        label={t('sortOrder')}
        placeholder={t('sortOrderPlaceholder')}
        rules={[{ required: true }]}
        fieldProps={{ precision: 0, min: 0 }}
      />

      <ProFormRadio.Group
        name="status"
        label={t('status')}
        rules={[{ required: true, message: t('requiredStatus') }]}
        options={getStatusOptions(t)}
        fieldProps={{ optionType: 'button', buttonStyle: 'solid' }}
      />

      <ProFormTextArea
        name="description"
        label={t('description')}
        placeholder={t('descriptionPlaceholder')}
        fieldProps={{ allowClear: true, rows: 2 }}
      />

      {/* 权限树 */}
      <div className="mb-6">
        <label className="block mb-2 text-sm font-medium text-[color:var(--ant-color-text)]">
          {t('permissions')}
        </label>
        <Spin spinning={treeLoading}>
          {treeData.length > 0 ? (
            <div className="rounded-lg border border-white/10 bg-white/5 p-2.5 dark:border-white/8 dark:bg-zinc-800/40">
              <Tree
                checkable
                checkedKeys={checkedKeys}
                onCheck={(checked) => {
                  setCheckedKeys(checked as number[]);
                }}
                treeData={treeData}
                defaultExpandAll
                key={treeVersion}
                className="max-h-80 overflow-auto"
              />
            </div>
          ) : (
            <div className="rounded-lg border border-dashed border-[color:var(--ant-color-border)] py-4 text-center text-sm text-[color:var(--ant-color-text-quaternary)]">
              {treeLoading ? '' : 'No permission data'}
            </div>
          )}
        </Spin>
      </div>
    </DrawerForm>
  );
};

export default RoleDrawer;
