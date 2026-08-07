import { NuqsAdapter } from 'nuqs/adapters/react-router/v7';
import { Suspense } from 'react';
import { HiDesktopComputer } from 'react-icons/hi';
import { HiCube, HiServer, HiSquare3Stack3D } from 'react-icons/hi2';
import { BrowserRouter, Route, Routes } from 'react-router';

import {
  AdminGuard,
  AppLayout,
  OrganizationGuard,
  PageGuard,
} from '@/components';
import {
  CreateInstancePage,
  HomePage,
  InstancesPage,
  KubernetesClusterCreatePage,
  KubernetesClusterEditPage,
  KubernetesClustersPage,
  LoginPage,
  ManagedInstanceDetailPage,
  ManagedInstancesPage,
  ManagedServiceDetailPage,
  ManagedServicesPage,
  OidcRedirectPage,
} from '@/pages';
import { Routes as RoutePath } from '@/types';

const links = [
  {
    Icon: HiDesktopComputer,
    label: 'Instances de VM',
    to: RoutePath.Instances,
  },
  {
    Icon: HiCube,
    label: 'Services managés',
    to: RoutePath.ManagedServices,
  },
  {
    Icon: HiSquare3Stack3D,
    label: 'Mes instances',
    to: RoutePath.ManagedInstances,
  },
  {
    adminOnly: true,
    Icon: HiServer,
    label: 'Clusters Kubernetes',
    to: RoutePath.KubernetesClusters,
  },
];

const Router = () => (
  <Suspense>
    <BrowserRouter>
      <NuqsAdapter>
        <Routes>
          {/* Index redirect, resolved before any guard runs on "/" */}
          <Route index element={<HomePage />} />
          {/* Authentication routes */}
          <Route element={<PageGuard />}>
            <Route path={RoutePath.Login} element={<LoginPage />} />
            <Route
              path="/auth/redirect/:provider"
              element={<OidcRedirectPage />}
            />
          </Route>
          {/* Authenticated routes */}
          <Route element={<PageGuard authenticated />}>
            <Route element={<OrganizationGuard />}>
              <Route element={<AppLayout links={links} />}>
                <Route path={RoutePath.Instances} element={<InstancesPage />} />
                <Route
                  path={RoutePath.CreateInstance}
                  element={<CreateInstancePage />}
                />
                <Route
                  path={RoutePath.ManagedServices}
                  element={<ManagedServicesPage />}
                />
                <Route
                  path={RoutePath.ManagedServiceDetail}
                  element={<ManagedServiceDetailPage />}
                />
                <Route
                  path={RoutePath.ManagedInstances}
                  element={<ManagedInstancesPage />}
                />
                <Route
                  path={RoutePath.ManagedInstanceDetail}
                  element={<ManagedInstanceDetailPage />}
                />
                {/* Admin routes — server enforces authorization; AdminGuard is a UX hint */}
                <Route element={<AdminGuard />}>
                  <Route
                    path={RoutePath.KubernetesClusters}
                    element={<KubernetesClustersPage />}
                  />
                  <Route
                    path={RoutePath.KubernetesClusterCreate}
                    element={<KubernetesClusterCreatePage />}
                  />
                  <Route
                    path={RoutePath.KubernetesClusterEdit}
                    element={<KubernetesClusterEditPage />}
                  />
                </Route>
              </Route>
            </Route>
          </Route>
        </Routes>
      </NuqsAdapter>
    </BrowserRouter>
  </Suspense>
);

export default Router;
