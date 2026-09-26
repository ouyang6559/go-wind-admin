import { apiClient } from "@/api/client";
import { PaginationQuery } from "@/core/transport/rest";

// ==============================
// AI 模型提供商（平台级配置）
// ==============================

/** 分页查询 AI 提供商 */
export async function fetchListAiProviders(query: PaginationQuery) {
  return apiClient.aiProviderService.List(query.toRawParams());
}

/**
 * 创建 AI 提供商。
 * apiKey 为请求级敏感字段：明文仅在本次请求内存在，后端加密落库、读取视图永不回显。
 */
export async function createAiProvider(values: Record<string, any>) {
  return apiClient.aiProviderService.Create({ data: values as any });
}

/**
 * 更新 AI 提供商：apiKey 留空表示不修改已存值（此时该字段不进 updateMask，
 * 避免空值经 FieldMask 被当成"显式清空"把已存的 Key 抹掉）。
 */
export async function updateAiProvider(id: number, values: Record<string, any>) {
  const payload: Record<string, any> = { ...values };
  const maskFields = [
    "name",
    "modelType",
    "modelName",
    "baseUrl",
    "organization",
    "localHost",
    "localPort",
    "timeoutSeconds",
    "systemPrompt",
    "isDefault",
    "isEnabled",
    "remark",
  ];
  if (payload.apiKey) {
    maskFields.push("apiKey");
  } else {
    delete payload.apiKey;
  }
  delete payload.apiKeyHint;
  return apiClient.aiProviderService.Update({
    id,
    data: payload as any,
    updateMask: maskFields.join(","),
  });
}

/** 删除 AI 提供商 */
export async function deleteAiProvider(id: number) {
  return apiClient.aiProviderService.Delete({ id });
}
