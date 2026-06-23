import { createContext, useContext, type Accessor } from "solid-js";
import type {
  AppNotification,
  AppSettings,
  NotificationCaptureStatus,
  NotificationListenerStatus,
  NotificationSoundPreset,
  OverlayMonitorOption,
} from "./types";

export type MainAppContextValue = {
  listenerStatus: Accessor<NotificationListenerStatus | undefined>;
  settings: Accessor<AppSettings | undefined>;
  settingsError: Accessor<string | undefined>;
  notificationError: Accessor<string | undefined>;
  notificationStatus: Accessor<string | undefined>;
  notificationSoundPresets: Accessor<NotificationSoundPreset[]>;
  overlayMonitors: Accessor<OverlayMonitorOption[]>;
  notifications: Accessor<AppNotification[]>;
  captureStatus: Accessor<NotificationCaptureStatus | undefined>;
  pushToast: (tone: string) => Promise<void>;
  pushDemoNotification: () => Promise<void>;
  dismissNotification: (id: string) => Promise<void>;
  clearNotificationHistory: () => Promise<void>;
  updateSettings: (patch: Partial<AppSettings>) => void;
  updateCapsLockSettings: (patch: Partial<AppSettings["capsLockLanguageSwitch"]>) => void;
};

export const MainAppContext = createContext<MainAppContextValue>();

export function useMainApp() {
  const context = useContext(MainAppContext);
  if (!context) {
    throw new Error("MainApp context is missing.");
  }
  return context;
}
