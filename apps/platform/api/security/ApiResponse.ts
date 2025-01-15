export interface SecurityResponse {
  config: Record<string, unknown>;
  data: {
    token_type: "Bearer";
    expires_in: number;
    access_token: string;
    refresh_token: string;
  };
  headers: Record<string, unknown>;
  request: Record<string, unknown>;
  status: number;
  statusText?: string;
}
