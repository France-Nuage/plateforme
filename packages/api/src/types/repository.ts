import { Fetch } from "ofetch";

export type Repository = (
  client: Fetch,
  config: Record<any, any>,
) => Record<string, Function>;
