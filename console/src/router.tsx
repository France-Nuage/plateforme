import { Suspense } from 'react';
import { HiDesktopComputer } from 'react-icons/hi';
import { BrowserRouter, Route, Routes } from 'react-router';

import { AppLayout, PageGuard } from '@/components';
import { HomePage, InstancesPage, LoginPage, OidcRedirectPage } from '@/pages';
import { Routes as RoutePath } from '@/types';

const links = [
  { Icon: HiDesktopComputer, label: 'Instances', to: RoutePath.Instances },
];

const Router = () => (
  <Suspense>
    <BrowserRouter>
      <Routes>
        {/* Authentication routes */}
        <Route element={<PageGuard authenticated={false} />}>
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
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
  </Suspense>
);

export default Router;
