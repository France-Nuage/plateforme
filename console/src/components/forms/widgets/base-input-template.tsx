import { Field, Input } from '@chakra-ui/react';
import type {
  BaseInputTemplateProps,
  FormContextType,
  RJSFSchema,
  StrictRJSFSchema,
} from '@rjsf/utils';
import { ariaDescribedByIds, getInputProps } from '@rjsf/utils';
import { ChangeEvent, FocusEvent, FunctionComponent } from 'react';

/**
 * Chakra v3 BaseInputTemplate adapted to surface helperText (description),
 * propagate the UI placeholder, and enforce `type="password"` whenever the
 * JSON Schema declares `format: "password"`.
 *
 * The default rjsf-chakra-ui BaseInputTemplate only renders the label + input
 * which leaves users without any contextual guidance and silently downgrades
 * password fields to plain text inputs. This template fixes both gaps while
 * remaining compatible with the rest of the rjsf-chakra-ui theme.
 */
export function ChakraBaseInputTemplate<
  T = unknown,
  S extends StrictRJSFSchema = RJSFSchema,
  F extends FormContextType = FormContextType,
>(props: BaseInputTemplateProps<T, S, F>): ReturnType<FunctionComponent> {
  const {
    id,
    htmlName,
    type,
    value,
    label,
    hideLabel,
    schema,
    onChange,
    onChangeOverride,
    onBlur,
    onFocus,
    options,
    required,
    readonly,
    rawErrors,
    autofocus,
    placeholder,
    disabled,
    uiSchema,
  } = props;

  const inputProps = getInputProps<T, S, F>(schema, type, options);
  if (schema.format === 'password') {
    inputProps.type = 'password';
  }

  const description =
    (uiSchema?.['ui:description'] as string | undefined) ?? schema.description;
  const showLabel = !hideLabel && !!label;
  const hasError = Array.isArray(rawErrors) && rawErrors.length > 0;

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    const next = event.target.value;
    onChange(next === '' ? (options.emptyValue as T) : (next as unknown as T));
  };
  const handleBlur = (event: FocusEvent<HTMLInputElement>) =>
    onBlur(id, event.target.value);
  const handleFocus = (event: FocusEvent<HTMLInputElement>) =>
    onFocus(id, event.target.value);

  return (
    <Field.Root
      mb={1}
      disabled={disabled || readonly}
      required={required}
      readOnly={readonly}
      invalid={hasError}
    >
      {showLabel && (
        <Field.Label>
          {label}
          <Field.RequiredIndicator />
        </Field.Label>
      )}
      <Input
        id={id}
        name={htmlName || id}
        value={value || value === 0 ? String(value) : ''}
        onChange={onChangeOverride || handleChange}
        onBlur={handleBlur}
        onFocus={handleFocus}
        autoFocus={autofocus}
        placeholder={placeholder}
        {...inputProps}
        aria-describedby={ariaDescribedByIds(id, !!schema.examples)}
      />
      {description && <Field.HelperText>{description}</Field.HelperText>}
    </Field.Root>
  );
}
