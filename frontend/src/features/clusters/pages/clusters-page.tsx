import CreateClusterModal from '../components/create-cluster-modal';
import { useDeleteCluster, useListClusters } from '@/services';
import {
  Container,
  ContainerAction,
  ContainerContent,
  ContainerDescription,
  ContainerHeader,
  ContainerTitle,
} from '@/components/ui/container';
import Breadcrumbs from '@/components/common/breadcrumbs';
import { toast } from '@/components/toast';
import { useState } from 'react';
import { Card, DataTable } from '@/components/common';
import { ClusterSummary } from '@/types';
import { ColumnDef, createColumnHelper } from '@tanstack/react-table';
import { generatePath, Link } from 'react-router-dom';
import { Routes } from '@/constants';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';

const DeleteClusterButton = ({ clusterId }: { clusterId: string }) => {
  const { mutate, isPending } = useDeleteCluster({
    onSuccess: () => {
      toast.success('Cluster deleted successfully');
    },
  });

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant='destructive' size='sm'>
          Delete
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm Deletion</DialogTitle>
        </DialogHeader>
        <p>
          Are you sure you want to delete this cluster? This action cannot be
          undone.
        </p>
        <DialogClose>
          <div className='mt-4 flex justify-end space-x-2'>
            <Button
              variant='outline'
              onClick={() => {
                // Close dialog
              }}
            >
              Cancel
            </Button>
            <Button
              variant='destructive'
              onClick={() => mutate({ clusterId })}
              disabled={isPending}
            >
              {isPending ? 'Deleting...' : 'Delete'}
            </Button>
          </div>
        </DialogClose>
      </DialogContent>
    </Dialog>
  );
};

const columnHelper = createColumnHelper<ClusterSummary>();
const CLUSTER_COLUMNS: ColumnDef<ClusterSummary>[] = [
  columnHelper.accessor('clusterName', {
    header: 'Name',
    cell: ({ cell, row }) => {
      return (
        <Link
          to={generatePath(Routes.CLUSTER_DETAILS, {
            clusterId: row.original.clusterId,
          })}
          className='text-blue-600 visited:text-purple-600 hover:underline'
        >
          {cell.renderValue()}
        </Link>
      );
    },
  }),
  columnHelper.accessor('totalNodes', {
    header: 'Total Nodes',
    cell: ({ cell }) => {
      return (
        <Badge color='gray' variant='secondary'>
          {cell.renderValue()}
        </Badge>
      );
    },
  }),
  columnHelper.accessor('busyNodes', {
    header: 'Busy Nodes',
    cell: ({ cell }) => {
      return (
        <Badge color='red' variant='secondary'>
          {cell.renderValue()}
        </Badge>
      );
    },
  }),
  columnHelper.accessor('totalRunningJobs', {
    header: 'Running Jobs',
    cell: ({ cell }) => {
      return (
        <Badge color='blue' variant='secondary'>
          {cell.renderValue()}
        </Badge>
      );
    },
  }),
  columnHelper.display({
    id: 'actions',
    header: 'Actions',
    cell: ({ row }) => {
      return <DeleteClusterButton clusterId={row.original.clusterId} />;
    },
  }),
] as Array<ColumnDef<ClusterSummary>>;

function ClustersPage() {
  const { data: clusters, isLoading } = useListClusters({
    onError: (error) =>
      toast.error('Failed to load clusters', {
        description: error.error,
      }),
  });
  const [isOpen, setOpen] = useState(false);

  return (
    <Container>
      <ContainerHeader>
        <div className='flex-1 shrink-0 grow-0 basis-full pb-4'>
          <Breadcrumbs
            breadcrumbs={[
              {
                content: 'Clusters',
                link: `/clusters`,
              },
            ]}
          />
        </div>
        <ContainerTitle>
          Clusters
          <ContainerDescription></ContainerDescription>
        </ContainerTitle>
        <ContainerAction>
          <CreateClusterModal isOpen={isOpen} setOpen={setOpen} />
        </ContainerAction>
      </ContainerHeader>

      <ContainerContent>
        <div className='flex flex-row space-x-4'>
          <Card
            className='w-full'
            content={
              <DataTable
                columns={CLUSTER_COLUMNS}
                data={clusters ?? []}
                isLoading={isLoading}
              />
            }
          />
        </div>
      </ContainerContent>
    </Container>
  );
}

export default ClustersPage;
