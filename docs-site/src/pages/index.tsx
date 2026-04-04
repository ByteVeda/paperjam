import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import type React from 'react';

const features = [
  {
    icon: '\uD83D\uDCC4',
    title: 'Multi-Format Support',
    description:
      'One unified API for PDF, DOCX, XLSX, PPTX, HTML, and EPUB documents.',
  },
  {
    icon: '\uD83D\uDCDD',
    title: 'Text & Table Extraction',
    description:
      'Extract text, lines, spans, and structured tables with positions and font info.',
  },
  {
    icon: '\uD83D\uDD04',
    title: 'Format Conversion',
    description:
      'Convert between formats seamlessly \u2014 PDF to DOCX, XLSX to PDF, HTML to EPUB, and more.',
  },
  {
    icon: '\u2699\uFE0F',
    title: 'Pipeline Engine',
    description:
      'Define multi-step document workflows in YAML \u2014 extract, transform, convert, and export.',
  },
  {
    icon: '\uD83E\uDD16',
    title: 'MCP Server (AI-native)',
    description:
      'Expose document operations as tools for AI agents via the Model Context Protocol.',
  },
  {
    icon: '\uD83D\uDCBB',
    title: 'CLI Tool',
    description:
      'Full-featured command-line interface for scripting and automation.',
  },
  {
    icon: '\u26A1',
    title: 'Native Async',
    description:
      'True async support powered by Rust and tokio \u2014 no Python thread pools.',
  },
  {
    icon: '\uD83C\uDF10',
    title: 'WASM Playground',
    description:
      'Try paperjam directly in your browser \u2014 no installation needed.',
  },
  {
    icon: '\uD83D\uDD12',
    title: 'Security & Compliance',
    description:
      'Encryption, redaction, digital signatures, PDF/A validation, and PDF/UA accessibility.',
  },
];

const formats = [
  { name: 'PDF', color: '#dc4a2f' },
  { name: 'DOCX', color: '#2b5797' },
  { name: 'XLSX', color: '#217346' },
  { name: 'PPTX', color: '#d24726' },
  { name: 'HTML', color: '#e44d26' },
  { name: 'EPUB', color: '#8b5cf6' },
];

function FeatureCard({
  icon,
  title,
  description,
}: {
  icon: string;
  title: string;
  description: string;
}) {
  return (
    <div
      style={{
        flex: '1 1 320px',
        padding: '1.5rem',
        borderRadius: '12px',
        border: '1px solid var(--ifm-color-emphasis-200)',
        background: 'var(--ifm-card-background-color)',
        transition: 'box-shadow 0.2s ease',
      }}
    >
      <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>{icon}</div>
      <h3 style={{ marginBottom: '0.5rem' }}>{title}</h3>
      <p style={{ margin: 0, color: 'var(--ifm-color-emphasis-700)' }}>
        {description}
      </p>
    </div>
  );
}

function FormatBadge({ name, color }: { name: string; color: string }) {
  return (
    <span
      style={{
        display: 'inline-block',
        padding: '0.4rem 1.2rem',
        borderRadius: '999px',
        background: color,
        color: '#fff',
        fontWeight: 600,
        fontSize: '0.95rem',
        letterSpacing: '0.02em',
      }}
    >
      {name}
    </span>
  );
}

export default function Home(): React.JSX.Element {
  return (
    <Layout title="Home" description="Fast document processing powered by Rust">
      {/* Hero */}
      <header
        style={{
          textAlign: 'center',
          padding: '5rem 2rem 3rem',
        }}
      >
        <img
          src={useBaseUrl('/img/logo.jpeg')}
          alt="paperjam logo"
          style={{
            width: '180px',
            height: 'auto',
            marginBottom: '1.5rem',
            borderRadius: '16px',
          }}
        />
        <h1
          style={{
            fontSize: '2.5rem',
            fontWeight: 800,
            marginBottom: '0.5rem',
          }}
        >
          paperjam
        </h1>
        <p
          style={{
            fontSize: '1.25rem',
            color: 'var(--ifm-color-emphasis-700)',
            marginBottom: '0.25rem',
          }}
        >
          Fast document processing powered by Rust
        </p>
        <p
          style={{
            fontSize: '1.1rem',
            color: 'var(--ifm-color-emphasis-600)',
            marginBottom: '2rem',
          }}
        >
          One API. Every document format. Rust speed.
        </p>

        <pre
          style={{
            display: 'inline-block',
            textAlign: 'left',
            padding: '0.75rem 1.5rem',
            borderRadius: '8px',
            background: 'var(--ifm-code-background)',
            fontSize: '1rem',
            marginBottom: '2rem',
          }}
        >
          pip install paperjam
        </pre>

        <div
          style={{
            display: 'flex',
            gap: '1rem',
            justifyContent: 'center',
            flexWrap: 'wrap',
          }}
        >
          <Link
            className="button button--primary button--lg"
            to="/getting-started/installation"
          >
            Get Started
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="/playground/"
          >
            Try in Browser
          </Link>
        </div>
      </header>

      <main style={{ maxWidth: '1200px', margin: '0 auto', padding: '0 2rem' }}>
        {/* Feature Grid */}
        <section style={{ padding: '3rem 0' }}>
          <h2 style={{ textAlign: 'center', marginBottom: '2rem' }}>
            Features
          </h2>
          <div
            style={{
              display: 'flex',
              flexWrap: 'wrap',
              gap: '1rem',
              justifyContent: 'center',
            }}
          >
            {features.map((f, i) => (
              <FeatureCard key={i} {...f} />
            ))}
          </div>
        </section>

        {/* Supported Formats */}
        <section style={{ padding: '3rem 0', textAlign: 'center' }}>
          <h2 style={{ marginBottom: '1.5rem' }}>Supported Formats</h2>
          <div
            style={{
              display: 'flex',
              gap: '0.75rem',
              justifyContent: 'center',
              flexWrap: 'wrap',
            }}
          >
            {formats.map((f, i) => (
              <FormatBadge key={i} {...f} />
            ))}
          </div>
        </section>

        {/* Quick Example */}
        <section style={{ padding: '3rem 0', textAlign: 'center' }}>
          <h2 style={{ marginBottom: '1.5rem' }}>Quick Example</h2>
          <pre
            style={{
              textAlign: 'left',
              padding: '1.5rem',
              borderRadius: '12px',
              background: 'var(--ifm-code-background)',
              display: 'inline-block',
              maxWidth: '680px',
              width: '100%',
              fontSize: '0.95rem',
              lineHeight: 1.6,
              overflow: 'auto',
            }}
          >
            {`import paperjam

# Open any document format
doc = paperjam.open("report.pdf")
docx = paperjam.open("document.docx")
xlsx = paperjam.open("data.xlsx")

# Extract text from any format
text = doc.pages[0].extract_text()

# Extract tables
tables = xlsx.pages[0].extract_tables()

# Convert between formats
paperjam.convert("report.pdf", "report.docx")
paperjam.convert("data.xlsx", "data.pdf")

# Async support
doc = await paperjam.aopen("report.pdf")
md = await doc.ato_markdown()`}
          </pre>
        </section>
      </main>
    </Layout>
  );
}
