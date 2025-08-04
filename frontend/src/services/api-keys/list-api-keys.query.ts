import { queryOptions, useQuery } from '@tanstack/react-query';
import { QueryKeys } from '../constants';
import { ApiKey, ServiceError } from '@/types';
import { useEffect } from 'react';
import { getHttp } from '@/lib/fetch';
import type { SnakeCasedPropertiesDeep as Sn } from 'type-fest';

export async function listApiKeys(): Promise<ApiKey[]> {
  const resp = await getHttp<Sn<ApiKey[]>>('/account/api-keys');
  return resp.map((key) => ({
    id: key.id,
    prefix: key.prefix,
    createdAt: key.created_at,
    lastUsedAt: key.last_used_at,
    expiresAt: key.expires_at,
  }));
}

export function listApiKeysQuery(enabled: boolean = true) {
  return queryOptions({
    queryKey: [QueryKeys.LIST_API_KEYS],
    queryFn: () => listApiKeys(),
    enabled,
    staleTime: 1000 * 60 * 5,
  });
}

interface UseListApiKeysProps {
  enabled?: boolean;
  onSuccess?: (apiKeys: ApiKey[]) => void;
  onError?: (error: ServiceError) => void;
}

export function useListApiKeys(props?: UseListApiKeysProps) {
  const query = useQuery(listApiKeysQuery(props?.enabled));

  useEffect(() => {
    if (props?.onSuccess && query.data != undefined) {
      props.onSuccess(query.data);
    }
  }, [props, query.data]);

  useEffect(() => {
    if (props?.onError && query.error != null) {
      props.onError(query.error);
    }
  }, [props, query.error]);

  return query;
}
