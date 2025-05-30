import { BrowserRouter, Route, Routes } from 'react-router';
import React, { Suspense } from 'react';
import { OidcRedirectPage, PlasmicHost } from '@/pages';
import { AuthenticationGuard } from '@/components';
import plasmic from '../plasmic.json';

type PlasmicComponent = NonNullable<typeof plasmic['projects'][0]['components'][0]>;

const componentModules = import.meta.glob('./generated/plasmic/*.tsx');

const pages = plasmic.projects
  .reduce((pages: PlasmicComponent[], { components }) => [...pages, ...components.filter(({ componentType }) => componentType === "page")], [])
  .map(({ id, importSpec, path, }) => {
    return ({
      Component: React.lazy(() => componentModules[`./generated/plasmic/${importSpec.modulePath}`]() as Promise<{ default: React.ComponentType }>),
      id,
      path,
    })
  });

export default () => (
  <Suspense>
    <BrowserRouter>
      <Routes>
        <Route path='/auth/redirect/:provider' element={<OidcRedirectPage />} />
        <Route path="/plasmic-host" element={<PlasmicHost />} />
        {pages.map(({ Component, id, path }) => (<Route key={id} path={path} element={<AuthenticationGuard><Component /></AuthenticationGuard>} />))}
      </Routes>
    </BrowserRouter>
  </Suspense>
);

