import {
  clearAuthenticationState,
  createInstance,
  fetchAllHypervisors,
  fetchAllInstances,
  registerHypervisor,
  setMode,
} from "@/features";
import { useAppDispatch, useAppSelector } from "@/hooks";
import { userManager } from "@/services";
import { DataProvider } from "@plasmicapp/react-web/lib/host";
import { forwardRef, ReactNode, useEffect, useImperativeHandle } from "react";

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
  ) => void;
  registerHypervisor: (
    authorizationToken: string,
    storageName: string,
    url: string,
  ) => void;
  signin: () => void;
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

    // Extract state subsets to expose to the plasmic app
    const application = useAppSelector((state) => state.application);
    const hypervisors = useAppSelector(
      (state) => state.hypervisors.hypervisors,
    );
    const instances = useAppSelector((state) => state.instances.instances);
    const user = useAppSelector((state) => state.authentication.user);

    // Expose actions to the plasmic app
    useImperativeHandle(ref, () => ({
      changeMode: () => dispatch(setMode()),
      createInstance: (maxCpuCores, maxMemoryBytes, name) =>
        dispatch(createInstance({ maxCpuCores, maxMemoryBytes, name })),
      registerHypervisor: (authorizationToken, storageName, url) =>
        dispatch(registerHypervisor({ authorizationToken, storageName, url })),
      signin: () => userManager.signinRedirect(),
      signout: async () => {
        await userManager.removeUser();
        dispatch(clearAuthenticationState());
      },
    }));

    // Load data on provider instantiation. This should be moved to the 
    // ApplicationLoader component.
    useEffect(() => {
      dispatch(fetchAllHypervisors());
      dispatch(fetchAllInstances());
    }, [application.mode, dispatch]);

    // Wrap the children in the plasmic DataProvider.
    return (
      <DataProvider
        name="France Nuage"
        data={{ application, hypervisors, instances, user }}
      >
        {children}
      </DataProvider>
    );
  },
);

ConsoleProvider.displayName = "France Nuage Console Provider";
