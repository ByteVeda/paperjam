import React from 'react';
import CopyButton from './CopyButton';
import styles from '../playground.module.css';

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
