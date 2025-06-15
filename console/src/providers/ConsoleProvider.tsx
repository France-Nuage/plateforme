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
  ({ children }, ref) => {
    const dispatch = useAppDispatch();
    const [, setSearchParams] = useSearchParams();

    // Extract state subsets to expose to the plasmic app
    const application = useAppSelector((state) => state.application);
    const hypervisors = useAppSelector(
      (state) => state.hypervisors.hypervisors,
    );
    const instances = useAppSelector((state) => state.instances.instances);
    const organizations = useAppSelector(
      (state) => state.resources.organizations,
    );
    const projects = useAppSelector((state) => state.resources.projects);
    const user = useAppSelector((state) => state.authentication.user);

    // Expose actions to the plasmic app
    useImperativeHandle(ref, () => ({
      changeMode: () => dispatch(setMode()),
      createInstance: (maxCpuCores, maxMemoryBytes, name, projectId) =>
        dispatch(
          createInstance({ maxCpuCores, maxMemoryBytes, name, projectId }),
        ),
      registerHypervisor: (
        authorizationToken,
        organizationId,
        storageName,
        url,
      ) =>
        dispatch(
          registerHypervisor({
            authorizationToken,
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
      <DataProvider
        name="France Nuage"
        data={{
          application,
          hypervisors,
          instances,
          organizations,
          projects,
          user,
        }}
      >
        {children}
      </DataProvider>
    );
  },
);

ConsoleProvider.displayName = 'France Nuage Console Provider';
