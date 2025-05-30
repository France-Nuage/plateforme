/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */

import * as React from "react";
import { createUseScreenVariants } from "@plasmicapp/react-web";

export type UnnamedGlobalGroupOfVariantsValue = "unnamedVariant";
export const UnnamedGlobalGroupOfVariantsContext = React.createContext<
  UnnamedGlobalGroupOfVariantsValue | undefined
>("PLEASE_RENDER_INSIDE_PROVIDER" as any);

export function useUnnamedGlobalGroupOfVariants() {
  return React.useContext(UnnamedGlobalGroupOfVariantsContext);
}

export default UnnamedGlobalGroupOfVariantsContext;
/* prettier-ignore-end */
