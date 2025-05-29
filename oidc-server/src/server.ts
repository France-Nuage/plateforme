import express from 'express';
import { Provider } from 'oidc-provider';

import config from './config';
import { createUser } from './storage';

const app = express();
app.use(express.json());

// Middleware to force HTTPS headers
app.use((req, res, next) => {
  (req.socket as any).encrypted = true;
  next();
});

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
  // Override interaction URL generation
  interactions: {
    url(ctx, interaction) {
      console.log('in interactions', ctx, interaction);
      return `https://oidc/interaction/${interaction.uid}`;
    }
  },
  scopes: ['openid', 'profile', 'email', 'offline_access'],
});

app.get('/health', async (req, res) => {
  res.sendStatus(200);
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

app.listen(config.port, () => {
  console.log(`OIDC server running on port ${config.port}`);
});

