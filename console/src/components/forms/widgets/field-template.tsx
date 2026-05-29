import { Fieldset } from '@chakra-ui/react';
import type {
  FieldTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from '@rjsf/utils';
import { getTemplate, getUiOptions } from '@rjsf/utils';
import { FunctionComponent } from 'react';

/**
 * Chakra v3 FieldTemplate that defers description rendering to the
 * BaseInputTemplate (via Field.HelperText). The stock rjsf-chakra-ui template
 * renders the description as a `Fieldset.Legend` ABOVE the input, which
 * duplicates what our custom BaseInputTemplate already renders BELOW the
 * input.
 *
 * Keeping the helperText placement under the input matches the standard
 * Chakra Field UX and the rest of the console.
 */
export function ChakraFieldTemplate<
  T = unknown,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(props: FieldTemplateProps<T, S, F>): ReturnType<FunctionComponent> {
  const {
    id,
    children,
    classNames,
    style,
    disabled,
    displayLabel,
    hidden,
    label,
    onKeyRename,
    onKeyRenameBlur,
    onRemoveProperty,
    readonly,
    registry,
    required,
    rawErrors = [],
    errors,
    help,
    rawDescription,
    schema,
    uiSchema,
  } = props;
  const uiOptions = getUiOptions<T, S, F>(uiSchema);
  const WrapIfAdditionalTemplate = getTemplate<
    'WrapIfAdditionalTemplate',
    T,
    S,
    F
  >('WrapIfAdditionalTemplate', registry, uiOptions);

  if (hidden) {
    return <div style={{ display: 'none' }}>{children}</div>;
  }

  return (
    <WrapIfAdditionalTemplate
      classNames={classNames}
      style={style}
      disabled={disabled}
      id={id}
      label={label}
      displayLabel={displayLabel}
      rawDescription={rawDescription}
      onKeyRename={onKeyRename}
      onKeyRenameBlur={onKeyRenameBlur}
      onRemoveProperty={onRemoveProperty}
      readonly={readonly}
      required={required}
      schema={schema}
      uiSchema={uiSchema}
      registry={registry}
    >
      <Fieldset.Root
        disabled={disabled}
        invalid={rawErrors && rawErrors.length > 0}
      >
        {help}
        <Fieldset.Content>{children}</Fieldset.Content>
        {errors}
      </Fieldset.Root>
    </WrapIfAdditionalTemplate>
  );
}
