import { apiClient } from "@/api/client";
import { PaginationQuery } from "@/core/transport/rest";

// ==============================
// 通知渠道（平台级配置）
// ==============================

/** 分页查询通知渠道 */
export async function fetchListNotificationChannels(query: PaginationQuery) {
  return apiClient.notificationChannelService.ListNotificationChannel(query.toRawParams());
}

/** 创建通知渠道：password 必填，服务端加密存储、不落日志 */
export async function createNotificationChannel(data: Record<string, any>, password: string) {
  return apiClient.notificationChannelService.CreateNotificationChannel({
    data: data as any,
    password,
  });
}

/**
 * 更新通知渠道：password 留空表示不修改已存密码；
 * updateMask 显式列举业务字段（与 react 基准一致），避免 password 等敏感字段混入。
 */
export async function updateNotificationChannel(
  id: number,
  values: Record<string, any>,
  password?: string,
) {
  return apiClient.notificationChannelService.UpdateNotificationChannel({
    id,
    data: values as any,
    password: password || undefined,
    updateMask: "name,type,smtpHost,smtpPort,smtpUsername,smtpFrom,smtpTls,enabled,remark",
  });
}

/** 删除通知渠道 */
export async function deleteNotificationChannel(id: number) {
  return apiClient.notificationChannelService.DeleteNotificationChannel({ id });
}

/** 向指定渠道发送测试邮件 */
export async function sendTestEmail(id: number, recipient: string) {
  return apiClient.notificationChannelService.SendTestEmail({ id, recipient });
}
