import { useDevices } from "../hooks/useDevices";
import { useWakeDevice } from "../hooks/useWakeDevice";
import { useDeleteDevice } from "../hooks/useDeleteDevice";
import { useToast } from "../hooks/useToast";
import { DeviceCard } from "./DeviceCard";
import styles from "./DeviceGrid.module.css";
import type { Device } from "../types/device";

interface DeviceGridProps {
  onAddDevice: () => void;
  onEditDevice: (device: Device) => void;
}

export function DeviceGrid({ onAddDevice, onEditDevice }: DeviceGridProps) {
  const { data: devices, isLoading, error } = useDevices();
  const wakeDevice = useWakeDevice();
  const deleteDevice = useDeleteDevice();
  const { showToast } = useToast();

  const handleWake = (device: Device) => {
    wakeDevice.mutate(device.id, {
      onSuccess: () => {
        showToast(`Wake packet sent to ${device.name}`, "success");
      },
      onError: () => {
        showToast("Failed to send wake packet", "error");
      },
    });
  };

  const handleDelete = async (device: Device) => {
    if (confirm(`Remove ${device.name}?`)) {
      try {
        await deleteDevice.mutateAsync(device.id);
        showToast(`${device.name} removed`, "success");
      } catch {
        showToast("Failed to delete device", "error");
      }
    }
  };

  if (isLoading) {
    return (
      <div className={styles.section}>
        <div className={styles.grid}>
          <div className="empty-state">
            <div className="empty-state-icon">Loading...</div>
            <div className="empty-state-title">Loading Devices</div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.section}>
        <div className={styles.grid}>
          <div className="empty-state">
            <div className="empty-state-icon">Error</div>
            <div className="empty-state-title">Failed to Load Devices</div>
            <div className="empty-state-text">
              Please try refreshing the page
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.section}>
      <div className={styles.header}>
        <h2 className={styles.title}>Controlled Devices</h2>
        <button className={styles.addBtn} onClick={onAddDevice}>
          <span className={styles.addBtnIcon}>
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
              <path d="M5 12h14"></path>
              <path d="M12 5v14"></path>
            </svg>
          </span>
          <span>Add Device</span>
        </button>
      </div>

      {!devices || devices.length === 0 ? (
        <div className={styles.emptyGrid}>
          <div className={styles.emptyBanner}>
            <div className={styles.emptyIcon}>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
                <path d="M3 3v5h5"></path>
              </svg>
            </div>
            <div className={styles.emptyTitle}>No Devices Found</div>
            <div className={styles.emptyText}>
              Add your first device to start controlling your network wake
              capabilities.
            </div>
          </div>
        </div>
      ) : (
        <div className={styles.grid}>
          {devices.map((device) => (
            <DeviceCard
              key={device.id}
              device={device}
              onWake={() => handleWake(device)}
              onEdit={() => onEditDevice(device)}
              onDelete={() => handleDelete(device)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
