import { useDevices } from "../hooks/useDevices";
import { useWakeDevice } from "../hooks/useWakeDevice";
import { useDeleteDevice } from "../hooks/useDeleteDevice";
import { usePacketCount } from "../hooks/usePacketCount";
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
  const { increment } = usePacketCount();
  const { showToast } = useToast();

  const handleWake = async (device: Device) => {
    try {
      const result = await wakeDevice.mutateAsync(device.id);
      if (result.success) {
        increment();
        showToast(`Wake packet sent to ${device.name}`, "success");
      } else {
        showToast(result.message || "Failed to send wake packet", "error");
      }
    } catch {
      showToast("Failed to send wake packet", "error");
    }
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
          <span className={styles.addBtnIcon}>+</span>
          <span>Add Device</span>
        </button>
      </div>

      {!devices || devices.length === 0 ? (
        <div className={styles.emptyGrid}>
          <div className={styles.emptyBanner}>
            <div className={styles.emptyIcon}>⟲</div>
            <div className={styles.emptyTitle}>No Devices Found</div>
            <div className={styles.emptyText}>
              Add your first device to start controlling your network wake
              capabilities.
            </div>
            <button className={styles.emptyAddBtn} onClick={onAddDevice}>
              <span className={styles.emptyAddBtnIcon}>+</span>
              <span>Add Device</span>
            </button>
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
