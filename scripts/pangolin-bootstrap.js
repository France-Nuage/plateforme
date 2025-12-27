#!/usr/bin/env node
/**
 * Pangolin Bootstrap Script
 *
 * This script automates the initial setup of Pangolin for CI testing.
 * It uses direct API calls to:
 * 1. Complete the initial setup with the setup token
 * 2. Login and obtain session
 * 3. Create a test organization
 * 4. Create a local site
 * 5. Generate an API key with all permissions
 *
 * Usage: node scripts/pangolin-bootstrap.js
 *
 * Environment variables:
 * - PANGOLIN_SETUP_TOKEN: The setup token from Pangolin logs (required)
 * - PANGOLIN_DASHBOARD_URL: The Pangolin dashboard URL (default: http://localhost:3002)
 *
 * Outputs:
 * - PANGOLIN_API_URL, PANGOLIN_API_KEY, PANGOLIN_ORG_ID to stdout for CI consumption
 */

const PANGOLIN_DASHBOARD_URL = process.env.PANGOLIN_DASHBOARD_URL || 'http://localhost:3002';
const PANGOLIN_SETUP_TOKEN = process.env.PANGOLIN_SETUP_TOKEN;

const TEST_ADMIN_EMAIL = 'admin@integration-test.local';
const TEST_ADMIN_PASSWORD = 'IntegrationTest123!';
const TEST_ORG_ID = 'integration-test-org';
const TEST_ORG_NAME = 'Integration Test Org';
const TEST_SITE_NAME = 'integration-test-site';

// Session cookies storage
let sessionCookies = '';

/**
 * Make an HTTP request to the Pangolin API
 */
async function apiRequest(method, path, body = null, options = {}) {
    const url = `${PANGOLIN_DASHBOARD_URL}/api/v1${path}`;
    const headers = {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
    };

    if (sessionCookies) {
        headers['Cookie'] = sessionCookies;
    }

    if (options.headers) {
        Object.assign(headers, options.headers);
    }

    const fetchOptions = {
        method,
        headers,
        redirect: 'manual',
    };

    if (body) {
        fetchOptions.body = JSON.stringify(body);
    }

    const response = await fetch(url, fetchOptions);

    // Capture Set-Cookie headers for session management
    const setCookie = response.headers.get('set-cookie');
    if (setCookie) {
        // Parse and store cookies
        const cookies = setCookie.split(',').map(c => c.split(';')[0].trim()).join('; ');
        if (cookies) {
            sessionCookies = cookies;
        }
    }

    return response;
}

/**
 * Wait for Pangolin to be ready
 */
async function waitForPangolin(maxAttempts = 60) {
    for (let i = 0; i < maxAttempts; i++) {
        try {
            const response = await fetch(`${PANGOLIN_DASHBOARD_URL}/api/v1/auth/initial-setup-complete`, {
                method: 'GET',
                headers: { 'Accept': 'application/json' },
            });
            if (response.ok) {
                console.error('Pangolin API is ready');
                return true;
            }
        } catch (e) {
            console.error(`Waiting for Pangolin API... (${i + 1}/${maxAttempts})`);
        }
        await new Promise(r => setTimeout(r, 2000));
    }
    throw new Error('Pangolin API did not become ready in time');
}

/**
 * Check if initial setup is already complete
 */
async function isSetupComplete() {
    const response = await apiRequest('GET', '/auth/initial-setup-complete');
    if (!response.ok) {
        throw new Error(`Failed to check setup status: ${response.status}`);
    }
    const data = await response.json();
    return data.data === true;
}

/**
 * Complete the initial server admin setup
 */
async function completeInitialSetup() {
    console.error('Starting initial setup...');

    const setupComplete = await isSetupComplete();
    if (setupComplete) {
        console.error('Initial setup already complete, skipping');
        return;
    }

    const response = await apiRequest('PUT', '/auth/set-server-admin', {
        email: TEST_ADMIN_EMAIL,
        password: TEST_ADMIN_PASSWORD,
        setupToken: PANGOLIN_SETUP_TOKEN,
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to complete initial setup: ${response.status} - ${text}`);
    }

    console.error('Initial setup completed');
}

/**
 * Login with admin credentials
 */
async function login() {
    console.error('Logging in...');

    const response = await apiRequest('POST', '/auth/login', {
        email: TEST_ADMIN_EMAIL,
        password: TEST_ADMIN_PASSWORD,
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to login: ${response.status} - ${text}`);
    }

    // Check for 2FA or other requirements
    const data = await response.json();
    if (data.data?.codeRequested || data.data?.twoFactorSetupRequired) {
        throw new Error('Two-factor authentication required - not supported in bootstrap');
    }

    console.error('Logged in successfully');
}

/**
 * Get organization creation defaults (subnets)
 */
async function getOrgDefaults() {
    console.error('Getting organization defaults...');

    const response = await apiRequest('GET', '/pick-org-defaults');

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to get org defaults: ${response.status} - ${text}`);
    }

    const data = await response.json();
    return data.data;
}

/**
 * Check if organization already exists
 */
async function orgExists(orgId) {
    const response = await apiRequest('GET', `/org/${orgId}`);
    return response.ok;
}

/**
 * Create the test organization
 */
async function createOrganization() {
    console.error('Creating organization...');

    // Check if org already exists
    if (await orgExists(TEST_ORG_ID)) {
        console.error('Organization already exists, skipping creation');
        return;
    }

    // Get default subnets
    const defaults = await getOrgDefaults();

    const response = await apiRequest('PUT', '/org', {
        orgId: TEST_ORG_ID,
        name: TEST_ORG_NAME,
        subnet: defaults.subnet,
        utilitySubnet: defaults.utilitySubnet,
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to create organization: ${response.status} - ${text}`);
    }

    console.error('Organization created');
}

