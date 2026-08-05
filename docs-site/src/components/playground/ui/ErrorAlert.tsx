import styles from '../playground.module.css';

interface Props {
  error: string | null;
}

export default function ErrorAlert({ error }: Props) {
  if (!error) return null;
  return (
    <div className={styles.errorAlert} role="alert">
      {error}
    </div>
  );
}
