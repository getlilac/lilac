import { useMutation, useQueryClient } from '@tanstack/react-query';
import { NewApiKey, ServiceError } from '@/types';
import { postHttp } from '@/lib/fetch';
import { QueryKeys } from '../constants';
import type { SnakeCasedPropertiesDeep as Sn } from 'type-fest';

async function createApiKey(): Promise<NewApiKey> {
  const resp = await postHttp<object, Sn<NewApiKey>>('/account/api-keys', {});
  return {
    id: resp.id,
    prefix: resp.prefix,
    createdAt: resp.created_at,
    key: resp.key,
  };
}

export interface UseCreateApiKeyProps {
  onSuccess?: (data: NewApiKey) => void;
  onError?: (error: ServiceError) => void;
}

export function useCreateApiKey(props: UseCreateApiKeyProps) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationKey: [QueryKeys.CREATE_API_KEY],
    mutationFn: () => createApiKey(),
    onSuccess: (data) => {
      queryClient.invalidateQueries({
        queryKey: [QueryKeys.LIST_API_KEYS],
      });
      if (props.onSuccess !== undefined) {
        props.onSuccess(data);
      }
    },
    onError: props.onError,
  });
}
