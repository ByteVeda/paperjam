import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  gettingStartedSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
        'getting-started/quickstart',
        'getting-started/architecture',
      ],
    },
  ],

  guidesSidebar: [
    {
      type: 'category',
      label: 'Guides',
      collapsed: false,
      items: [
        'guides/extraction',
        'guides/tables',
        'guides/manipulation',
        'guides/metadata',
        'guides/annotations',
        'guides/layout',
        'guides/security',
        'guides/forms',
        'guides/diff',
        'guides/rendering',
        'guides/signatures',
        'guides/async',
      ],
    },
  ],

  apiSidebar: [
    {
      type: 'category',
      label: 'API Reference',
      collapsed: false,
      items: [
        'api/functions',
        'api/document',
        'api/page',
        'api/types',
        'api/enums',
        'api/exceptions',
      ],
    },
  ],

  playgroundSidebar: [
    {
      type: 'category',
      label: 'Playground',
      collapsed: false,
      items: [
        'playground/index',
        'playground/text-extraction',
        'playground/table-extraction',
        'playground/markdown',
        'playground/document-info',
        'playground/split-merge',
        'playground/search',
        'playground/security',
        'playground/structure-layout',
      ],
    },
  ],
};

export default sidebars;
