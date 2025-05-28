import express from 'express';
import https from 'https';
import { Provider } from 'oidc-provider';
import selfsigned from 'selfsigned';

import config from './config';
import { createUser } from './storage';

const app = express();
app.use(express.json());

const oidc = new Provider(config.issuer, {
  clientBasedCORS: () => true,
  clients: [{
    client_id: config.clientId,
    client_secret: config.clientSecret,
    grant_types: ['authorization_code', 'refresh_token'],
    response_types: ['code'],
    redirect_uris: config.allowedRedirects,
    token_endpoint_auth_method: 'none', // allows requests without client_secret
  }],
  features: {
    devInteractions: { enabled: true } // Built-in login UI
  },
  scopes: ['openid', 'profile', 'email', 'offline_access'],
});

app.get('/health', async (req, res) => {
  res.status(200).send();
});

app.post('/api/users', async (req, res) => {
  const { username = 'rstraub', email = 'robin@straub.pro', name = 'Robin Straub' } = req.body;

  // Store user
  const user = {
    sub: username,
    email: email || `${username}@test.com`,
    name: name || username,
    email_verified: true
  };
  createUser(username, user);

  // Generate tokens like a real auth flow would
  const client = (await oidc.Client.find(config.clientId))!;

  const AccessToken = oidc.AccessToken;
  const accessToken = new AccessToken({
    accountId: username,
    client: client,
    grantId: `test_${Date.now()}`,
    scope: 'openid profile email',
    gty: 'authorization_code',
  });

  // Generate ID token
  const IdToken = oidc.IdToken;
  const idToken = new IdToken(user, {
    client,
  });

  const now = Math.floor(Date.now() / 1000);
  const expiresAt = now + 3600;

  // Create profile object (decoded ID token payload)
  const profile = {
    sub: username,
    aud: config.clientId,
    exp: expiresAt,
    iat: now,
    iss: config.issuer
  };

  res.json({
    id_token: await idToken.issue({ use: 'idtoken' }),
    session_state: null,
    access_token: await accessToken.save(),
    token_type: "Bearer",
    scope: "email openid profile",
    profile: profile,
    expires_at: expiresAt
  });
});

app.use('/', oidc.callback());

// Generate proper self-signed certificate
const pems = selfsigned.generate([
  { name: 'commonName', value: 'localhost' },
  { name: 'commonName', value: 'oidc' },
  { name: 'commonName', value: 'console' }
], { days: 365 });

const httpsOptions = {
  key: pems.private,
  cert: pems.cert
};

https.createServer(httpsOptions, app).listen(config.port);

