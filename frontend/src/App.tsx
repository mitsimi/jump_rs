import { useState } from "react";
import { Header } from "./components/Header";
import { StatsBar } from "./components/StatsBar";
import { DeviceGrid } from "./components/DeviceGrid";
import { DeviceModal } from "./components/DeviceModal";
import { ToastProvider } from "./components/Toast";
import type { Device } from "./types/device";

function AppContent() {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingDevice, setEditingDevice] = useState<Device | null>(null);

  const openAddModal = () => {
    setEditingDevice(null);
    setIsModalOpen(true);
  };

  const openEditModal = (device: Device) => {
    setEditingDevice(device);
    setIsModalOpen(true);
  };

  const closeModal = () => {
    setIsModalOpen(false);
    setEditingDevice(null);
  };

  return (
    <div className="container">
      <Header />
      <StatsBar />
      <DeviceGrid onAddDevice={openAddModal} onEditDevice={openEditModal} />
      <DeviceModal
        isOpen={isModalOpen}
        mode={editingDevice ? "edit" : "add"}
        device={editingDevice || undefined}
        onClose={closeModal}
      />
    </div>
  );
}

export default function App() {
  return (
    <ToastProvider>
      <AppContent />
    </ToastProvider>
  );
}
