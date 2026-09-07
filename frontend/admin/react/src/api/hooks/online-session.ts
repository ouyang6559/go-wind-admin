import { useMutation, useQueryClient, type UseMutationOptions, type UseQueryOptions } from '@tanstack/react-query';
import {
  type online_sessionservicev1_ForceLogoutSessionRequest,
  type online_sessionservicev1_ForceLogoutSessionResponse,
  type online_sessionservicev1_ListOnlineSessionResponse,
} from '@/api/generated/admin/service/v1';
import { queryClient } from '@/core';
import { apiClient } from '@/api/client';

// ==============================
// 在线会话（在线用户 + 强制下线）
// ==============================

const LIST_KEY = 'listOnlineSessions';

export type OnlineSessionListParams = {
  keyword?: string;
  page?: number;
  pageSize?: number;
};

export function useListOnlineSessions(
  params: OnlineSessionListParams,
  options?: UseQueryOptions<online_sessionservicev1_ListOnlineSessionResponse, Error>,
) {
  return useQuery({
    queryKey: [LIST_KEY, params],
    queryFn: () => apiClient.onlineSessionService.ListOnlineSession(params),
    ...options,
  });
}

export async function fetchListOnlineSessions(params: OnlineSessionListParams) {
  return queryClient.fetchQuery({
    queryKey: [LIST_KEY, params],
    queryFn: () => apiClient.onlineSessionService.ListOnlineSession(params),
    retry: 0,
  });
}

export function useForceLogoutSession(
  options?: UseMutationOptions<
    online_sessionservicev1_ForceLogoutSessionResponse,
    Error,
    online_sessionservicev1_ForceLogoutSessionRequest
  >,
) {
  const queryClientRef = useQueryClient();
  return useMutation({
    mutationFn: (req: online_sessionservicev1_ForceLogoutSessionRequest) =>
      apiClient.onlineSessionService.ForceLogoutSession(req),
    onSuccess: () => {
      // 被踢会话已从 Redis 删除，刷新在线列表
      queryClientRef.invalidateQueries({ queryKey: [LIST_KEY] });
    },
    ...options,
  });
}
