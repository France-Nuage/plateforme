import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider as ReduxProvider } from 'react-redux';

import { ChakraProvider } from './components';
import { UserProvider } from './providers/user-provider.tsx';
import Router from './router.tsx';
import { store } from './store.ts';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ChakraProvider>
      <ReduxProvider store={store}>
        <UserProvider>
          <Router />
        </UserProvider>
      </ReduxProvider>
    </ChakraProvider>
  </StrictMode>,
);
