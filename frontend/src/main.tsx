import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { client } from "./api/generated/client.gen";
import App from "./App.tsx";
import "./styles/index.css";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

client.setConfig({
  baseUrl: "",
  credentials: "include",
});

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30000,
      retry: 1,
    },
  },
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  </StrictMode>,
);
