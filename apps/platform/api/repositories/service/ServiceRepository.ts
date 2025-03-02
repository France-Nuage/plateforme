import type { $Fetch } from "ofetch";
import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

/**
 * @deprecated
 */
interface PostServiceData {}

/**
 * @deprecated
 */
interface ServiceResource {
  id: string;
  name: string;
  updated_at: string;
  created_at: string;
}

/**
 * @deprecated
 */
type PatchServiceData = Partial<ServiceResource> | { resultCode: string };

/**
 * @deprecated
 */
export const ServiceRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<ServiceResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/services${apiCallParams}`);
    },
    get: async (
      serviceId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<ServiceResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/services/${serviceId}${apiCallParams}`);
    },
    post: async (
      body: PostServiceData,
    ): Promise<ApiResponse<ServiceResource>> => {
      return client(`/api/v1/services`, { method: "POST", body: body });
    },
    patch: async (
      serviceId: string,
      body: PatchServiceData,
    ): Promise<ApiResponse<ServiceResource>> => {
      return client(`/api/v1/services/${serviceId}`, { method: "PUT", body });
    },
    delete: async (body: Array<string>): Promise<ApiResponse<any>> => {
      return client(`/api/v1/services`, { method: "DELETE", body });
    },
  };
};
