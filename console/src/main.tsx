import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { Provider } from 'react-redux';

import { ApplicationLoader } from './components/application-loader.tsx';
import Router from './router.tsx';
import { store } from './store.ts';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Provider store={store}>
      <ApplicationLoader>
        <Router />
      </ApplicationLoader>
    </Provider>
  </StrictMode>,
);
