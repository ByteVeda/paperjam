import type React from 'react';
import styles from '../playground.module.css';
import CopyButton from './CopyButton';

interface Props {
  children: React.ReactNode;
  copyText?: string;
  copyLabel?: string;
}

export default function ResultPanel({ children, copyText, copyLabel }: Props) {
  return (
    <div className={styles.resultPanel}>
      {copyText && (
        <div className={styles.resultPanelHeader}>
          <CopyButton text={copyText} label={copyLabel} />
        </div>
      )}
      <div className={styles.resultPanelBody}>{children}</div>
    </div>
  );
}
