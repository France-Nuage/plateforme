const APP_URL = process.env.BASE_URL;
const APP_URL_DASHBOARD = new RegExp(`^${process.env.BASE_URL}/dashboard?.*$`);

export { APP_URL, APP_URL_DASHBOARD };
