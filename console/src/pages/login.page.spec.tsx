// @vitest-environment jsdom
import { ChakraProvider, defaultSystem } from '@chakra-ui/react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import config from '@/config';

import { LoginPage } from './login.page';

const assign = vi.fn();

beforeEach(() => {
  assign.mockReset();
  // Stub only the navigation boundary; the real `loginRedirect` still runs.
  Object.defineProperty(window, 'location', {
    configurable: true,
    value: { assign },
  });
});

afterEach(() => {
  cleanup();
});

/**
 * Renders {@link LoginPage} at `/login`, seeding the query string so the
 * component reads `auth_error` exactly as the control plane would have sent it.
 */
function renderLogin(entry: string): void {
  render(
    <ChakraProvider value={defaultSystem}>
      <MemoryRouter initialEntries={[entry]}>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
        </Routes>
      </MemoryRouter>
    </ChakraProvider>,
  );
}

/**
 * The French copy the login page must render for each rejection reason. This is
 * the contract under test, so it is spelled out here rather than imported from
 * the component (which would make the assertion circular).
 */
const REASON_MESSAGES: ReadonlyArray<readonly [string, string]> = [
  ['state', 'Votre session de connexion a expiré ou est invalide. Réessayez.'],
  ['nonce', 'Votre session de connexion a expiré ou est invalide. Réessayez.'],
  [
    'session',
    "Votre session n'a pas pu être établie côté serveur. Contactez le support si le problème persiste.",
  ],
  [
    'exchange',
    "L'échange avec le fournisseur d'identité a échoué. Réessayez dans un instant.",
  ],
  [
    'no_id_token',
    "Réponse incomplète du fournisseur d'identité. Réessayez dans un instant.",
  ],
  [
    'validation',
    "Le jeton d'identité reçu est invalide. Contactez le support si le problème persiste.",
  ],
];

describe('LoginPage auth_error handling', () => {
  it.each(REASON_MESSAGES)(
    'renders the message for reason "%s" and does not auto-redirect',
    (reason, message) => {
      renderLogin(`/login?auth_error=${reason}`);

      expect(screen.getByText(message)).not.toBeNull();
      // The whole point: an error must break the loop, never re-enter it.
      expect(assign).not.toHaveBeenCalled();
    },
  );

  it('falls back to a generic message for an unknown reason', () => {
    renderLogin('/login?auth_error=teapot');

    expect(
      screen.getByText(
        'La connexion a échoué. Réessayez, ou contactez le support si le problème persiste.',
      ),
    ).not.toBeNull();
    expect(assign).not.toHaveBeenCalled();
  });

  it('auto-redirects to /auth/login when there is no auth_error', () => {
    renderLogin('/login');

    expect(assign).toHaveBeenCalledWith(`${config.controlplane}/auth/login`);
  });

  it('re-initiates the login flow when the retry button is clicked', () => {
    renderLogin('/login?auth_error=state');
    expect(assign).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole('button', { name: 'Réessayer la connexion' }),
    );

    expect(assign).toHaveBeenCalledWith(`${config.controlplane}/auth/login`);
  });
});
