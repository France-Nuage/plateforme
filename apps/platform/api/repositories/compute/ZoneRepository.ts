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

export const ZoneRepository = function (client: any, config: Record<any, any>) {
  return {
    list: async (params?: AllowedParams<any, null, null>): Promise<ApiResponse<OrganizationResource[]>> => {
      const apiCallParams = params ? parseUri(params) : '';
      return client(`/zones${apiCallParams}`, { method: 'GET' });
    },
    get: async (
      zoneId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<OrganizationResource>> => {
      const apiCallParams = params ? parseUri(params) : '';
      return client(`/zones/${zoneId}${apiCallParams}`);
    },
    post: async (body: PostOrganizationData): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/zones`, {  method: 'POST', body });
    },
    patch: async (zoneId: string, body: PatchOrganizationData): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/zones/${zoneId}`, {  method: 'PUT', body });
    },
  };
};