import { CodeBlock, Textarea, createShikiAdapter } from '@chakra-ui/react';
import { FunctionComponent, useMemo } from 'react';
import { HighlighterGeneric } from 'shiki';
import { createHighlighter } from 'shiki';

import { useColorMode } from '@/hooks';

export type AppCodeEditorProps = {
  code: string;
  editable: boolean;
  lang: 'yaml';
  onChange: (value: string) => void;
};

export const AppCodeEditor: FunctionComponent<AppCodeEditorProps> = ({
  editable,
  code,
  lang,
  onChange,
}) => {
  const { colorMode } = useColorMode();

  const adapter = useMemo(
    () =>
      createShikiAdapter<
        HighlighterGeneric<AppCodeEditorProps['lang'], 'github-light'>
      >({
        async load() {
          return createHighlighter({
            langs: ['yaml'],
            themes: ['github-dark', 'github-light'],
          });
        },
        theme: {
          dark: 'github-dark',
          light: 'github-light',
        },
      }),
    [],
  );

  return editable ? (
    <Textarea
      autoresize
      onChange={(event) => onChange(event.target.value)}
      value={code}
    />
  ) : (
    <CodeBlock.AdapterProvider value={adapter}>
      <CodeBlock.Root
        code={code}
        language={lang}
        meta={{ colorScheme: colorMode }}
        width="full"
      >
        <CodeBlock.Content bg="bg">
          <CodeBlock.Code>
            <CodeBlock.CodeText />
          </CodeBlock.Code>
        </CodeBlock.Content>
      </CodeBlock.Root>
    </CodeBlock.AdapterProvider>
  );
};
