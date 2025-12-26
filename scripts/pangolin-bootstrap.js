#!/usr/bin/env node
/**
 * Pangolin Bootstrap Script
 *
 * This script automates the initial setup of Pangolin for CI testing.
 * It uses Playwright to interact with the Pangolin UI to:
 * 1. Complete the initial setup with the setup token
 * 2. Create a test organization
 * 3. Create a local site
 * 4. Generate an API key
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

const { chromium } = require('playwright');

const PANGOLIN_DASHBOARD_URL = process.env.PANGOLIN_DASHBOARD_URL || 'http://localhost:3002';
const PANGOLIN_SETUP_TOKEN = process.env.PANGOLIN_SETUP_TOKEN;

const TEST_ADMIN_EMAIL = 'admin@integration-test.local';
const TEST_ADMIN_PASSWORD = 'IntegrationTest123!';
const TEST_ORG_ID = 'integration-test-org';
const TEST_ORG_NAME = 'Integration Test Org';
const TEST_SITE_NAME = 'integration-test-site';

async function waitForPangolin(page, maxAttempts = 60) {
    for (let i = 0; i < maxAttempts; i++) {
        try {
            const response = await page.goto(`${PANGOLIN_DASHBOARD_URL}/setup`, {
                timeout: 5000,
                waitUntil: 'domcontentloaded'
            });
            if (response && response.ok()) {
                console.error('Pangolin dashboard is ready');
                return true;
            }
        } catch (e) {
            console.error(`Waiting for Pangolin dashboard... (${i + 1}/${maxAttempts})`);
            await new Promise(r => setTimeout(r, 2000));
        }
    }
    throw new Error('Pangolin dashboard did not become ready in time');
}

async function completeInitialSetup(page) {
    console.error('Starting initial setup...');

    // Navigate to setup page - it should redirect to /auth/initial-setup
    await page.goto(`${PANGOLIN_DASHBOARD_URL}/setup`);
    await page.waitForLoadState('networkidle');

    // Wait for the setup form
    await page.waitForTimeout(2000);

    // Fill setup token
    const tokenInput = page.getByRole('textbox', { name: /token|jeton/i });
    await tokenInput.fill(PANGOLIN_SETUP_TOKEN);

    // Fill email
    const emailInput = page.getByRole('textbox', { name: /email|mail/i });
    await emailInput.fill(TEST_ADMIN_EMAIL);

    // Fill password (first password field)
    const passwordInputs = page.getByRole('textbox', { name: /password|mot de passe/i });
    const passwordInput = passwordInputs.first();
    await passwordInput.fill(TEST_ADMIN_PASSWORD);

    // Fill confirm password
    const confirmPasswordInput = page.getByRole('textbox', { name: /confirm/i });
    await confirmPasswordInput.fill(TEST_ADMIN_PASSWORD);

    // Submit setup - look for create admin button
    const createButton = page.getByRole('button', { name: /create|créer/i });
    await createButton.click();

    // Wait for redirect to login page
    await page.waitForURL(/\/auth\/login/, { timeout: 30000 });

    console.error('Initial setup completed');
}

async function login(page) {
    console.error('Logging in...');

    // Wait for login page to be fully loaded
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    // Wait for login button to be visible before proceeding
    // Note: UI can be French ("Se connecter") or English ("Log in" with space)
    const loginButton = page.getByRole('button', { name: /se connecter|log\s*in|sign\s*in/i });
    await loginButton.waitFor({ state: 'visible', timeout: 30000 });
    console.error('Login page loaded, filling credentials...');

    // Fill email - supports both French ("Adresse mail") and English ("Email")
    const emailInput = page.getByRole('textbox', { name: /adresse.*mail|e-?mail/i });
    await emailInput.fill(TEST_ADMIN_EMAIL);

    // Fill password - supports both French ("Mot de passe") and English ("Password")
    const passwordInput = page.getByRole('textbox', { name: /mot de passe|password/i });
    await passwordInput.fill(TEST_ADMIN_PASSWORD);

    // Click login button
    await loginButton.click();

    // Wait for redirect to setup/org creation
    await page.waitForURL(/\/setup/, { timeout: 30000 });

    console.error('Logged in successfully');
}

async function createOrganization(page) {
    console.error('Creating organization...');

    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Fill org name - French: "Nom de l'organisation", English: "Organization Name"
    const orgNameInput = page.getByRole('textbox', { name: /nom.*organisation|organization\s*name/i });
    await orgNameInput.fill(TEST_ORG_NAME);

    // Fill org ID - French: "ID de l'organisation", English: "Organization ID"
    const orgIdInput = page.getByRole('textbox', { name: /id.*organisation|organization\s*id/i });
    await orgIdInput.fill(TEST_ORG_ID);

    // Click create organization - French: "Créer une organisation", English: "Create Organization"
    const createOrgButton = page.getByRole('button', { name: /créer.*organisation|create\s*org/i });
    await createOrgButton.click();

    // Wait for redirect to site creation
    await page.waitForURL(/\/settings\/sites\/create/, { timeout: 30000 });

    console.error('Organization created');
}

async function createSite(page) {
    console.error('Creating local site...');

    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // The "Local" option should already be selected, but click it to be sure
    const localRadio = page.getByRole('radio', { name: /locale|local/i });
    if (await localRadio.isVisible()) {
        await localRadio.click();
    }

    // Fill site name - French: "Nom", English: "Name"
    const siteNameInput = page.getByRole('textbox', { name: /^nom$|^name$/i });
    await siteNameInput.fill(TEST_SITE_NAME);

    // Click create site - French: "Créer un nœud", English: "Create Site/Node"
    const createSiteButton = page.getByRole('button', { name: /créer.*nœud|create\s*(site|node)/i });
    await createSiteButton.click();

    // Wait for redirect to site settings
    await page.waitForURL(/\/settings\/sites\/.*\/general/, { timeout: 30000 });

    console.error('Site created');
}

async function generateApiKey(page) {
    console.error('Generating API key...');

    // Navigate to API keys page
    await page.goto(`${PANGOLIN_DASHBOARD_URL}/${TEST_ORG_ID}/settings/api-keys`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Click generate API key button - French: "Générer une clé d'API", English: "Generate API Key"
    const generateButton = page.getByRole('button', { name: /générer.*clé.*api|generate\s*api\s*key/i });
    await generateButton.click();

    // Wait for form to load
    await page.waitForURL(/\/api-keys\/create/, { timeout: 10000 });
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Fill API key name - French: "Nom", English: "Name"
    const nameInput = page.getByRole('textbox', { name: /^nom$|^name$/i });
    await nameInput.fill('integration-test-key');

    // Check "Allow all permissions" - use the specific checkbox id to avoid matching category toggles
    const allPermissionsCheckbox = page.locator('#toggle-all-permissions');
    await allPermissionsCheckbox.click();

    // Click generate - French: "Générer", English: "Generate"
    const submitButton = page.getByRole('button', { name: /^générer$|^generate$/i });
    await submitButton.click();

    // Wait for the API key to be shown
    await page.waitForTimeout(3000);

    // Extract the API key from the code element
    const apiKeyElement = page.locator('code');
    const apiKey = await apiKeyElement.textContent();

    if (!apiKey) {
        throw new Error('Failed to extract API key');
    }

    console.error('API key generated');
    return apiKey.trim();
}

async function main() {
    if (!PANGOLIN_SETUP_TOKEN) {
        console.error('Error: PANGOLIN_SETUP_TOKEN environment variable is required');
        console.error('You can find this token in the Pangolin container logs');
        process.exit(1);
    }

    const browser = await chromium.launch({
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
        await waitForPangolin(page);
        await completeInitialSetup(page);
        await login(page);
        await createOrganization(page);
        await createSite(page);
        const apiKey = await generateApiKey(page);

        // Determine API URL based on dashboard URL
        const apiUrl = PANGOLIN_DASHBOARD_URL.replace(':3002', ':3003') + '/v1';

        // Output credentials for CI (to stdout for parsing)
        console.log(`PANGOLIN_API_URL=${apiUrl}`);
        console.log(`PANGOLIN_API_KEY=${apiKey}`);
        console.log(`PANGOLIN_ORG_ID=${TEST_ORG_ID}`);

        console.error('Bootstrap completed successfully!');

    } catch (error) {
        console.error('Bootstrap failed:', error.message);

        // Take screenshot for debugging
        try {
            await page.screenshot({ path: '/tmp/pangolin-bootstrap-error.png', fullPage: true });
            console.error('Error screenshot saved to /tmp/pangolin-bootstrap-error.png');
        } catch (e) {
            console.error('Could not save error screenshot');
        }

        process.exit(1);
    } finally {
        await browser.close();
    }
}

main();
