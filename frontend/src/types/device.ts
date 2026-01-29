export type { Device, ImportRequest as ImportDevice } from "../api/generated";

export interface DeviceFormData {
  name: string;
  mac_address: string;
  ip_address: string | null;
  port: string;
  description: string | null;
}

export type ToastType = "success" | "error";

export interface ToastMessage {
  id: string;
  message: string;
  type: ToastType;
}

export type ModalMode = "add" | "edit";

export interface DeviceModalProps {
  isOpen: boolean;
  mode: ModalMode;
  device?: import("../api/generated").Device;
  onClose: () => void;
}
