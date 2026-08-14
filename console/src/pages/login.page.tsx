import { Alert, Button, Center, Spinner, Stack } from '@chakra-ui/react';
import { FunctionComponent, useEffect } from 'react';
import { useSearchParams } from 'react-router';

import { AuthErrorReason, loginRedirect } from '@/services';

/**
 * Human-readable French copy for each callback rejection reason. Total over the
 * union so a new reason cannot be added server-side without also being explained
 * here (the `Record` makes the exhaustiveness a compile error).
 */
const AUTH_ERROR_MESSAGES: Record<AuthErrorReason, string> = {
  exchange:
    "L'échange avec le fournisseur d'identité a échoué. Réessayez dans un instant.",
  no_id_token:
    "Réponse incomplète du fournisseur d'identité. Réessayez dans un instant.",
  nonce: 'Votre session de connexion a expiré ou est invalide. Réessayez.',
  session:
    "Votre session n'a pas pu être établie côté serveur. Contactez le support si le problème persiste.",
  state: 'Votre session de connexion a expiré ou est invalide. Réessayez.',
  validation:
    "Le jeton d'identité reçu est invalide. Contactez le support si le problème persiste.",
};

/**
 * Fallback message for an `auth_error` value that is not one of the known
 * reasons (e.g. a future server reason this build predates).
 */
const GENERIC_AUTH_ERROR_MESSAGE =
  'La connexion a échoué. Réessayez, ou contactez le support si le problème persiste.';

/**
 * Narrows an arbitrary query value to a known {@link AuthErrorReason}.
 */
function isAuthErrorReason(value: string): value is AuthErrorReason {
  return Object.prototype.hasOwnProperty.call(AUTH_ERROR_MESSAGES, value);
}

/**
 * Total lookup from a raw `auth_error` query value to its French message; an
 * unknown value maps to the generic message rather than being left unhandled.
 */
function authErrorMessage(reason: string): string {
  if (isAuthErrorReason(reason)) {
    return AUTH_ERROR_MESSAGES[reason];
  }
  return GENERIC_AUTH_ERROR_MESSAGE;
}

/**
 * Login page component.
 *
 * With no `auth_error` in the URL it immediately redirects the browser to the
 * control plane's `/auth/login`, which runs the confidential-client
 * authorization-code flow against the identity provider; a spinner covers the
 * brief moment before navigation.
 *
 * When the control plane sent the browser back with `?auth_error=<reason>`
 * (a failed `/auth/callback`), it instead shows the reason and a manual retry.
 * Auto-redirecting in that case would send the visitor straight back through the
 * same failing callback and loop, never surfacing the error — so the effect only
 * redirects when there is no error.
 */
export const LoginPage: FunctionComponent = () => {
  const [searchParams] = useSearchParams();
  const authError = searchParams.get('auth_error');

  useEffect(() => {
    if (authError === null) {
      loginRedirect();
    }
  }, [authError]);

  if (authError !== null) {
    return (
      <Center h="100vh">
        <Alert.Root status="error" maxW="480px">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>La connexion a échoué</Alert.Title>
            <Alert.Description>
              <Stack gap="3" align="flex-start">
                {authErrorMessage(authError)}
                <Button
                  variant="outline"
                  size="sm"
                  colorPalette="red"
                  onClick={loginRedirect}
                >
                  Réessayer la connexion
                </Button>
              </Stack>
            </Alert.Description>
          </Alert.Content>
        </Alert.Root>
      </Center>
    );
  }

  return (
    <Center h="100vh">
      <Spinner size="xl" color="blue.solid" />
    </Center>
  );
};
