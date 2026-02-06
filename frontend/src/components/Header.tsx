import styles from "./Header.module.css";
import { useAuth } from "../hooks/useAuth";
import { useLogout } from "../hooks/useLogout";

interface HeaderProps {
  onImportExport: () => void;
}

export function Header({ onImportExport }: HeaderProps) {
  const { isAuthenticated, isAuthRequired, username } = useAuth();
  const logoutMutation = useLogout();

  const handleLogout = () => {
    logoutMutation.mutate();
  };

  const showAuthControls = isAuthRequired && isAuthenticated;

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
        {showAuthControls && (
          <button
            className={styles.logoutBtn}
            onClick={handleLogout}
            disabled={logoutMutation.isPending}
          >
            <span className={styles.username}>{username}</span>
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
              <path d="m16 17 5-5-5-5"></path>
              <path d="M21 12H9"></path>
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
            </svg>
          </button>
        )}
      </div>
    </header>
  );
}
