// tests/metrics.spec.ts
import { test } from '@japa/runner'
import supertest from 'supertest'
import axios from 'axios'
import Env from '#start/env'
import MockAdapter from 'axios-mock-adapter'

/**
 * If these environment variables might be undefined,
 * provide fallback values or cast them as strings.
 */
const APP_URL = Env.get('APP_URL', 'http://127.0.0.1:3333')
const MIMIR_URL = Env.get('MIMIR_URL', 'http://mimir:9009')

test.group('MetricsController', () => {
  /**
   * 1) Integration test: send metrics to Mimir and verify via the query_range API.
   */
  test('should send metrics to Mimir and verify via query_range', async ({ assert }) => {
    // Valid test data
    const payload = {
      hostname: 'test-host',
      ip_address: '127.0.0.1',
      os: 'linux',
      os_version: 'Ubuntu 20.04',
      cpu_count: 4,
      total_memory: 8192,
      disk_space: 256,
    }

    // Ensure APP_URL is a string (TypeScript-safe check or cast)

    // 1. Send a POST request to the /metrics route
    const response = await supertest(APP_URL).post('/metrics').send(payload).expect(200)

    // 2. Verify the controller's response
    assert.equal(response.body.message, 'Metrics received and pushed successfully')
    assert.deepEqual(response.body.receivedData, payload)

    // 3. Wait a moment for Mimir to ingest the metric
    await new Promise((resolve) => setTimeout(resolve, 1000))

    // 4. Since the metric does not have a name, we query using the "instance" label
    const mimirQueryParams = {
      query: '{instance="test-host"}',
      start: '2024-02-18T00:00:00Z',
      end: '2024-02-18T01:00:00Z',
      step: '15s',
    }

    // Ensure MIMIR_URL is a string (TypeScript-safe check or cast)
    if (typeof MIMIR_URL !== 'string') {
      throw new Error('MIMIR_URL must be a string')
    }

    // 5. Send a GET request to Mimir's query_range API to verify the push
    const mimirResponse = await axios.get(`${MIMIR_URL}/prometheus/api/v1/query_range`, {
      params: mimirQueryParams,
    })

    // 6. Verify Mimir's response
    assert.equal(mimirResponse.status, 200)
    assert.exists(mimirResponse.data.data, 'Mimir should return data')
    assert.isTrue(
      Array.isArray(mimirResponse.data.data.result) && mimirResponse.data.data.result.length > 0,
      'The response should contain at least one time series'
    )
  })

  /**
   * 2) Validation test: check that an incomplete payload returns a validation error.
   */
  test('should return a validation error when required fields are missing', async ({ assert }) => {
    // Incomplete payload (missing the "hostname" field)
    const payload = {
      ip_address: '127.0.0.1',
      os: 'linux',
      os_version: 'Ubuntu 20.04',
      cpu_count: 4,
      total_memory: 8192,
      disk_space: 256,
    }

    // Ensure APP_URL is a string

    // Send a POST request with the incomplete payload
    const response = await supertest(APP_URL).post('/metrics').send(payload).expect(422)

    // Check that the response body has an "errors" array
    assert.exists(response.body.errors, 'Response should contain an "errors" array')

    // Optionally define a type for the errors to be more explicit
    // type ValidationError = { field: string; message: string }
    // const errors = response.body.errors as ValidationError[]

    // For demonstration, just do a basic check:
    assert.isTrue(
      Array.isArray(response.body.errors) &&
        response.body.errors.some((err: any) => err.field === 'hostname'),
      'The validation error should mention the "hostname" field'
    )
  })

  /**
   * 3) Error handling test: simulate a failure when pushing metrics to Mimir.
   */
  test('should throw an error when Mimir push fails', async ({ assert }) => {
    // Create a mock adapter to intercept axios calls to Mimir
    const mock = new MockAdapter(axios)

    // Simulate an error response (status 500) for the Mimir push call
    if (typeof MIMIR_URL !== 'string') {
      throw new Error('MIMIR_URL must be a string')
    }
    mock.onPost(`${MIMIR_URL}/api/v1/push`).reply(500, { error: 'Internal Server Error' })

    const payload = {
      hostname: 'test-host',
      ip_address: '127.0.0.1',
      os: 'linux',
      os_version: 'Ubuntu 20.04',
      cpu_count: 4,
      total_memory: 8192,
      disk_space: 256,
    }

    let errorThrown = false
    try {
      // Ensure APP_URL is a string

      await supertest(APP_URL).post('/metrics').send(payload)
    } catch (error: any) {
      errorThrown = true
      // Verify that the error message mentions the failure to push metrics to Mimir
      assert.match(
        error.message,
        /Failed to push metrics/,
        'The error message should mention the failure to push metrics to Mimir'
      )
    }
    assert.isTrue(errorThrown, 'An error should be thrown when Mimir returns an error')

    // Restore axios to its original behavior
    mock.restore()
  })
})
