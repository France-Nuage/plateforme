import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import Router from "./router.tsx";

import { projects } from '../plasmic.json';

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Router />
  </StrictMode>,
);
