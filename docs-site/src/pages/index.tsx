import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import type React from 'react';

const features = [
  {
    title: 'Text Extraction',
    description:
      'Extract text, lines, and spans with font info and positions from any PDF.',
  },
  {
    title: 'Table Detection',
    description:
      'Detect and extract structured tables using lattice, stream, or auto strategies.',
  },
  {
    title: 'PDF to Markdown',
    description:
      'Convert entire PDFs to clean, structured Markdown with layout awareness.',
  },
  {
    title: 'Split & Merge',
    description:
      'Split documents by page ranges and merge multiple PDFs into one.',
  },
  {
    title: 'Native Async',
    description:
      'True async support powered by Rust and tokio — no Python thread pools.',
  },
  {
    title: 'WASM Playground',
    description: 'Try paperjam in your browser — no installation needed.',
  },
];

function Feature({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <div style={{ flex: '1 1 300px', padding: '1rem' }}>
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  );
}

export default function Home(): React.JSX.Element {
  return (
    <Layout title="Home" description="Fast PDF processing powered by Rust">
      <header style={{ textAlign: 'center', padding: '4rem 2rem' }}>
        <h1 style={{ fontSize: '3rem' }}>paperjam</h1>
        <p style={{ fontSize: '1.25rem', opacity: 0.8 }}>
          Fast PDF processing powered by Rust
        </p>
        <div
          style={{
            marginTop: '2rem',
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
        <pre
          style={{
            marginTop: '2rem',
            display: 'inline-block',
            textAlign: 'left',
            padding: '1rem',
            borderRadius: '8px',
            background: 'var(--ifm-code-background)',
          }}
        >
          pip install paperjam
        </pre>
      </header>

      <main
        style={{ padding: '3rem 2rem', maxWidth: '1200px', margin: '0 auto' }}
      >
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
          {features.map((f, i) => (
            <Feature key={i} {...f} />
          ))}
        </div>

        <div style={{ marginTop: '3rem', textAlign: 'center' }}>
          <h2>Quick Example</h2>
          <pre
            style={{
              textAlign: 'left',
              padding: '1.5rem',
              borderRadius: '8px',
              background: 'var(--ifm-code-background)',
              display: 'inline-block',
              maxWidth: '600px',
              width: '100%',
            }}
          >
            {`import paperjam

doc = paperjam.open("report.pdf")

# Extract text
text = doc.pages[0].extract_text()

# Extract tables
tables = doc.pages[0].extract_tables()

# Convert to Markdown
md = doc.to_markdown(layout_aware=True)

# Async support
doc = await paperjam.aopen("report.pdf")
md = await doc.ato_markdown()`}
          </pre>
        </div>
      </main>
    </Layout>
  );
}
