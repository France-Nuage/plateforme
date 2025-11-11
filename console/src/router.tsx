import { Suspense } from 'react';
import { HiDesktopComputer } from 'react-icons/hi';
import { BrowserRouter, Route, Routes } from 'react-router';

import { AppLayout, PageGuard } from '@/components';
import {
  CreateInstancePage,
  HomePage,
  InstancesPage,
  LoginPage,
  OidcRedirectPage,
} from '@/pages';
import { Routes as RoutePath } from '@/types';

const links = [
  {
    Icon: HiDesktopComputer,
    label: 'Instances de VM',
    to: RoutePath.Instances,
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
          <Route element={<AppLayout links={links} />}>
            <Route index element={<HomePage />} />
            <Route path={RoutePath.Instances} element={<InstancesPage />} />
            <Route
              path={RoutePath.CreateInstance}
              element={<CreateInstancePage />}
            />
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </Suspense>
);

export default Router;
