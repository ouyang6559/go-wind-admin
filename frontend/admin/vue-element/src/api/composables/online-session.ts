import {
  useMutation,
  type UseMutationOptions,
} from "@tanstack/vue-query";
import type {
  online_sessionservicev1_ForceLogoutSessionRequest,
  online_sessionservicev1_ForceLogoutSessionResponse,
  online_sessionservicev1_ListOnlineSessionResponse,
} from "@/api/generated/admin/service/v1";
import { apiClient } from "@/api/client";
import { queryClient } from "@/plugins/vue-query";

// ==============================
// 在线会话（在线用户 + 强制下线）
// ==============================

const LIST_KEY = "listOnlineSessions";

export type OnlineSessionListParams = {
  keyword?: string;
  page?: number;
  pageSize?: number;
};

export async function fetchListOnlineSessions(params: OnlineSessionListParams) {
  return queryClient.fetchQuery({
    queryKey: [LIST_KEY, params],
    queryFn: () => apiClient.onlineSessionService.ListOnlineSession(params),
    staleTime: 0,
    retry: 0,
  });
}

export function useForceLogoutSession(
  options?: UseMutationOptions<
    online_sessionservicev1_ForceLogoutSessionResponse,
    Error,
    online_sessionservicev1_ForceLogoutSessionRequest
  >
) {
  return useMutation({
    mutationFn: (req: online_sessionservicev1_ForceLogoutSessionRequest) =>
      apiClient.onlineSessionService.ForceLogoutSession(req),
    ...options,
  });
}
