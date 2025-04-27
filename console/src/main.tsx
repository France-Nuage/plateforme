import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { BrowserRouter, useLocation } from "react-router";
import { Routes } from "react-router";
import { Route } from "react-router";
import { ComponentRenderData, PageParamsProvider, PlasmicCanvasHost, PlasmicComponent, PlasmicRootProvider } from "@plasmicapp/loader-react";
import { PLASMIC } from "./plasmic-init.ts";
import { useSearchParams } from "react-router";

createRoot(document.getElementById("root")!).render(
  <PlasmicRootProvider loader={PLASMIC}>
    <StrictMode>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<PlasmicApp />} />
          <Route path="/plasmic-host" element={<PlasmicCanvasHost />} />
        </Routes>
      </BrowserRouter>
    </StrictMode>,
  </PlasmicRootProvider>
);

function PlasmicApp() {
  const [loading, setLoading] = useState(true);
  const [pageData, setPageData] = useState<ComponentRenderData | null>(null);
  const location = useLocation();
  const [searchParams] = useSearchParams();

  useEffect(() => {
    PLASMIC.maybeFetchComponentData(location.pathname).then((pageData) => {
      setPageData(pageData);
      setLoading(false);
    })
  });

  if (loading) {
    return <div>Loading...</div>;
  }
  if (!pageData) {
    return <div>Not found</div>;
  }

  return (
    <PageParamsProvider route={location.pathname} query={Object.fromEntries(searchParams)}>
      <PlasmicComponent component={location.pathname} />
    </PageParamsProvider>
  )
}
