import { useState } from 'react';
import { Avatar, Card, Tabs, Descriptions, Tag, Form, Input, Select, Button, App, Spin, Progress, Typography} from 'antd';

const { Text } = Typography;
import { useTranslation } from 'react-i18next';
import { useQueryClient } from '@tanstack/react-query';
import type { identityservicev1_User as User } from '@/api/generated/admin/service/v1';
import {
  useGetUserProfile,
  useUpdateUserProfile,
  useChangePassword,
} from '@/api/hooks/user-profile';
import { useAuthStore } from '@/stores';
import ContentContainer from '@/layouts/components/PageContainer/ContentContainer';
import { getCharColor, getRandomColor } from '@/utils/color';
import { getGenderOptions } from '../constants';
import MfaManagement from './MfaManagement';
import MySessions from './MySessions';
import NotificationPreference from './NotificationPreference';
import { useMyTenantUsage } from '@/api/hooks/my-tenant-usage';

/** 格式化 wellKnownTimestamp */
function formatTimestamp(ts: any): string {
  if (!ts) return '-';
  let ms: number;
  if (typeof ts === 'object' && ts.seconds !== undefined) {
    ms = Number(ts.seconds) * 1000 + Math.floor((ts.nanos || 0) / 1e6);
  } else if (typeof ts === 'number') {
    ms = ts > 1e12 ? ts : ts * 1000;
  } else if (typeof ts === 'string') {
    ms = new Date(ts).getTime();
  } else {
    return '-';
  }
  const d = new Date(ms);
  if (isNaN(d.getTime())) return '-';
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

/**
 * 个人中心页面
 */
const UserProfile = () => {
  const { t } = useTranslation('profile');

  const { message } = App.useApp();
  const queryClient = useQueryClient();
  const setUserInfo = useAuthStore((s) => s.setUserInfo);

  // 获取当前用户信息
  const { data: userData, isLoading } = useGetUserProfile();
  const user = userData as User | undefined | null;

  // 租户自助用量：租户管理员可见本租户套餐用量与配额（平台用户不展示）
  const accessStore = useAuthStore((st) => st.accessToken);
  const isTenantUser = !!accessStore && (user?.tenantId ?? 0) > 0;
  const usageQuery = useMyTenantUsage(isTenantUser);

  // 编辑表单
  const [form] = Form.useForm();
  const [saving, setSaving] = useState(false);

  // 修改密码表单
  const [pwdForm] = Form.useForm();
  const [changingPwd, setChangingPwd] = useState(false);

  // 更新用户资料
  const updateMutation = useUpdateUserProfile({
    onSuccess: () => {
      message.success(t('saveSuccess'));
      queryClient.invalidateQueries({ queryKey: ['getMe'] });
      // 同步更新 auth store
      queryClient.fetchQuery({ queryKey: ['getMe'] }).then((info) => {
        if (info) setUserInfo(info as unknown as UserInfo);
      });
    },
    onError: (err: Error) => {
      message.error(err.message || t('saveFailed'));
    },
  });

  // 修改密码
  // 后端在改密成功后已吊销本人全部会话令牌（含当前会话），
  // 这里前端同步 forceLogout（不调已失效的 logout API），由路由守卫重定向到登录页。
  const changePwdMutation = useChangePassword({
    onSuccess: () => {
      message.success(t('passwordChangedReLogin'));
      pwdForm.resetFields();
      useAuthStore.getState().forceLogout();
    },
    onError: (err: Error) => {
      message.error(err.message || t('changePasswordFailed'));
    },
  });

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      if (!user?.id) return;
      setSaving(true);
      await updateMutation.mutateAsync({ id: user.id, values });
    } catch {
      // 表单校验失败
    } finally {
      setSaving(false);
    }
  };

  const handleChangePassword = async () => {
    try {
      const values = await pwdForm.validateFields();
      if (values.newPassword !== values.confirmPassword) {
        message.error(t('passwordMismatch'));
        return;
      }
      setChangingPwd(true);
      await changePwdMutation.mutateAsync({
        oldPassword: values.oldPassword,
        newPassword: values.newPassword,
      });
    } catch {
      // 表单校验失败
    } finally {
      setChangingPwd(false);
    }
  };

  // 首字母头像颜色
  const firstChar = user?.username?.slice(0, 1).toUpperCase() || '?';
  const avatarColor = getCharColor(firstChar);

  if (isLoading) {
    return (
      <ContentContainer heightMode="auto" padding="24px">
        <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />
      </ContentContainer>
    );
  }

  return (
    <ContentContainer heightMode="auto" padding="24px">
      <Card>
        <Tabs
          defaultActiveKey="basicInfo"
          items={[
            {
              key: 'basicInfo',
              label: t('tab.basicInfo'),
              children: (
                <div style={{ display: 'flex', gap: 32, flexWrap: 'wrap' }}>
                  {/* 头像区域 */}
                  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 16 }}>
                    <Avatar
                      src={user?.avatar || undefined}
                      size={120}
                      style={!user?.avatar ? { backgroundColor: avatarColor, fontSize: 60 } : {}}
                    >
                      {firstChar}
                    </Avatar>
                  </div>

                  {/* 编辑表单 */}
                  <div style={{ flex: 1, minWidth: 400 }}>
                    <Form
                      form={form}
                      layout="vertical"
                      initialValues={{
                        nickname: user?.nickname || '',
                        realname: user?.realname || '',
                        email: user?.email || '',
                        mobile: user?.mobile || '',
                        gender: user?.gender,
                        region: user?.region || '',
                        address: user?.address || '',
                        remark: user?.remark || '',
                        locale: user?.locale || '',
                      }}
                      style={{ maxWidth: 500 }}
                    >
                      <Form.Item label={t('username')}>
                        <span style={{ fontWeight: 500 }}>{user?.username}</span>
                      </Form.Item>

                      <Form.Item name="nickname" label={t('nickname')} rules={[{ required: true, message: t('nicknamePlaceholder') }]}>
                        <Input placeholder={t('nicknamePlaceholder')} />
                      </Form.Item>

                      <Form.Item name="realname" label={t('realname')}>
                        <Input placeholder={t('realnamePlaceholder')} />
                      </Form.Item>

                      <Form.Item name="email" label={t('email')}>
                        <Input placeholder={t('emailPlaceholder')} />
                      </Form.Item>

                      <Form.Item name="mobile" label={t('mobile')}>
                        <Input placeholder={t('mobilePlaceholder')} />
                      </Form.Item>

                      <Form.Item name="gender" label={t('gender')}>
                        <Select placeholder={t('genderPlaceholder')} allowClear options={getGenderOptions(t)} />
                      </Form.Item>

                      <Form.Item name="region" label={t('region')}>
                        <Input placeholder={t('regionPlaceholder')} />
                      </Form.Item>

                      <Form.Item name="locale" label={t('locale')}>
                        <Select
                          placeholder={t('localePlaceholder')}
                          allowClear
                          options={[
                            { value: 'zh-CN', label: '中文' },
                            { value: 'en-US', label: 'English' },
                          ]}
                        />
                      </Form.Item>

                      <Form.Item name="address" label={t('address')}>
                        <Input placeholder={t('addressPlaceholder')} />
                      </Form.Item>

                      <Form.Item name="remark" label={t('remark')}>
                        <Input.TextArea rows={3} placeholder={t('remarkPlaceholder')} />
                      </Form.Item>

                      {/* 只读信息 */}
                      <Descriptions column={1} size="small" style={{ marginBottom: 16 }}>
                        <Descriptions.Item label={t('roleNames')}>
                          {user?.roleNames?.map((role) => (
                            <Tag key={role} style={{ backgroundColor: getRandomColor(role), color: '#333', border: 'none' }}>
                              {role}
                            </Tag>
                          ))}
                        </Descriptions.Item>
                        <Descriptions.Item label={t('orgUnitNames')}>
                          {user?.orgUnitNames?.map((org) => (
                            <Tag key={org} style={{ backgroundColor: getRandomColor(org), color: '#333', border: 'none' }}>
                              {org}
                            </Tag>
                          ))}
                        </Descriptions.Item>
                        <Descriptions.Item label={t('positionNames')}>
                          {user?.positionNames?.map((pos) => (
                            <Tag key={pos} style={{ backgroundColor: getRandomColor(pos), color: '#333', border: 'none' }}>
                              {pos}
                            </Tag>
                          ))}
                        </Descriptions.Item>
                        <Descriptions.Item label={t('tenantName')}>{user?.tenantName || '-'}</Descriptions.Item>
                        <Descriptions.Item label={t('lastLoginAt')}>{formatTimestamp(user?.lastLoginAt)}</Descriptions.Item>
                        <Descriptions.Item label={t('lastLoginIp')}>{user?.lastLoginIp || '-'}</Descriptions.Item>
                        <Descriptions.Item label={t('createdAt')}>{formatTimestamp(user?.createdAt)}</Descriptions.Item>
                      </Descriptions>

                      <Form.Item>
                        <Button type="primary" loading={saving} onClick={handleSave}>
                          {t('save')}
                        </Button>
                      </Form.Item>
                    </Form>
                  </div>
                </div>
              ),
            },
            {
              key: 'security',
              label: t('tab.security'),
              children: (
                <div style={{ maxWidth: 460 }}>
                  <h3 style={{ marginBottom: 24 }}>{t('changePassword')}</h3>
                  <Form form={pwdForm} layout="vertical">
                    <Form.Item
                      name="oldPassword"
                      label={t('oldPassword')}
                      rules={[{ required: true, message: t('requiredOldPassword') }]}
                    >
                      <Input.Password placeholder={t('oldPasswordPlaceholder')} />
                    </Form.Item>

                    <Form.Item
                      name="newPassword"
                      label={t('newPassword')}
                      rules={[{ required: true, message: t('requiredNewPassword') }]}
                    >
                      <Input.Password placeholder={t('newPasswordPlaceholder')} />
                    </Form.Item>

                    <Form.Item
                      name="confirmPassword"
                      label={t('confirmPassword')}
                      rules={[{ required: true, message: t('requiredConfirmPassword') }]}
                    >
                      <Input.Password placeholder={t('confirmPasswordPlaceholder')} />
                    </Form.Item>

                      <Form.Item>
                        <Button type="primary" loading={changingPwd} onClick={handleChangePassword}>
                          {t('changePassword')}
                        </Button>
                      </Form.Item>
                    </Form>
                  <MfaManagement />
                </div>
              ),
            },
            {
              key: 'sessions',
              label: t('tab.sessions'),
              children: <MySessions />,
            },
            {
              key: 'notification',
              label: t('tab.notification'),
              children: <NotificationPreference />,
            },
            ...(isTenantUser
              ? [{
                  key: 'tenantUsage',
                  label: t('tab.tenantUsage'),
                  children: <TenantUsagePanel usageQuery={usageQuery} />,
                }]
              : []),
          ]}
        />
      </Card>
    </ContentContainer>
  );
};

