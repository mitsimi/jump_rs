import { useState } from "react";
import { Header } from "./components/Header";
import { DeviceGrid } from "./components/DeviceGrid";
import { DeviceModal } from "./components/DeviceModal";
import { ImportExportModal } from "./components/ImportExportModal";
import { LoginScreen } from "./components/LoginScreen";
import { ToastProvider } from "./components/Toast";
import { AuthProvider } from "./context/AuthContext";
import { useAuth } from "./hooks/useAuth";
import type { Device } from "./types/device";

function LoadingScreen() {
  return (
    <div
      className="loading-screen"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <div className="loading-spinner" aria-hidden="true" />
      <span className="sr-only">Loading…</span>
    </div>
  );
}

function AppContent() {
  const { isAuthenticated, isLoading, isAuthRequired } = useAuth();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingDevice, setEditingDevice] = useState<Device | null>(null);
  const [isImportExportOpen, setIsImportExportOpen] = useState(false);

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

  if (isLoading) {
    return <LoadingScreen />;
  }

  if (isAuthRequired && !isAuthenticated) {
    return <LoginScreen />;
  }

  return (
    <div className="container">
      <Header onImportExport={() => setIsImportExportOpen(true)} />
      <DeviceGrid onAddDevice={openAddModal} onEditDevice={openEditModal} />
      <DeviceModal
        isOpen={isModalOpen}
        mode={editingDevice ? "edit" : "add"}
        device={editingDevice || undefined}
        onClose={closeModal}
      />
      <ImportExportModal
        isOpen={isImportExportOpen}
        onClose={() => setIsImportExportOpen(false)}
      />
    </div>
  );
}

export default function App() {
  return (
    <AuthProvider>
      <ToastProvider>
        <AppContent />
      </ToastProvider>
    </AuthProvider>
  );
}
