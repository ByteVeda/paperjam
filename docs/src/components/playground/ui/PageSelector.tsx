import styles from '../playground.module.css';

interface Props {
  page: number;
  pageCount: number;
  onChange: (page: number) => void;
}

export default function PageSelector({ page, pageCount, onChange }: Props) {
  return (
    <div className={styles.pageSelector}>
      <button
        type="button"
        className={styles.btn}
        disabled={page <= 1}
        onClick={() => onChange(page - 1)}
        aria-label="Previous page"
      >
        &larr;
      </button>
      <label>
        Page{' '}
        <input
          type="number"
          min={1}
          max={pageCount}
          value={page}
          onChange={(e) => {
            const v = parseInt(e.target.value, 10);
            if (v >= 1 && v <= pageCount) onChange(v);
          }}
          aria-label="Page number"
          className={styles.pageInput}
        />{' '}
        of {pageCount}
      </label>
      <button
        type="button"
        className={styles.btn}
        disabled={page >= pageCount}
        onClick={() => onChange(page + 1)}
        aria-label="Next page"
      >
        &rarr;
      </button>
    </div>
  );
}
