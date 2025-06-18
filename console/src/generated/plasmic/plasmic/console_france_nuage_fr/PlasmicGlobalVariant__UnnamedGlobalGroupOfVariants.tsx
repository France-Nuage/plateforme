/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { createUseScreenVariants } from '@plasmicapp/react-web';
import * as React from 'react';

export type UnnamedGlobalGroupOfVariantsValue = 'unnamedVariant';
export const UnnamedGlobalGroupOfVariantsContext = React.createContext<
  UnnamedGlobalGroupOfVariantsValue | undefined
>('PLEASE_RENDER_INSIDE_PROVIDER' as any);
export function UnnamedGlobalGroupOfVariantsContextProvider(
  props: React.PropsWithChildren<{
    value: UnnamedGlobalGroupOfVariantsValue | undefined;
  }>,
) {
  return (
    <UnnamedGlobalGroupOfVariantsContext.Provider value={props.value}>
      {props.children}
    </UnnamedGlobalGroupOfVariantsContext.Provider>
  );
}

export function useUnnamedGlobalGroupOfVariants() {
  return React.useContext(UnnamedGlobalGroupOfVariantsContext);
}

export default UnnamedGlobalGroupOfVariantsContext;
/* prettier-ignore-end */
