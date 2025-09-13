'use client';

import {
  ChakraProvider as BaseChakraProvider,
  defaultSystem,
} from '@chakra-ui/react';
import type { ThemeProviderProps } from 'next-themes';

import { ColorModeProvider } from './color-mode';
import { Toaster } from './toaster';

export const ChakraProvider = (props: ThemeProviderProps) => (
  <BaseChakraProvider value={defaultSystem}>
    <Toaster />
    <ColorModeProvider {...props} />
  </BaseChakraProvider>
);
