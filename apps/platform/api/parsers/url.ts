/**
 * from a nested object to a flat array of key value pairs objects
 *
 * @return array|string
 * @param value
 * @param path
 */
function flattenWithObjectQueryParams(value, path = "") {
  if (typeof value === "object" && !Array.isArray(value)) {
    return Object.entries(value)
      .filter(([, value]) => !!value)
      .map(([key, value]) => {
        return flattenWithObjectQueryParams(
          value,
          path.length === 0 ? key : `${path}[${key}]`,
        );
      })
      .flat();
  }

  if (Array.isArray(value)) {
    return value
      .filter((subValue) => !!subValue)
      .map((subValue) => {
        return flattenWithObjectQueryParams(subValue, `${path}`);
      })
      .flat();
  }

  return [{ key: path, value }];
}

/**
 * get a string from an array of objects { key, value }
 * merge values with the same key bcs it's a query string
 */
function arrayOfObjectToString(array) {
  const keyValueObject = array.reduce((acc, { key, value }) => {
    if (!value) return acc;
    if (key in acc) {
      acc[key] += `,${value}`;
    } else {
      acc[key] = value;
    }
    return acc;
  }, {});
  return Object.entries(keyValueObject)
    .map(([key, value]) => `${key}=${value}`)
    .join("&");
}

export const parseUri = (params = {}) => {
  if (typeof params === "string") {
    return params;
  }

  // rename filters to filter
  params.filter = params.filter || params.filters || {};
  delete params.filters;

  return "?" + arrayOfObjectToString(flattenWithObjectQueryParams(params));
};
