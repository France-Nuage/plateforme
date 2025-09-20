import { SetStateAction, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router';

/**
 * React hook that extends useState behavior to sync with URL search parameters.
 * Uses React Router's useSearchParams to maintain state persistence in the URL.
 *
 * @param name - The URL parameter name to sync with
 * @param defaultValue - Initial value when parameter is not present
 * @returns Tuple of [value, setValue] matching React's useState API
 */
export function useUrlState(
  name: string,
  defaultValue: string,
): [string, React.Dispatch<SetStateAction<string>>] {
  const [searchParams, setSearchParams] = useSearchParams();
  const [modified, setModified] = useState(false);
  const [value, setValue] = useState(defaultValue);
  const searchParamValue = searchParams.get(name);
  useEffect(() => {
    // the value has been updated and the search param is out-of-sync, update the latter
    if (modified && value !== searchParamValue) {
      setSearchParams((prev) => {
        prev.set(name, value);
        return prev;
      });
    }

    // the search param is ahead of the value, update the former
    if (
      !modified &&
      !!searchParamValue &&
      value !== searchParamValue &&
      value === defaultValue
    ) {
      setValue(searchParamValue);
    }

    // mark the state as modified so the url param can be updated to none
    if (!modified && !!value) {
      setModified(true);
    }
  }, [defaultValue, modified, name, searchParamValue, setSearchParams, value]);

  return [value, setValue];
}