/** 租户套餐用量面板：用户数/存储占用 vs 配额上限的进度条。 */
function TenantUsagePanel({ usageQuery }: {
  usageQuery: { data?: import('@/api/generated/admin/service/v1').identityservicev1_TenantUsage | undefined; isLoading: boolean };
}) {
  const { t } = useTranslation('profile');
  const usage = usageQuery.data;

  if (usageQuery.isLoading) {
    return <Spin style={{ display: 'block', margin: '48px auto' }} />;
  }
  if (!usage || !usage.quotas || usage.quotas.length === 0) {
    return <Text type="secondary">{t('tenantUsage.empty')}</Text>;
  }

  // 64-bit 计数经 protojson 是字符串（文档已知坑），必须 Number() 转换再运算
  const num = (v: number | string | undefined): number => Number(v ?? 0);
  const currentOf = (quotaType: string): number => {
    switch (quotaType) {
      case 'USER_LIMIT': return num(usage.userCount);
      case 'STORAGE': return Math.round(num(usage.storageUsedBytes) / 1024 / 1024);
      case 'API_CALL': return num(usage.apiCallCount);
      default: return 0;
    }
  };
  const unitOf = (quotaType: string): string =>
    quotaType === 'STORAGE' ? 'MB' : '';

  return (
    <div style={{ maxWidth: 560 }}>
      <p style={{ marginBottom: 16 }}>
        <Text type="secondary">
          {t('tenantUsage.plan')}
          {usage.planName ? <strong>{usage.planName}</strong> : <strong>{t('tenantUsage.noPlan')}</strong>}
        </Text>
      </p>
      {usage.quotas.map((q, idx) => {
        const quotaType = q.quotaType?.toString() ?? '';
        const limit = num(q.quotaValue);
        const current = currentOf(quotaType);
        const pct = limit > 0 ? Math.min(100, (current / limit) * 100) : 0;
        const unit = unitOf(quotaType);
        return (
          <div key={idx} style={{ marginBottom: 20 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
              <span>{t(`tenantUsage.quota.${quotaType}`, quotaType)}</span>
              <span>
                {current}{unit} / {limit}{unit}
                {pct >= 100 && (
                  <Text type="danger" style={{ marginLeft: 8 }}>{t('tenantUsage.reached')}</Text>
                )}
              </span>
            </div>
            <Progress
              percent={pct}
              size="small"
              status={pct >= 100 ? 'exception' : pct >= 80 ? 'active' : 'normal'}
            />
          </div>
        );
      })}
    </div>
  );
}

export default UserProfile;
