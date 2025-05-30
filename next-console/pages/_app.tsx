import { ApplicationLoader } from "@/components/core/ApplicationLoader";
import { AuthenticationGuard } from "@/components/core/AuthenticationGuard";
import { wrapper } from "@/store";
import "@/styles/globals.css";
import { PlasmicRootProvider } from "@plasmicapp/react-web";
import type { AppProps } from "next/app";
import Head from "next/head";
import Link from "next/link";
import { Provider } from "react-redux";
import { ToastContainer } from "react-toastify";

export default function MyApp({ Component, router, ...rest }: AppProps) {
  const { store, props } = wrapper.useWrappedStore(rest);
  return (
    <Provider store={store}>
      <ApplicationLoader>
        <PlasmicRootProvider Head={Head} Link={Link}>
          <AuthenticationGuard router={router}>
            <ToastContainer />
            <Component {...props.pageProps} />
          </AuthenticationGuard>
        </PlasmicRootProvider>
      </ApplicationLoader>
    </Provider>
  );
}
