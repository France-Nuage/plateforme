import { SitemapStream, streamToPromise } from 'sitemap';
import { writeFile } from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

// Get __dirname in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

(async () => {
  const sitemap = new SitemapStream({ hostname: 'https://france-nuage.fr' });

  // Add your routes
  sitemap.write({ url: '/', changefreq: 'daily', priority: 1.0 });
  sitemap.write({ url: '/about', changefreq: 'monthly', priority: 0.8 });

  // Add footer page URLs (assuming all footer pages are static)
  const footerPages = [
    '/submitTicket',
    '/marketing',
    '/automation',
    '/documentation',
    '/insights',
    '/guides',
    '/legalNotice',
    '/termsOfService',
    '/privacyPolicy',
  ];

  for (const page of footerPages) {
    sitemap.write({ url: page, changefreq: 'monthly', priority: 0.7 });
  }

  sitemap.end();

  const sitemapBuffer = await streamToPromise(sitemap);

  // Write sitemap to the `dist` folder
  const distPath = path.resolve(__dirname, '.nuxt/dist/sitemap.xml');
  await writeFile(distPath, sitemapBuffer);

  console.log('Sitemap generated successfully in the dist folder.');
})();