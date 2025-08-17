'use client';

import {
  ChakraProvider as BaseChakraProvider,
  defaultSystem,
} from '@chakra-ui/react';
import type { ThemeProviderProps } from 'next-themes';

import { ColorModeProvider } from './color-mode';

export const ChakraProvider = (props: ThemeProviderProps) => (
  <BaseChakraProvider value={defaultSystem}>
    <ColorModeProvider {...props} />
  </BaseChakraProvider>
);
