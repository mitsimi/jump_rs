import styles from "./Header.module.css";

export function Header() {
  return (
    <header className={styles.header}>
      <div className={styles.brand}>
        <h1 className={styles.title}>
          JUMP<span className={styles.accent}>_</span>RS
        </h1>
        <div className={styles.subtitle}>Network Wake Controller</div>
      </div>
      <div className={styles.status}>
        <div className={styles.statusDot}></div>
        <span className={styles.statusText}>System Active</span>
      </div>
    </header>
  );
}
