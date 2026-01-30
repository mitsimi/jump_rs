import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { Modal, ModalContent, ModalFooter } from "./Modal";
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
    const body = {
      name: data.name,
      mac_address: data.mac_address,
      ip_address: data.ip_address || null,
      port: data.port ? parseInt(data.port, 10) : 9,
      description: data.description || null,
    };

    if (device) {
      await updateDevice.mutateAsync(
        { id: device.id, data: body },
        {
          onSuccess: () => {
            reset();
            onClose();
          },
          onError: (error) => {
            const errorMessage =
              error instanceof Error ? error.message : "Failed to save changes";
            toast.showToast(errorMessage, "error");
          },
        },
      );
    } else {
      await createDevice.mutateAsync(body, {
        onSuccess: () => {
          reset();
          onClose();
        },
        onError: (error) => {
          const errorMessage =
            error instanceof Error
              ? error.message
              : "Failed to create device";
          toast.showToast(errorMessage, "error");
        },
      });
    }
  };

  const handleLookupMac = () => {
    const ip = getValues("ip_address");
    if (!ip) {
      alert("Enter an IP address first");
      return;
    }

    lookupMac.mutate(ip, {
      onSuccess: (result) => {
        setValue("mac_address", result.mac);
      },
      onError: (error) => {
        const errorMessage =
          error instanceof Error
            ? error.message
            : "Failed to lookup MAC address";
        toast.showToast(errorMessage, "error");
      },
    });
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={mode === "edit" ? "Edit Device" : "Add Device"}
    >
      <form onSubmit={handleSubmit(onSubmit)}>
        <ModalContent>
          <div className="form-group">
            <label className="form-label">Device Name</label>
            <input
              className="form-input"
              placeholder="e.g., Gaming PC"
              {...register("name", { required: "Name is required" })}
            />
            {errors.name && (
              <span className="error-message">{errors.name.message}</span>
            )}
          </div>

          <div className="form-group">
            <label className="form-label">
              MAC Address
              <span className="form-hint">AA:BB:CC:DD:EE:FF</span>
            </label>
            <div className="mac-input-wrapper">
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
                className="lookup-btn"
                onClick={handleLookupMac}
                disabled={lookupMac.isPending}
              >
                {lookupMac.isPending ? "..." : "Lookup"}
              </button>
            </div>
            {errors.mac_address && (
              <span className="error-message">
                {errors.mac_address.message}
              </span>
            )}
          </div>

          <div className="form-row">
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

          <div className="form-group" style={{ margin: "20px 0px" }}>
            <label className="form-label">
              Description <span className="form-hint">(optional)</span>
            </label>
            <input
              className="form-input"
              placeholder="Notes about this device..."
              {...register("description")}
            />
          </div>
        </ModalContent>
        <ModalFooter>
          <button type="button" className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            type="submit"
            className="btn btn-primary"
            disabled={createDevice.isPending || updateDevice.isPending}
          >
            {mode === "edit" ? "Save Changes" : "Add Device"}
          </button>
        </ModalFooter>
      </form>
    </Modal>
  );
}
