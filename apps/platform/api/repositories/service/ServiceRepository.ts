import type { $Fetch } from "nitropack";
import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

interface PostServiceData {}

interface ServiceResource {
  id: string;
  name: string;
  updated_at: string;
  created_at: string;
}

type PatchServiceData = Partial<ServiceResource> | { resultCode: string };

export const ServiceRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<ServiceResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/services${apiCallParams}`);
    },
    get: async (
      serviceId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<ServiceResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/services/${serviceId}${apiCallParams}`);
    },
    post: async (
      body: PostServiceData,
    ): Promise<ApiResponse<ServiceResource>> => {
      return client(`/services`, { method: "POST", body: body });
    },
    patch: async (
      serviceId: string,
      body: PatchServiceData,
    ): Promise<ApiResponse<ServiceResource>> => {
      return client(`/services/${serviceId}`, { method: "PUT", body });
    },
    delete: async (body: Array<string>): Promise<ApiResponse<any>> => {
      return client(`/services`, { method: "DELETE", body });
    },
  };
};
