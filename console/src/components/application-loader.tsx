import { FunctionComponent, ReactNode, useEffect, useState } from 'react';
import { toast } from 'react-toastify';

import {
  clearAuthenticationState,
  fetchAllDatacenters,
  fetchAllHypervisors,
  fetchAllInstances,
  fetchAllOrganizations,
  fetchAllProjects,
  fetchAllZeroTrustNetworkTypes,
  fetchAllZeroTrustNetworks,
  setOIDCUser,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { userManager } from '@/services';

export type ApplicationLoaderProps = {
  children: ReactNode;
};

/**
 * The application loaded component.
 *
 * This component is responsible for initializing the parts of the application
 * state that cannot be lazy-loaded, such as user authentication.
 *
 * On application load, it calls the userManager to attempt to retrieve the
 * persisted user, and dispatch an action depending on the user retrieval.
 * The `oidc-client-ts` library provides several security features which is why
 * we rely on it for authentication persistance, rather than a custom-brewed
 * redux persistance middleware.
 */
export const ApplicationLoader: FunctionComponent<ApplicationLoaderProps> = ({
  children,
}) => {
  const dispatch = useAppDispatch();
  const application = useAppSelector((state) => state.application);
  const [loading, setLoading] = useState<boolean>(true);

  // Attempt to retrieve the persisted user, then mark the app as loaded
  useEffect(() => {
    userManager
      .getUser()
      .then((user) => {
        if (user) {
          dispatch(setOIDCUser({ ...user }));
        } else {
          dispatch(clearAuthenticationState());
        }
        setLoading(false);
      })
      .catch(toast.error);
  }, [dispatch]);

  // Load data on provider instantiation. This should be moved to the
  // ApplicationLoader component.
  useEffect(() => {
    dispatch(fetchAllDatacenters());
    dispatch(fetchAllHypervisors());
    dispatch(fetchAllInstances());
    dispatch(fetchAllOrganizations());
    dispatch(fetchAllProjects());
    dispatch(fetchAllZeroTrustNetworkTypes());
    dispatch(fetchAllZeroTrustNetworks());
  }, [application.mode, dispatch]);

  return loading ? 'loading' : children;
};
