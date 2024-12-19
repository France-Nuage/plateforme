import { parseUri } from '../../parsers/url';
import type { AllowedParams } from './../ApiParams';
import type { ApiResponse } from './../ApiResponse';
import {useNavigationStore} from "~/stores/navigation";

interface PostUserData {}

interface UserResource {
    id: string;
    name: string;
    establishmentIdentifier: string;
    updatedAt: string;
    createdAt: string;
}

type PatchUserData = Partial<UserResource> | { resultCode: string };

export const MemberRepository = function (client: any, config: Record<any, any>) {
    return {
        list: async (params?: AllowedParams<any, null, null>): Promise<ApiResponse<any>> => {
            const apiCallParams = params ? parseUri(params) : '';
            return client(`/members${apiCallParams}`);
        },
        get: async (memberId: string, params?: AllowedParams<null, null, null>): Promise<ApiResponse<UserResource>> => {
            const apiCallParams = params ? parseUri(params) : '';
            return client(`/members/${memberId}${apiCallParams}`);
        },
        post: async (body: PostUserData): Promise<ApiResponse<UserResource>> => {
            return client(`/members`, { method: 'POST' , body});
        },
        patch: async (memberId: string, body: PatchUserData): Promise<ApiResponse<UserResource>> => {
            return client(`/members/${memberId}`, { method: 'PUT' , body });
        },
        delete: async (ids: Array<string>): Promise<Awaited<any>[]> => {
            return Promise.all(ids.map(async (id: string) => await client(`/members/${id}`, { method: 'DELETE' })))
        }
    };
};
