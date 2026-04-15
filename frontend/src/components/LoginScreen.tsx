import { useForm } from "react-hook-form";
import { useLogin } from "../hooks/useLogin";
import styles from "./LoginScreen.module.css";

interface LoginFormData {
  username: string;
  password: string;
}

export function LoginScreen() {
  const loginMutation = useLogin();

  const {
    register,
    handleSubmit,
    formState: { errors },
    setError,
  } = useForm<LoginFormData>({
    defaultValues: {
      username: "",
      password: "",
    },
  });

  const onSubmit = async (data: LoginFormData) => {
    try {
      await loginMutation.mutateAsync(data);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Login failed";
      setError("root", { message: errorMessage });
    }
  };

  return (
    <div className={styles.screen}>
      <div className={styles.container}>
        {/* Decorative corner brackets */}
        <div className={`${styles.corner} ${styles.topLeft}`} />
        <div className={`${styles.corner} ${styles.topRight}`} />
        <div className={`${styles.corner} ${styles.bottomLeft}`} />
        <div className={`${styles.corner} ${styles.bottomRight}`} />

        <div className={styles.card}>
          <header className={styles.header}>
            <div className={styles.statusIndicator}>
              <span className={styles.statusDot} />
              <span className={styles.statusText}>System Locked</span>
            </div>
            <h1 className={styles.title}>Authentication Required</h1>
            <p className={styles.subtitle}>
              Enter credentials to access Jump.rs control panel
            </p>
          </header>

          <form onSubmit={handleSubmit(onSubmit)} className={styles.form}>
            {errors.root && (
              <div className={styles.errorBanner}>
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="8" x2="12" y2="12" />
                  <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <span>{errors.root.message}</span>
              </div>
            )}

            <div className={styles.fieldGroup}>
              <label className={styles.label} htmlFor="username">
                <span className={styles.labelText}>Username</span>
                <span className={styles.labelLine} />
              </label>
              <div className={styles.inputWrapper}>
                <span className={styles.inputIcon}>
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                  </svg>
                </span>
                <input
                  id="username"
                  className={styles.input}
                  placeholder="Enter username"
                  autoComplete="username"
                  autoFocus
                  {...register("username", {
                    required: "Username is required",
                  })}
                />
              </div>
              {errors.username && (
                <span className={styles.fieldError}>
                  {errors.username.message}
                </span>
              )}
            </div>

            <div className={styles.fieldGroup}>
              <label className={styles.label} htmlFor="password">
                <span className={styles.labelText}>Password</span>
                <span className={styles.labelLine} />
              </label>
              <div className={styles.inputWrapper}>
                <span className={styles.inputIcon}>
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
                    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                  </svg>
                </span>
                <input
                  id="password"
                  className={styles.input}
                  type="password"
                  placeholder="Enter password"
                  autoComplete="current-password"
                  {...register("password", {
                    required: "Password is required",
                  })}
                />
              </div>
              {errors.password && (
                <span className={styles.fieldError}>
                  {errors.password.message}
                </span>
              )}
            </div>

            <button
              type="submit"
              className={styles.submitBtn}
              disabled={loginMutation.isPending}
            >
              <span className={styles.btnText}>
                {loginMutation.isPending ? "Authenticating..." : "Authenticate"}
              </span>
              <span className={styles.btnIcon}>
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <path d="M5 12h14" />
                  <path d="m12 5 7 7-7 7" />
                </svg>
              </span>
            </button>
          </form>

          <footer className={styles.footer}>
            <div className={styles.footerLine} />
            <span className={styles.footerText}>Jump.rs Wake-on-LAN</span>
          </footer>
        </div>
      </div>
    </div>
  );
}
