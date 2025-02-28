import { writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { SitemapStream, streamToPromise } from "sitemap";

// Get __dirname in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

(async () => {
  const sitemap = new SitemapStream({ hostname: "https://france-nuage.fr" });

  // Add your routes
  sitemap.write({ url: "/", changefreq: "daily", priority: 1.0 });
  sitemap.write({ url: "/about", changefreq: "monthly", priority: 0.8 });

  // Add footer page URLs (assuming all footer pages are static)
  const footerPages = [
    "/submitTicket",
    "/marketing",
    "/automation",
    "/documentation",
    "/insights",
    "/guides",
    "/legalNotice",
    "/termsOfService",
    "/privacyPolicy",
  ];

  for (const page of footerPages) {
    sitemap.write({ url: page, changefreq: "monthly", priority: 0.7 });
  }

  sitemap.end();

  const sitemapBuffer = await streamToPromise(sitemap);

  // Write sitemap to the `dist` folder
  const distPath = path.resolve(__dirname, "./public/sitemap.xml");
  await writeFile(distPath, sitemapBuffer);
})();
