import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import Router from "./router.tsx";
import { Provider } from "react-redux";
import { store } from "./store.ts";
import { ApplicationLoader } from "./components/application-loader.tsx";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Provider store={store}>
      <ApplicationLoader>
        <Router />
      </ApplicationLoader>
    </Provider>
  </StrictMode>,
);
