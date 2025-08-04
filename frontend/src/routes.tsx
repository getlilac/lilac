import { createBrowserRouter, Navigate } from 'react-router-dom';
import { Routes } from './constants/routes';
import { MainLayout } from './components/layout';
import ProtectedRoute from './components/router/protected-route';
import LoginPage from './features/account/pages/login-page';
import SignUpPage from './features/account/pages/sign-up-page';
import ErrorBoundary from './error-boundary';
import AccountSettingsPage from './features/account/pages/account-settings-page';
import ClustersPage from './features/clusters/pages/clusters-page';
import { ApiKeysSettings } from './features/api-keys/pages/api-keys-settings';
import { QueuesPage } from './features/queues/pages/queues-page';
import ClusterDetailsPage from './features/clusters/pages/cluster-details-page';
import QueueDetailsPage from './features/queues/pages/queue-details-page';
import JobsPage from './features/jobs/pages/jobs-page';
import JobDetailsPage from './features/jobs/pages/job-details';
import NodeDetailsPage from './features/nodes/pages/node-details';

export const router = createBrowserRouter([
  {
    errorElement: <ErrorBoundary />,
    children: [
      {
        path: '/login',
        element: <LoginPage />,
      },
      {
        path: '/signup',
        element: <SignUpPage />,
      },
      {
        element: <ProtectedRoute />,
        children: [
          {
            element: <MainLayout />,
            children: [
              {
                index: true,
                element: <Navigate to={Routes.CLUSTERS} replace />,
              },
              {
                path: Routes.CLUSTERS,
                element: <ClustersPage />,
              },
              {
                path: Routes.CLUSTER_DETAILS,
                element: <ClusterDetailsPage />,
              },
              {
                path: Routes.NODE_DETAILS,
                element: <NodeDetailsPage />,
              },
              {
                path: Routes.QUEUES,
                element: <QueuesPage />,
              },
              {
                path: Routes.QUEUE_DETAILS,
                element: <QueueDetailsPage />,
              },
              {
                path: Routes.JOBS,
                element: <JobsPage />,
              },
              {
                path: Routes.JOB_DETAILS,
                element: <JobDetailsPage />,
              },
              {
                path: Routes.API_KEYS,
                element: <ApiKeysSettings />,
              },
              {
                path: Routes.ACCOUNT_SETTINGS,
                element: <AccountSettingsPage />,
              },
            ],
          },
        ],
      },
      {
        path: '*',
        element: <div>404 Not Found</div>,
      },
    ],
  },
]);
