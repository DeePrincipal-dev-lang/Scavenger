# Scavngr Documentation Website

This directory contains the source for the [Scavngr documentation website](https://docs.scavngr.io), built with [Docusaurus 3](https://docusaurus.io/).

## Local Development

```bash
npm install
npm start
```

Opens http://localhost:3000 with live reload.

## Build

```bash
npm run build
```

Generates static content in `build/`. Serve with:

```bash
npm run serve
```

## Deployment

The site is automatically deployed to GitHub Pages on every push to `main` that modifies files in `website/` or `docs/`. See `.github/workflows/docs.yml`.

## Adding Pages

1. Create a `.md` or `.mdx` file in `docs/`.
2. Add it to `sidebars.js`.
3. Submit a PR.

See the [Documentation Style Guide](/docs/contributing/style-guide) for writing conventions.

## Search

Search is powered by Algolia DocSearch. To update the index configuration, edit `docusaurus.config.js` → `themeConfig.algolia`.

## Analytics

Google Analytics (GA4) is enabled via the `@docusaurus/plugin-google-gtag` plugin. The tracking ID is configured in `docusaurus.config.js`.
