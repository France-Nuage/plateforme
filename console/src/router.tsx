import { BrowserRouter, Route, Routes } from 'react-router';
import React, { Suspense } from 'react';
import plasmic from '../plasmic.json';
import { OidcRedirectPage, PlasmicHost } from './pages';

type PlasmicComponent = NonNullable<typeof plasmic['projects'][0]['components'][0]>;

const pages = plasmic.projects
  .reduce((pages: PlasmicComponent[], { components }) => [...pages, ...components.filter(({ componentType }) => componentType === "page")], [])
  .map(({ id, importSpec, path, }) => {
    console.log(`constructing path "./generated/plasmic/${importSpec.modulePath}"`);
    return ({
      Component: React.lazy(() => import(`./generated/plasmic/${importSpec.modulePath}`)),
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
        {pages.map(({ Component, id, path }) => (<Route key={id} path={path} element={<Component />} />))}
      </Routes>
    </BrowserRouter>
  </Suspense>
);

