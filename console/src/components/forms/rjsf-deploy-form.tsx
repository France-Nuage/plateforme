import { Box } from '@chakra-ui/react';
import Form from '@rjsf/chakra-ui';
import type { IChangeEvent } from '@rjsf/core';
import type { RJSFSchema, TemplatesType, UiSchema } from '@rjsf/utils';
import validator from '@rjsf/validator-ajv8';
import { FunctionComponent, useCallback, useMemo, useState } from 'react';

import { splitUserAndSecretValues } from './lib/split-secrets';
import { ChakraBaseInputTemplate, ChakraFieldTemplate } from './widgets';

const customTemplates: Partial<TemplatesType> = {
  BaseInputTemplate: ChakraBaseInputTemplate,
  FieldTemplate: ChakraFieldTemplate,
};

export type RjsfDeployFormProps = {
  /** JSON Schema for the configurable values of a managed service version. */
  schema: RJSFSchema;
  /**
   * Optional rjsf UI Schema published alongside the JSON Schema by the chart
   * CI. When absent, rjsf renders a best-effort form from the JSON Schema
   * alone.
   */
  uiSchema?: UiSchema;
  /** Current user values payload, serialized as JSON. */
  userValues: string;
  /** Current secret values payload, serialized as JSON. */
  secretValues: string;
  /**
   * Fires whenever the form changes. Receives the user/secret split already
   * applied: secret-flagged fields are extracted into `secretValues`, the rest
   * stays in `userValues`. Empty payloads are serialized as the empty string
   * to match the API contract.
   */
  onValuesChange: (next: { userValues: string; secretValues: string }) => void;
};

/**
 * Renders the deployment form of a managed service version using rjsf with
 * the Chakra v3 theme. Splits the resulting payload into user and secret
 * buckets based on `format: "password"` fields in the schema.
 *
 * The form has no submit button: the parent page owns the submit action and
 * triggers it from outside (the deploy page renders its own primary button).
 */
export const RjsfDeployForm: FunctionComponent<RjsfDeployFormProps> = ({
  schema,
  uiSchema,
  userValues,
  secretValues,
  onValuesChange,
}) => {
  const initialFormData = useMemo(
    () => mergeValueBags(userValues, secretValues),
    [userValues, secretValues],
  );
  const [formData, setFormData] =
    useState<Record<string, unknown>>(initialFormData);

  const handleChange = useCallback(
    (event: IChangeEvent) => {
      const next = (event.formData ?? {}) as Record<string, unknown>;
      setFormData(next);
      const { userValues: user, secretValues: secret } =
        splitUserAndSecretValues(schema, next);
      onValuesChange({
        secretValues: secret ? JSON.stringify(secret) : '',
        userValues: user ? JSON.stringify(user) : '',
      });
    },
    [schema, onValuesChange],
  );

  return (
    <Box>
      <Form
        schema={schema}
        uiSchema={uiSchema}
        formData={formData}
        validator={validator}
        onChange={handleChange}
        showErrorList={false}
        templates={customTemplates}
      >
        {/* No submit button: parent page controls the deploy action. */}
        <span />
      </Form>
    </Box>
  );
};

function mergeValueBags(
  userJson: string,
  secretJson: string,
): Record<string, unknown> {
  const user = safeParseObject(userJson);
  const secret = safeParseObject(secretJson);
  return deepMerge(user, secret);
}

function safeParseObject(payload: string): Record<string, unknown> {
  if (!payload) return {};
  try {
    const parsed = JSON.parse(payload);
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : {};
  } catch {
    return {};
  }
}

function deepMerge(
  a: Record<string, unknown>,
  b: Record<string, unknown>,
): Record<string, unknown> {
  const out: Record<string, unknown> = { ...a };
  for (const [key, value] of Object.entries(b)) {
    const current = out[key];
    if (
      current &&
      typeof current === 'object' &&
      !Array.isArray(current) &&
      value &&
      typeof value === 'object' &&
      !Array.isArray(value)
    ) {
      out[key] = deepMerge(
        current as Record<string, unknown>,
        value as Record<string, unknown>,
      );
    } else {
      out[key] = value;
    }
  }
  return out;
}
