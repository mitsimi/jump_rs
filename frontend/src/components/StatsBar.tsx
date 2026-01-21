import styles from "./StatsBar.module.css";
import { useDevices } from "../hooks/useDevices";
import { usePacketCount } from "../hooks/usePacketCount";

export function StatsBar() {
  const { data: devices } = useDevices();
  const { count } = usePacketCount();

  return (
    <div className={styles.statsBar}>
      <div className={styles.statItem}>
        <div className={styles.statValue}>{devices?.length || 0}</div>
        <div className={styles.statLabel}>Registered Devices</div>
      </div>
      <div className={styles.statItem}>
        <div className={styles.statValue}>{count}</div>
        <div className={styles.statLabel}>Packets Sent</div>
      </div>
    </div>
  );
}
