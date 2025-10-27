export interface TokenResponse {
  access_token: string;
  expires_in: number;
  refresh_expires_in: number;
  refresh_token: string;
  token_type: 'Bearer';
  'not-before-policy': 0;
  session_state: string;
  scope: string;
}
