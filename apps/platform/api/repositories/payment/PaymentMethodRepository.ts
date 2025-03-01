import type { $Fetch } from "ofetch";
import { parseUri } from "./../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

/**
 * @deprecated
 */
interface PostPaymentData {}

/**
 * @deprecated
 */
interface PaymentResource {
  id: string;
  name: string;
  updated_at: string;
  created_at: string;
}

/**
 * @deprecated
 */
type PatchPaymentData = Partial<PaymentResource> | { resultCode: string };

/**
 * @deprecated
 */
export const PaymentMethodRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<PaymentResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/payment-methods${apiCallParams}`);
    },
    get: async (
      paymentId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<PaymentResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/payment-methods/${paymentId}${apiCallParams}`);
    },
    post: async (
      body: PostPaymentData,
    ): Promise<ApiResponse<PaymentResource>> => {
      return client(`/payment-methods`, { method: "POST", body: body });
    },
    patch: async (
      paymentId: string,
      body: PatchPaymentData,
    ): Promise<ApiResponse<PaymentResource>> => {
      return client(`/payment-methods/${paymentId}`, { method: "PUT", body });
    },
    delete: async (body: Array<string>): Promise<ApiResponse<any>> => {
      return client(`/payment-methods`, { method: "DELETE", body });
    },
  };
};
