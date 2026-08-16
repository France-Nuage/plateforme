import { describe, expect, it } from 'vitest';

import resourcesSlice, { fetchAllOrganizations } from './resources.slice';

const reducer = resourcesSlice.reducer;

const request = 'req';

describe('resources organizations loading', () => {
  it('starts not loaded and without error', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.organizationsLoaded).toBe(false);
    expect(state.organizationsError).toBe(false);
  });

  it('marks organizations loaded and clears any error on success', () => {
    const state = reducer(
      undefined,
      fetchAllOrganizations.fulfilled([], request, undefined),
    );
    expect(state.organizationsLoaded).toBe(true);
    expect(state.organizationsError).toBe(false);
  });

  it('records an error when the fetch is rejected (so the guard leaves the spinner)', () => {
    const state = reducer(
      undefined,
      fetchAllOrganizations.rejected(
        new Error('unavailable'),
        request,
        undefined,
      ),
    );
    expect(state.organizationsError).toBe(true);
  });

  it('clears the error when a retry starts, so the guard shows the spinner again', () => {
    const failed = reducer(
      undefined,
      fetchAllOrganizations.rejected(
        new Error('unavailable'),
        request,
        undefined,
      ),
    );
    expect(failed.organizationsError).toBe(true);

    const retrying = reducer(
      failed,
      fetchAllOrganizations.pending(request, undefined),
    );
    expect(retrying.organizationsError).toBe(false);
  });

  it('clears the error when a retry succeeds', () => {
    const failed = reducer(
      undefined,
      fetchAllOrganizations.rejected(
        new Error('unavailable'),
        request,
        undefined,
      ),
    );
    const recovered = reducer(
      failed,
      fetchAllOrganizations.fulfilled([], request, undefined),
    );
    expect(recovered.organizationsLoaded).toBe(true);
    expect(recovered.organizationsError).toBe(false);
  });
});
