import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Queue, ServiceError } from '@/types';
import { postHttp } from '@/lib/fetch';
import { QueryKeys } from '../constants';
import type { SnakeCasedPropertiesDeep as Sn } from 'type-fest';

export interface CreateQueueRequest {
  name: string;
  priority: number;
  clusterTargets: string[];
}

async function createQueue(payload: CreateQueueRequest): Promise<Queue> {
  const resp = await postHttp<Sn<CreateQueueRequest>, Sn<Queue>>('/queues', {
    name: payload.name,
    priority: payload.priority,
    cluster_targets: payload.clusterTargets,
  });
  return {
    id: resp.id,
    name: resp.name,
    priority: resp.priority,
    clusterTargets: resp.cluster_targets,
  };
}

export interface UseCreateQueueProps {
  onSuccess?: (data: Queue) => void;
  onError?: (error: ServiceError) => void;
}

export function useCreateQueue(props: UseCreateQueueProps) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationKey: [QueryKeys.CREATE_QUEUE],
    mutationFn: createQueue,
    onSuccess: (data) => {
      queryClient.invalidateQueries({
        queryKey: [QueryKeys.LIST_QUEUES],
      });
      if (props.onSuccess !== undefined) {
        props.onSuccess(data);
      }
    },
    onError: props.onError,
  });
}
