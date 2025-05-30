/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { createUseScreenVariants } from '@plasmicapp/react-web';
import * as React from 'react';

export type ScreenValue = 'mobile';
export const ScreenContext = React.createContext<ScreenValue[] | undefined>(
  'PLEASE_RENDER_INSIDE_PROVIDER' as any,
);

export const useScreenVariants = createUseScreenVariants(true, {
  mobile: '(max-width:414px)',
});

export default ScreenContext;
/* prettier-ignore-end */
