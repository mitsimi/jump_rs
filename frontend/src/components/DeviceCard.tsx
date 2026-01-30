import styles from "./DeviceCard.module.css";
import { useWakeDevice } from "../hooks/useWakeDevice";
import { useDeleteDevice } from "../hooks/useDeleteDevice";
import type { Device } from "../types/device";
import { useToast } from "../hooks/useToast";

interface DeviceCardProps {
  device: Device;
  onWake?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
}

export function DeviceCard({ device, onEdit }: DeviceCardProps) {
  const wakeDevice = useWakeDevice();
  const deleteDevice = useDeleteDevice();
  const { showToast } = useToast();

  const handleWake = () => {
    wakeDevice.mutate(device.id, {
      onSuccess: () => {
        showToast("Wake signal sent", "success");
      },
      onError: () => {
        showToast("Failed to send wake signal", "error");
      },
    });
  };

  const handleDelete = () => {
    if (confirm(`Remove ${device.name}?`)) {
      deleteDevice.mutate(device.id);
    }
  };

  return (
    <div className={`${styles.card}`}>
      <div className={styles.header}>
        <div className={styles.name}>{device.name}</div>
        <div className={styles.id}>{device.id.slice(0, 8)}</div>
      </div>

      <div className={styles.info}>
        <div className={styles.infoRow}>
          <span className={styles.label}>MAC</span>
          <span className={`${styles.value} ${styles.valueMac}`}>
            {device.mac_address}
          </span>
        </div>
        {device.ip_address && (
          <div className={styles.infoRow}>
            <span className={styles.label}>IP</span>
            <span className={styles.value}>{device.ip_address}</span>
          </div>
        )}
        {device.description && (
          <div className={styles.infoRow}>
            <span className={styles.label}>Note</span>
            <span className={styles.value}>{device.description}</span>
          </div>
        )}
      </div>

      <div className={styles.actions}>
        <button className="btn btn-primary" onClick={handleWake}>
          Wake
        </button>
        <button className="btn btn-secondary" onClick={onEdit}>
          Edit
        </button>
        <button className="btn btn-danger" onClick={handleDelete}>
          Remove
        </button>
      </div>
    </div>
  );
}
