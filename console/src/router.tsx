import { BrowserRouter, Route, Routes } from 'react-router';
import { PlasmicCanvasHost } from "@plasmicapp/loader-react";
import React, { Suspense } from 'react';
import App from './App';
import plasmic from '../plasmic.json';

type PlasmicComponent = NonNullable<typeof plasmic['projects'][0]['components'][0]>;

const pages = plasmic.projects
  .reduce((pages: PlasmicComponent[], { components }) => [...pages, ...components.filter(({ componentType }) => componentType === "page")], [])
  .map(({ id, importSpec, path, }) => ({
    element: React.lazy(() => import(`./generated/${importSpec.modulePath}`)),
    id,
    path,
  }));

export default () => (
  <Suspense>
    <BrowserRouter>
      <Routes>
        <Route path='/' element={<App />} />
        <Route path="/plasmic-host" element={<PlasmicCanvasHost />} />
        {pages.map(({ element, id, path }) => (<Route key={id} path={path} element={element} />))}
      </Routes>
    </BrowserRouter>
  </Suspense>
);

