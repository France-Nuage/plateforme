import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import { BrowserRouter, Route, Routes } from "react-router";
import { PlasmicCanvasHost } from "@plasmicapp/loader-react";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/plasmic-host" element={<PlasmicCanvasHost />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>,
);
