import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'paperjam',
  tagline: 'Fast PDF processing powered by Rust',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://docs.byteveda.org',
  baseUrl: '/paperjam/',

  organizationName: 'ByteVeda',
  projectName: 'paperjam',

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  markdown: {
    mermaid: true,
  },

  themes: ['@docusaurus/theme-mermaid'],

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: '/',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/ByteVeda/paperjam/tree/main/docs-site/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    [
      require.resolve('@easyops-cn/docusaurus-search-local'),
      {
        hashed: true,
        language: ['en'],
        docsRouteBasePath: '/',
        searchResultLimits: 8,
      },
    ],
  ],

  themeConfig: {
    colorMode: {
      defaultMode: 'dark',
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'paperjam',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'gettingStartedSidebar',
          position: 'left',
          label: 'Getting Started',
        },
        {
          type: 'docSidebar',
          sidebarId: 'guidesSidebar',
          position: 'left',
          label: 'Guides',
        },
        {
          type: 'docSidebar',
          sidebarId: 'apiSidebar',
          position: 'left',
          label: 'API Reference',
        },
        {
          type: 'docSidebar',
          sidebarId: 'playgroundSidebar',
          position: 'left',
          label: 'Playground',
        },
        {
          href: 'https://github.com/ByteVeda/paperjam',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Installation', to: '/getting-started/installation'},
            {label: 'API Reference', to: '/api/functions'},
            {label: 'Guides', to: '/guides/extraction'},
          ],
        },
        {
          title: 'Tools',
          items: [
            {label: 'Interactive Playground', to: '/playground/'},
            {label: 'PyPI', href: 'https://pypi.org/project/paperjam/'},
          ],
        },
        {
          title: 'More',
          items: [
            {label: 'GitHub', href: 'https://github.com/ByteVeda/paperjam'},
          ],
        },
      ],
      copyright: `Copyright \u00a9 ${new Date().getFullYear()} paperjam contributors.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['python', 'rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
