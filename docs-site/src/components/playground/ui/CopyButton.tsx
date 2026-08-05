import { useClipboard } from '@site/src/hooks/useClipboard';
import styles from '../playground.module.css';

interface Props {
  text: string;
  label?: string;
}

export default function CopyButton({ text, label = 'Copy' }: Props) {
  const { copied, copy } = useClipboard();

  return (
    <button
      type="button"
      className={styles.copyBtn}
      onClick={() => copy(text)}
      aria-label={copied ? 'Copied' : label}
    >
      {copied ? 'Copied!' : label}
    </button>
  );
}
