import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import type { WasmModule } from '@site/src/types/paperjam';
import { useEffect, useState } from 'react';

let cachedModule: WasmModule | null = null;

export function usePaperjam() {
  const { siteConfig } = useDocusaurusContext();
  const cacheBuster =
    (siteConfig.customFields?.wasmCacheBuster as string) || '';

  const [loading, setLoading] = useState(!cachedModule);
  const [error, setError] = useState<string | null>(null);
  const [paperjam, setPaperjam] = useState<WasmModule | null>(cachedModule);

  useEffect(() => {
    if (cachedModule) return;
    const wasmUrl = `/paperjam/wasm/paperjam_wasm.js?v=${cacheBuster}`;
    (async () => {
      try {
        const mod = await import(/* webpackIgnore: true */ wasmUrl);
        await mod.default();
        cachedModule = mod as WasmModule;
        setPaperjam(mod as WasmModule);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    })();
  }, [cacheBuster]);

  return { paperjam, loading, error };
}
