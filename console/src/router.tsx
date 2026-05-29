import { Suspense } from 'react';
import { HiDesktopComputer } from 'react-icons/hi';
import { HiCube, HiSquare3Stack3D } from 'react-icons/hi2';
import { BrowserRouter, Route, Routes } from 'react-router';

import { AppLayout, OrganizationGuard, PageGuard } from '@/components';
import {
  CreateInstancePage,
  HomePage,
  InstancesPage,
  LoginPage,
  ManagedInstanceDetailPage,
  ManagedInstancesPage,
  ManagedServiceDeployPage,
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
];

const Router = () => (
  <Suspense>
    <BrowserRouter>
      <Routes>
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
              <Route index element={<HomePage />} />
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
                path={RoutePath.ManagedServiceDeploy}
                element={<ManagedServiceDeployPage />}
              />
              <Route
                path={RoutePath.ManagedInstances}
                element={<ManagedInstancesPage />}
              />
              <Route
                path={RoutePath.ManagedInstanceDetail}
                element={<ManagedInstanceDetailPage />}
              />
            </Route>
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </Suspense>
);

export default Router;
