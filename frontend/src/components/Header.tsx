import styles from "./Header.module.css";

interface HeaderProps {
  onImportExport: () => void;
}

export function Header({ onImportExport }: HeaderProps) {
  return (
    <header className={styles.header}>
      <div className={styles.brand}>
        <h1 className={styles.title}>
          JUMP<span className={styles.accent}>_</span>RS
        </h1>
        <div className={styles.subtitle}>Network Wake Controller</div>
      </div>
      <div className={styles.right}>
        <button className={styles.dataBtn} onClick={onImportExport}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M12 15V3"></path>
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <path d="m7 10 5 5 5-5"></path>
          </svg>
          IMPORT / EXPORT
        </button>
        <div className={styles.status}>
          <div className={styles.statusDot}></div>
          <span className={styles.statusText}>System Active</span>
        </div>
      </div>
    </header>
  );
}
