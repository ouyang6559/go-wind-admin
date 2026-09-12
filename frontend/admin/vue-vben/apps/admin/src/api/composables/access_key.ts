import { useMutation, type UseMutationOptions } from '@tanstack/vue-query';

import { apiClient } from '#/api/client';
import { makeUpdateMask, type PaginationQuery } from '#/transport/rest';
import type { access_keyservicev1_CreateAccessKeyResponse } from '#/api/generated/admin/service/v1';
import type { access_keyservicev1_AccessKey as AccessKey } from '#/api/generated/admin/service/v1';

export async function fetchListAccessKeys(query: PaginationQuery) {
  return apiClient.accessKeyService.List(query.toRawParams());
}

/** 创建凭证：响应含一次性明文 secret */
export function useCreateAccessKey(
  options?: UseMutationOptions<
    access_keyservicev1_CreateAccessKeyResponse,
    Error,
    Record<string, any>
  >,
) {
  return useMutation({
    mutationFn: (values) =>
      apiClient.accessKeyService.Create({ data: { ...values } as any }),
    ...options,
  });
}

export function useUpdateAccessKey(
  options?: UseMutationOptions<{}, Error, { id: number; values: Record<string, any> }>,
) {
  return useMutation({
    mutationFn: ({ id, values }: { id: number; values: Record<string, any> }) =>
      apiClient.accessKeyService.Update({
        id,
        data: { ...values } as any,
        updateMask: makeUpdateMask(Object.keys(values ?? {})),
      }),
    ...options,
  });
}

export function useDeleteAccessKey(
  options?: UseMutationOptions<{}, Error, { id: number }>,
) {
  return useMutation({
    mutationFn: (req: { id: number }) => apiClient.accessKeyService.Delete(req),
    ...options,
  });
}

export type { AccessKey };
