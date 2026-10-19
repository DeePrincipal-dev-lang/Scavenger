// @ts-check
/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/environment-setup',
      ],
    },
    {
      type: 'category',
      label: 'Architecture',
      items: [
        'architecture/overview',
        'architecture/smart-contract',
        'architecture/frontend',
        'architecture/indexer',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/participant-registration',
        'guides/waste-submission',
        'guides/waste-transfer',
        'guides/incentives',
        'guides/rewards',
      ],
    },
    {
      type: 'category',
      label: 'Deployment',
      items: [
        'deployment/testnet',
        'deployment/mainnet',
        'deployment/docker',
        'deployment/ci-cd',
      ],
    },
    {
      type: 'category',
      label: 'Contributing',
      items: [
        'contributing/index',
        'contributing/code-style',
        'contributing/testing',
        'contributing/i18n',
      ],
    },
    {
      type: 'category',
      label: 'Governance',
      items: [
        'governance/charter',
        'governance/decision-making',
        'governance/contribution-process',
        'governance/conflict-resolution',
        'governance/faq',
      ],
    },
  ],

  apiSidebar: [
    'api/overview',
    {
      type: 'category',
      label: 'Contract Functions',
      items: [
        'api/admin',
        'api/participants',
        'api/waste',
        'api/incentives',
        'api/stats',
      ],
    },
    'api/errors',
    'api/events',
  ],
};

module.exports = sidebars;
