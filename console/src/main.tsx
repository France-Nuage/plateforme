import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider as ReduxProvider } from 'react-redux';

import { ApplicationLoader, ChakraProvider } from './components';
import Router from './router.tsx';
import { store } from './store.ts';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ChakraProvider>
      <ReduxProvider store={store}>
        <ApplicationLoader>
          <Router />
        </ApplicationLoader>
      </ReduxProvider>
    </ChakraProvider>
  </StrictMode>,
);
