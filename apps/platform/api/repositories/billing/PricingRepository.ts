import { parseUri } from '../../parsers/url';
import type { AllowedParams } from './../ApiParams';
import type { ApiResponse } from './../ApiResponse';

interface PostOrganizationData {}

interface OrganizationResource {
  id: string;
  name: string;
  phone: string;
  fax: string;
  email: string;
  updated_at: string;
  created_at: string;
}

type PatchOrganizationData = Partial<OrganizationResource> | { resultCode: string };

export const PricingRepository = function (client: any, config: Record<any, any>) {
  return {
    list: async (params?: AllowedParams<any, null, null>): Promise<ApiResponse<OrganizationResource[]>> => {
      const apiCallParams = params ? parseUri(params) : '';
      return client(`/pricing${apiCallParams}`, { method: 'GET' });
    },
    get: async (
      pricingId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<OrganizationResource>> => {
      const apiCallParams = params ? parseUri(params) : '';
      return client(`/pricing/${pricingId}${apiCallParams}`);
    },
    post: async (body: PostOrganizationData): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/pricing`, {  method: 'POST', body });
    },
    patch: async (pricingId: string, body: PatchOrganizationData): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/pricing/${pricingId}`, {  method: 'PUT', body });
    },
  };
};