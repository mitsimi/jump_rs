export interface Device {
  id: string;
  name: string;
  mac_address: string;
  ip_address: string | null;
  port: number;
  description: string | null;
}

export interface DeviceFormData {
  name: string;
  mac_address: string;
  ip_address: string | null;
  port: string;
  description: string | null;
}

export interface WakeResponse {
  success: boolean;
  message?: string;
}

export interface MacLookupRequest {
  ip: string;
}

export interface MacLookupResponse {
  found: boolean;
  mac?: string;
  error?: string;
}

export type ToastType = "success" | "error";

export interface ToastMessage {
  id: string;
  message: string;
  type: ToastType;
}

export type ModalMode = "add" | "edit";

export interface DeviceCardProps {
  device: Device;
  onWake: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export interface DeviceModalProps {
  isOpen: boolean;
  mode: ModalMode;
  device?: Device;
  onClose: () => void;
}

export interface ApiError {
  status: number;
  message: string;
}
