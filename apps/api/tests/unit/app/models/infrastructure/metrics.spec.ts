// tests/metrics.spec.ts
import { test } from '@japa/runner'
import supertest from 'supertest'
import axios from 'axios'
import Env from '#start/env'
import sinon from 'sinon'

// Ensure these are always strings
const APP_URL: string = Env.get('APP_URL') ?? 'http://127.0.0.1:3333'
const MIMIR_URL: string = Env.get('MIMIR_URL') ?? 'http://mimir:9009'

// Create a dedicated Axios instance to avoid type mismatches
const axiosInstance = axios.create()

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

    // Option 1: Remove if using mocks
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
    // Stub the Axios post method to simulate an error response (status 500)
    const stub = sinon.stub(axiosInstance, 'post').rejects({
      response: {
        status: 500,
        data: { error: 'Internal Server Error' },
      },
    })

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

    // Restore the original behavior of the Axios post method
    stub.restore()
  })
})
