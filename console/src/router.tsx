import { Suspense } from 'react';
import { HiDesktopComputer } from 'react-icons/hi';
import { BrowserRouter, Route, Routes } from 'react-router';
import { Navigate } from 'react-router';

import { AppLayout } from '@/components';
import { InstancesPage, OidcRedirectPage } from '@/pages';
import { Routes as RoutePath } from '@/types';

const links = [
  { Icon: HiDesktopComputer, label: 'Instances', to: RoutePath.Instances },
];

const Router = () => (
  <Suspense>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<AppLayout links={links} />}>
          <Route
            index
            element={<Navigate replace to={RoutePath.Instances} />}
          />
          <Route path={RoutePath.Instances} element={<InstancesPage />} />
        </Route>
        <Route path="/auth/redirect/:provider" element={<OidcRedirectPage />} />
      </Routes>
    </BrowserRouter>
  </Suspense>
);

export default Router;
