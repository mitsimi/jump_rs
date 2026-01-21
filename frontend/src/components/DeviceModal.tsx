import { useEffect } from "react";
import { useForm } from "react-hook-form";
import styles from "./DeviceModal.module.css";
import { useCreateDevice } from "../hooks/useCreateDevice";
import { useUpdateDevice } from "../hooks/useUpdateDevice";
import { useMacLookup } from "../hooks/useMacLookup";
import type { DeviceModalProps, DeviceFormData } from "../types/device";
import { useToast } from "../context/ToastContext";

export function DeviceModal({
  isOpen,
  mode,
  device,
  onClose,
}: DeviceModalProps) {
  const createDevice = useCreateDevice();
  const updateDevice = useUpdateDevice();
  const lookupMac = useMacLookup();
  const toast = useToast();

  const {
    register,
    handleSubmit,
    formState: { errors },
    setValue,
    getValues,
    reset,
  } = useForm<DeviceFormData>({
    defaultValues: {
      name: "",
      mac_address: "",
      ip_address: "",
      port: "9",
      description: "",
    },
  });

  useEffect(() => {
    if (device) {
      reset({
        name: device.name,
        mac_address: device.mac_address,
        ip_address: device.ip_address || "",
        port: device.port.toString(),
        description: device.description || "",
      });
    } else {
      reset({
        name: "",
        mac_address: "",
        ip_address: "",
        port: "9",
        description: "",
      });
    }
  }, [device, reset]);

  const onSubmit = async (data: DeviceFormData) => {
    try {
      if (device) {
        await updateDevice.mutateAsync({ id: device.id, data });
      } else {
        await createDevice.mutateAsync(data);
      }
      onClose();
    } catch {
      // Error handled by mutation
    }
  };

  const handleLookupMac = async () => {
    const ip = getValues("ip_address");
    if (!ip) {
      alert("Enter an IP address first");
      return;
    }

    const result = await lookupMac.mutateAsync(ip);
    if (result.found && result.mac) {
      setValue("mac_address", result.mac);
    } else {
      toast.showToast(result.error!, "error");
    }
  };

  return (
    <div className={`${styles.overlay} ${isOpen ? styles.overlayActive : ""}`}>
      <div className={styles.modal}>
        <div className={styles.header}>
          <h3 className={styles.title}>
            {mode === "edit" ? "Edit Device" : "Add Device"}
          </h3>
          <button className={styles.closeBtn} onClick={onClose}>
            &times;
          </button>
        </div>

        <form onSubmit={handleSubmit(onSubmit)}>
          <div className={styles.body}>
            <div className="form-group">
              <label className="form-label">Device Name</label>
              <input
                className="form-input"
                placeholder="e.g., Gaming PC"
                {...register("name", { required: "Name is required" })}
              />
              {errors.name && (
                <span className={styles.error}>{errors.name.message}</span>
              )}
            </div>

            <div className="form-group">
              <label className="form-label">
                MAC Address
                <span className="form-hint">AA:BB:CC:DD:EE:FF</span>
              </label>
              <div className={styles.macInputWrapper}>
                <input
                  className="form-input"
                  placeholder="AA:BB:CC:DD:EE:FF"
                  {...register("mac_address", {
                    required: "MAC address is required",
                    pattern: {
                      value: /^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$/,
                      message: "Invalid MAC address format",
                    },
                  })}
                />
                <button
                  type="button"
                  className={styles.lookupBtn}
                  onClick={handleLookupMac}
                  disabled={lookupMac.isPending}
                >
                  {lookupMac.isPending ? "..." : "Lookup"}
                </button>
              </div>
              {errors.mac_address && (
                <span className={styles.error}>
                  {errors.mac_address.message}
                </span>
              )}
            </div>

            <div className={styles.formRow}>
              <div className="form-group" style={{ marginBottom: 0 }}>
                <label className="form-label">
                  IP Address <span className="form-hint">(optional)</span>
                </label>
                <input
                  className="form-input"
                  placeholder="192.168.1.100"
                  {...register("ip_address")}
                />
              </div>
              <div className="form-group" style={{ marginBottom: 0 }}>
                <label className="form-label">Port</label>
                <input
                  className="form-input"
                  type="number"
                  placeholder="9"
                  {...register("port")}
                />
              </div>
            </div>

            <div className="form-group" style={{ marginTop: "20px" }}>
              <label className="form-label">
                Description <span className="form-hint">(optional)</span>
              </label>
              <input
                className="form-input"
                placeholder="Notes about this device..."
                {...register("description")}
              />
            </div>
          </div>

          <div className={styles.footer}>
            <button
              type="button"
              className="btn btn-secondary"
              onClick={onClose}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={createDevice.isPending || updateDevice.isPending}
            >
              {mode === "edit" ? "Save Changes" : "Add Device"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