/**
 * Get site creation defaults
 */
async function getSiteDefaults() {
    console.error('Getting site defaults...');

    const response = await apiRequest('GET', `/org/${TEST_ORG_ID}/pick-site-defaults`);

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to get site defaults: ${response.status} - ${text}`);
    }

    const data = await response.json();
    return data.data;
}

/**
 * Check if a site with given name already exists
 */
async function siteExists(siteName) {
    const response = await apiRequest('GET', `/org/${TEST_ORG_ID}/sites`);
    if (!response.ok) {
        return false;
    }
    const data = await response.json();
    const sites = data.data || [];
    return sites.some(s => s.name === siteName);
}

/**
 * Create a local site
 */
async function createSite() {
    console.error('Creating local site...');

    // Check if site already exists
    if (await siteExists(TEST_SITE_NAME)) {
        console.error('Site already exists, skipping creation');
        return;
    }

    const response = await apiRequest('PUT', `/org/${TEST_ORG_ID}/site`, {
        name: TEST_SITE_NAME,
        type: 'local',
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to create site: ${response.status} - ${text}`);
    }

    console.error('Site created');
}

/**
 * List existing API keys
 */
async function listApiKeys() {
    const response = await apiRequest('GET', `/org/${TEST_ORG_ID}/api-keys`);
    if (!response.ok) {
        return [];
    }
    const data = await response.json();
    return data.data || [];
}

/**
 * Get all available API key actions/permissions
 */
async function getAvailableActions() {
    // The actions are defined in the Pangolin codebase
    // These are the standard actions available for org-level API keys
    return [
        'getOrg', 'updateOrg', 'deleteOrg',
        'listSites', 'getSite', 'createSite', 'updateSite', 'deleteSite',
        'listResources', 'getResource', 'createResource', 'updateResource', 'deleteResource',
        'listTargets', 'getTarget', 'createTarget', 'updateTarget', 'deleteTarget',
        'listRoles', 'getRole', 'createRole', 'updateRole', 'deleteRole',
        'listUsers', 'getUser', 'createUser', 'updateUser', 'deleteUser',
        'listInvitations', 'createInvitation', 'deleteInvitation',
        'listApiKeys', 'getApiKey', 'deleteApiKey',
        'listDomains', 'getDomain', 'createDomain', 'updateDomain', 'deleteDomain',
        'listClients', 'getClient', 'createClient', 'updateClient', 'deleteClient',
        'listAccessTokens', 'getAccessToken', 'createAccessToken', 'deleteAccessToken',
    ];
}

/**
 * Set API key permissions
 */
async function setApiKeyPermissions(apiKeyId, actions) {
    console.error(`Setting API key permissions (${actions.length} actions)...`);

    const response = await apiRequest('POST', `/org/${TEST_ORG_ID}/api-key/${apiKeyId}/actions`, {
        actionIds: actions,
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to set API key permissions: ${response.status} - ${text}`);
    }

    console.error('API key permissions set');
}

/**
 * Generate an API key with all permissions
 */
async function generateApiKey() {
    console.error('Generating API key...');

    // Check if we already have an integration-test-key
    const existingKeys = await listApiKeys();
    const existingKey = existingKeys.find(k => k.name === 'integration-test-key');
    if (existingKey) {
        console.error('API key already exists but we cannot retrieve the secret.');
        console.error('Creating a new API key with different name...');
    }

    const keyName = existingKey
        ? `integration-test-key-${Date.now()}`
        : 'integration-test-key';

    const response = await apiRequest('PUT', `/org/${TEST_ORG_ID}/api-key`, {
        name: keyName,
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Failed to create API key: ${response.status} - ${text}`);
    }

    const data = await response.json();
    const apiKeyData = data.data;

    if (!apiKeyData || !apiKeyData.apiKey) {
        throw new Error('API key response did not contain the key');
    }

    const apiKey = apiKeyData.apiKey;
    const apiKeyId = apiKeyData.apiKeyId;

    console.error(`API key generated: ${apiKey.substring(0, 10)}...`);

    // Set all permissions on the API key
    const actions = await getAvailableActions();
    await setApiKeyPermissions(apiKeyId, actions);

    return apiKey;
}

async function main() {
    if (!PANGOLIN_SETUP_TOKEN) {
        console.error('Error: PANGOLIN_SETUP_TOKEN environment variable is required');
        console.error('You can find this token in the Pangolin container logs');
        process.exit(1);
    }

    try {
        await waitForPangolin();
        await completeInitialSetup();
        await login();
        await createOrganization();
        await createSite();
        const apiKey = await generateApiKey();

        // Determine API URL based on dashboard URL
        // Integration API is on port 3003 with /v1/ prefix (bypasses CSRF protection)
        // Port 3000 (external) has CSRF protection that requires browser session
        // Port 3001 is internal API without full endpoints
        // Port 3002 is Next.js dashboard
        const apiUrl = PANGOLIN_DASHBOARD_URL.replace(':3002', ':3003');

        // Output credentials for CI (to stdout for parsing)
        console.log(`PANGOLIN_API_URL=${apiUrl}`);
        console.log(`PANGOLIN_API_KEY=${apiKey}`);
        console.log(`PANGOLIN_ORG_ID=${TEST_ORG_ID}`);

        console.error('Bootstrap completed successfully!');

    } catch (error) {
        console.error('Bootstrap failed:', error.message);
        process.exit(1);
    }
}

main();
