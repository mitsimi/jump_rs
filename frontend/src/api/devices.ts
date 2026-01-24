import type {
  Device,
  DeviceFormData,
  MacLookupResponse,
} from "../types/device";

const API_BASE = "/api";

class ApiErrorClass extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({
      message: `HTTP Error: ${response.status}`,
    }));
    throw new ApiErrorClass(response.status, error.message || "Unknown error");
  }
  return response.json();
}

export async function fetchDevices(): Promise<Device[]> {
  const response = await fetch(`${API_BASE}/devices`);
  return handleResponse<Device[]>(response);
}

export async function createDevice(data: DeviceFormData): Promise<Device> {
  const payload = {
    name: data.name,
    mac_address: data.mac_address,
    ip_address: data.ip_address || null,
    port: parseInt(data.port, 10) || 9,
    description: data.description || null,
  };

  const response = await fetch(`${API_BASE}/devices`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<Device>(response);
}

export async function updateDevice(
  id: string,
  data: DeviceFormData,
): Promise<Device> {
  const payload = {
    name: data.name,
    mac_address: data.mac_address,
    ip_address: data.ip_address || null,
    port: parseInt(data.port, 10) || 9,
    description: data.description || null,
  };

  const response = await fetch(`${API_BASE}/devices/${id}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<Device>(response);
}

export async function deleteDevice(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/devices/${id}`, {
    method: "DELETE",
  });
  await handleResponse<void>(response);
}

export async function wakeDevice(id: string): Promise<void> {
  const response = await fetch(`${API_BASE}/devices/${id}/wake`, {
    method: "POST",
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({
      message: `HTTP Error: ${response.status}`,
    }));
    throw new ApiErrorClass(response.status, error.message || "Unknown error");
  }
  // Backend returns 204 No Content, no response body
}

export async function lookupMacAddress(ip: string): Promise<MacLookupResponse> {
  const response = await fetch(`${API_BASE}/arp-lookup`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ ip }),
  });
  return handleResponse<MacLookupResponse>(response);
}

export interface ExportDevice {
  name: string;
  mac_address: string;
  port: number;
  ip_address: string | null;
  description: string | null;
}

export async function exportDevices(): Promise<ExportDevice[]> {
  const response = await fetch(`${API_BASE}/devices/export`, {
    method: "GET",
  });
  return handleResponse<ExportDevice[]>(response);
}

export async function importDevices(
  devices: {
    name: string;
    mac_address?: string;
    ip_address: string | null;
    port?: number;
    description: string | null;
  }[],
): Promise<Device[]> {
  const payload = devices.map((d) => ({
    name: d.name,
    mac_address: d.mac_address || "",
    port: d.port || 9,
    ip_address: d.ip_address || null,
    description: d.description || null,
  }));

  const response = await fetch(`${API_BASE}/devices/import`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<Device[]>(response);
}

export type { ApiErrorClass as ApiError };
