import type { RJSFSchema } from '@rjsf/utils';

/**
 * Walks a JSON Schema and returns the dot-path of every leaf field flagged as
 * a secret (`format: "password"`). Used to split a flat formData payload into
 * two buckets: regular user_values and secret_values (the latter persisted as
 * Kubernetes Secrets, never in Postgres).
 *
 * Only `properties` (object schemas) are traversed; arrays and primitive
 * leaves are inspected but not descended into beyond their declared type.
 */
export function collectSecretPaths(schema: RJSFSchema): string[] {
  const paths: string[] = [];

  function walk(node: RJSFSchema, prefix: string): void {
    if (node.format === 'password' && prefix.length > 0) {
      paths.push(prefix);
      return;
    }
    if (node.type === 'object' && node.properties) {
      for (const [key, child] of Object.entries(node.properties)) {
        const childSchema = child as RJSFSchema;
        const childPath = prefix.length > 0 ? `${prefix}.${key}` : key;
        walk(childSchema, childPath);
      }
    }
  }

  walk(schema, '');
  return paths;
}

type Bag = Record<string, unknown>;

/**
 * Splits a form payload into (user_values, secret_values) according to the
 * password-flagged paths in the schema. Mutation-free: returns new objects.
 *
 * Empty objects collapse to `undefined` so the caller can pass them straight
 * to the API without sending `"{}"` strings.
 */
export function splitUserAndSecretValues(
  schema: RJSFSchema,
  formData: Bag,
): { userValues: Bag | undefined; secretValues: Bag | undefined } {
  const secretPaths = collectSecretPaths(schema);
  if (secretPaths.length === 0) {
    return {
      secretValues: undefined,
      userValues: Object.keys(formData).length > 0 ? formData : undefined,
    };
  }

  const userValues = structuredClone(formData) as Bag;
  const secretValues: Bag = {};

  for (const path of secretPaths) {
    const segments = path.split('.');
    const leaf = segments.pop()!;
    let userCursor = userValues;
    let secretCursor = secretValues;
    let exists = true;

    for (const segment of segments) {
      const next = userCursor[segment];
      if (next === undefined || next === null || typeof next !== 'object') {
        exists = false;
        break;
      }
      userCursor = next as Bag;
      if (secretCursor[segment] === undefined) {
        secretCursor[segment] = {};
      }
      secretCursor = secretCursor[segment] as Bag;
    }

    if (!exists) continue;
    if (userCursor[leaf] === undefined || userCursor[leaf] === '') continue;

    secretCursor[leaf] = userCursor[leaf];
    delete userCursor[leaf];

    for (let i = segments.length - 1; i >= 0; i -= 1) {
      let cursor = userValues;
      for (let j = 0; j < i; j += 1) {
        cursor = cursor[segments[j]] as Bag;
      }
      const inner = cursor[segments[i]] as Bag;
      if (inner && Object.keys(inner).length === 0) {
        delete cursor[segments[i]];
      }
    }
  }

  return {
    secretValues:
      Object.keys(secretValues).length > 0 ? secretValues : undefined,
    userValues: Object.keys(userValues).length > 0 ? userValues : undefined,
  };
}
