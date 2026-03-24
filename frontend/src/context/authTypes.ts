import { createContext } from "react";

interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  isAuthRequired: boolean;
  username: string | null;
}

export interface AuthContextType extends AuthState {
  login: () => void;
  logout: () => void;
  checkAuth: () => Promise<void>;
}

export const AuthContext = createContext<AuthContextType | null>(null);
