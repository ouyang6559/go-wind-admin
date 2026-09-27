import { useMutation } from '@tanstack/react-query';
import { apiClient } from '@/api/client';
import { queryClient } from '@/core';

export interface AiUsageSummaryParams {
  lang?: string;
}

// useGetAiUsageSummary — 当月用量汇总（tokens/调用次数/配额上限）
export function useGetAiUsageSummary() {
  return useMutation({
    mutationFn: () => apiClient.aiUsageLogService.GetUsageSummary({}),
  });
}

import { type PaginationQuery } from '@/core';

// fetchListAiUsageLogs — ProTable request 回调用
export async function fetchListAiUsageLogs(params: PaginationQuery) {
  return queryClient.fetchQuery({
    queryKey: ['listAiUsageLogs', params],
    queryFn: () => apiClient.aiUsageLogService.List(params.toRawParams()),
    retry: 0,
  });
}
