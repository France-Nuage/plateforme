import { DataProvider } from '@plasmicapp/react-web/lib/host';
import { ReactNode, forwardRef, useImperativeHandle } from 'react';
import { useSearchParams } from 'react-router';

import {
  clearAuthenticationState,
  createInstance,
  registerHypervisor,
  setMode,
} from '@/features';
import { Organization, Project } from '@/generated/rpc/resources';
import {
  useActiveParamsReconciliation,
  useAppDispatch,
  useAppSelector,
} from '@/hooks';
import { userManager } from '@/services';

export type Props = {
  children: ReactNode;
  className?: string;
};

/**
 * Defines actions exposed to the plasmic studio.
 *
 * An action is meant to be used inside the plasmic studio builder UI. As such,
 * it should have:
 * - a meaningful name and meaningful parameters names
 * - primitive types as parameters (rather than objects)
 *
 * This allows for a better vizualisation of how to use the action in the
 * studio.
 */
export type Actions = {
  /**
   * Switch between `ServiceMode.Rpc` and `ServiceMode.Mock`.
   */
  changeMode: () => void;

  /**
   * Create a new instance with the given config.
   */
  createInstance: (
    maxCpuCores: number,
    maxMemoryBytes: number,
    name: string,
    projectId: string,
  ) => void;

  /**
   * Register a new hypervisor on the controlplane.
   */
  registerHypervisor: (
    authorizationToken: string,
    datacenterId: string,
    organizationId: string,
    storageName: string,
    url: string,
  ) => void;

  /**
   * Set the active organization.
   */
  setActiveOrganization: (organization: Organization) => void;

  /**
   * Set the active project.
   */
  setActiveProject: (project: Project) => void;

  /**
   * Redirect the user to the external login page.
   */
  signin: () => void;

  /**
   * Sign the user out.
   */
  signout: () => void;
};

/**
 * The console provider component.
 *
 * This provider component allows the plasmic studio to access specific parts of
 * the application state as well as handcrafted actions.
 *
 * @see https://docs.plasmic.app/learn/data-provider/
 */
export const ConsoleProvider = forwardRef<Actions, Props>(
  ({ children, className }, ref) => {
    const dispatch = useAppDispatch();
    const [, setSearchParams] = useSearchParams();

    // Extract state subsets to expose to the plasmic app
    const application = useAppSelector((state) => state.application);
    const datacenters = useAppSelector(
      (state) => state.infrastructure.datacenters,
    );
    const hypervisors = useAppSelector(
      (state) => state.hypervisors.hypervisors,
    );
    const instances = useAppSelector((state) => state.instances.instances);
    const organizations = useAppSelector(
      (state) => state.resources.organizations,
    );
    const projects = useAppSelector((state) => state.resources.projects);
    const user = useAppSelector((state) => state.authentication.user);
    const zeroTrustNetworkTypes = useAppSelector(
      (state) => state.infrastructure.zeroTrustNetworkTypes,
    );
    const zeroTrustNetworks = useAppSelector(
      (state) => state.infrastructure.zeroTrustNetworks,
    );

    // Expose actions to the plasmic app
    useImperativeHandle(ref, () => ({
      changeMode: () => dispatch(setMode()),
      createInstance: (maxCpuCores, maxMemoryBytes, name, projectId) =>
        dispatch(
          createInstance({ maxCpuCores, maxMemoryBytes, name, projectId }),
        ),
      registerHypervisor: (
        authorizationToken,
        datacenterId,
        organizationId,
        storageName,
        url,
      ) =>
        dispatch(
          registerHypervisor({
            authorizationToken,
            datacenterId,
            organizationId,
            storageName,
            url,
          }),
        ),
      setActiveOrganization: (organization: Organization) => {
        setSearchParams((previous) => ({
          ...Object.fromEntries(previous),
          organization: organization.id,
          project: projects.find(
            (project) => project.organizationId === organization.id,
          )!.id,
        }));
      },
      setActiveProject: (project: Project) => {
        setSearchParams((previous) => ({
          ...Object.fromEntries(previous),
          organization: project.organizationId,
          project: project.id,
        }));
      },
      signin: () => userManager.signinRedirect(),
      signout: async () => {
        await userManager.removeUser();
        dispatch(clearAuthenticationState());
      },
    }));

    useActiveParamsReconciliation();

    // Wrap the children in the plasmic DataProvider.
    return (
      <div className={className}>
        <DataProvider
          name="France Nuage"
          data={{
            application,
            datacenters,
            hypervisors,
            instances,
            organizations,
            projects,
            user,
            zeroTrustNetworks,
            zeroTrustNetworkTypes,
          }}
        >
          {children}
        </DataProvider>
      </div>
    );
  },
);

ConsoleProvider.displayName = 'France Nuage Console Provider';
