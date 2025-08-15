import { FunctionComponent, ReactNode, useEffect } from 'react';

import {
  fetchAllDatacenters,
  fetchAllHypervisors,
  fetchAllInstances,
  fetchAllOrganizations,
  fetchAllProjects,
  fetchAllZeroTrustNetworkTypes,
  fetchAllZeroTrustNetworks,
  setApplicationLoaded,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';

export type DataProviderProps = {
  children: ReactNode;
};

/**
 * The data provider component.
 */
export const DataProvider: FunctionComponent<DataProviderProps> = ({
  children,
}) => {
  const dispatch = useAppDispatch();
  const application = useAppSelector((state) => state.application);

  // Load data on provider instantiation.
  useEffect(() => {
    dispatch(setApplicationLoaded(false));
    Promise.all([
      dispatch(fetchAllDatacenters()),
      dispatch(fetchAllHypervisors()),
      dispatch(fetchAllInstances()),
      dispatch(fetchAllOrganizations()),
      dispatch(fetchAllProjects()),
      dispatch(fetchAllZeroTrustNetworkTypes()),
      dispatch(fetchAllZeroTrustNetworks()),
    ]).then(() => {
      dispatch(setApplicationLoaded(true));
    });
  }, [application.mode, dispatch]);

  return application.loaded ? children : null;
};
