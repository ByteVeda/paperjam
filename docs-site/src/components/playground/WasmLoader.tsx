import React from 'react';
import { usePaperjam } from '@site/src/hooks/usePaperjam';
import type { WasmModule } from '@site/src/types/paperjam';
import styles from './playground.module.css';

interface Props {
  children: (wasm: WasmModule) => React.ReactNode;
}

export default function WasmLoader({ children }: Props) {
  const { paperjam, loading, error } = usePaperjam();

  if (loading) {
    return (
      <div className={styles.emptyState}>Loading WASM module...</div>
    );
  }

  if (error) {
    return (
      <div className={styles.errorAlert} role="alert">
        Failed to load WASM module: {error}
      </div>
    );
  }

  return <>{children(paperjam!)}</>;
}
