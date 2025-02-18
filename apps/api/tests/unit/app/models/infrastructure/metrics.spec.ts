// tests/metrics.spec.ts
import { test } from '@japa/runner'
import supertest from 'supertest'
import axios, { AxiosInstance } from 'axios'
import Env from '#start/env'
import MockAdapter from 'axios-mock-adapter'

// Ensure these are always strings
const APP_URL: string = Env.get('APP_URL') ?? 'http://127.0.0.1:3333'
const MIMIR_URL: string = Env.get('MIMIR_URL') ?? 'http://mimir:9009'

// Create a dedicated Axios instance to avoid type mismatches
const axiosInstance: AxiosInstance = axios.create()
const mock = new MockAdapter(axiosInstance)

test.group('MetricsController', () => {
  /**
   * Integration test: send metrics to Mimir and verify via the query_range API
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

    // Send a POST request to /metrics
    const response = await supertest(APP_URL).post('/metrics').send(payload).expect(200)

    // Verify the controller's response
    assert.equal(response.body.message, 'Metrics received and pushed successfully')
    assert.deepEqual(response.body.receivedData, payload)

    // Wait a moment for Mimir to ingest the metric
    await new Promise((resolve) => setTimeout(resolve, 1000))

    // Query Mimir using the "instance" label
    const mimirQueryParams = {
      query: '{instance="test-host"}',
      start: '2024-02-18T00:00:00Z',
      end: '2024-02-18T01:00:00Z',
      step: '15s',
    }

    // Use axiosInstance instead of axios
    const mimirResponse = await axiosInstance.get(`${MIMIR_URL}/prometheus/api/v1/query_range`, {
      params: mimirQueryParams,
    })

    // Verify Mimir's response
    assert.equal(mimirResponse.status, 200, 'Mimir response should have a status of 200')
    assert.exists(mimirResponse.data.data, 'Mimir should return data')
    assert.isTrue(
      Array.isArray(mimirResponse.data.data.result) && mimirResponse.data.data.result.length > 0,
      'The response should contain at least one time series'
    )
  })

  /**
   * Validation test: check that an incomplete payload returns a validation error
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

    const response = await supertest(APP_URL).post('/metrics').send(payload).expect(422)
    assert.exists(response.body.errors, 'Expected validation errors')

    // Verify the response references the missing "hostname"
    assert.isTrue(
      response.body.errors.some((err: any) => err.field === 'hostname'),
      'The validation error should mention the "hostname" field'
    )
  })

  /**
   * Error handling test: simulate a failure when pushing metrics to Mimir
   */
  test('should throw an error when Mimir push fails', async ({ assert }) => {
    // Simulate an error response (status 500) for the Mimir push call
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
      await supertest(APP_URL).post('/metrics').send(payload)
    } catch (error: any) {
      errorThrown = true
      assert.match(
        error.message,
        /Failed to push metrics/,
        'Error message should mention the failure to push metrics to Mimir'
      )
    }
    assert.isTrue(errorThrown, 'An error should be thrown when Mimir returns an error')

    // Restore axios to its original behavior
    mock.reset()
    mock.restore()
  })
})
