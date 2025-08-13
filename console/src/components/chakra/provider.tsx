'use client';

import {
  ChakraProvider as BaseChakraProvider,
  defaultSystem,
} from '@chakra-ui/react';

import { ColorModeProvider, type ColorModeProviderProps } from './color-mode';

export const ChakraProvider = (props: ColorModeProviderProps) => (
  <BaseChakraProvider value={defaultSystem}>
    <ColorModeProvider {...props} />
  </BaseChakraProvider>
);
