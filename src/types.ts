export type ToolId =
  | "persistent-notifications"
  | "eye-rest"
  | "caps-lock-language-switch"
  | "current-language-indicator"
  | "settings";

export type AppNotification = {
  id: string;
  title: string;
  body: string;
  source: string;
  sourceAppUserModelId?: string;
  origin: "windows" | "demo";
  createdAt: string;
  tone: string;
  silent?: boolean;
};

export type NotificationDeliveryStatus = {
  ok: boolean;
  message: string;
};

export type PreviewNotificationResult = {
  notification: AppNotification;
  nativeNotification: NotificationDeliveryStatus;
  overlay: NotificationDeliveryStatus;
};

export type NotificationListenerStatus = {
  feasible: boolean;
  api: string;
  permissionRequired: boolean;
  packagingRisk: string;
  prototypeStep: string;
};

export type SettingsStorageStatus = {
  path: string;
  exists: boolean;
  bytes?: number;
  modifiedAt?: string;
  loadOk: boolean;
  message: string;
};

export type CloseBehavior = "minimizeToTray" | "exit";
export type CapsLockFallbackHotkey = "ctrlCaps" | "shiftCaps" | "altCaps";
export type OverlayPlacement =
  | "topLeft"
  | "topCenter"
  | "topRight"
  | "middleLeft"
  | "center"
  | "middleRight"
  | "bottomLeft"
  | "bottomCenter"
  | "bottomRight";

export type NotificationCaptureStatus = {
  enabled: boolean;
  access: string;
  message: string;
  mode: string;
};

export type OverlayMonitorOption = {
  id: string;
  label: string;
  isPrimary: boolean;
};

export type EyeRestStatus = {
  enabled: boolean;
  intervalMinutes: number;
  active: boolean;
  phase: "idle" | "prompt" | "resting" | "done";
  nextDueAt: number;
  restStartedAt?: number;
  restReadyAt?: number;
  now: number;
};

export type InputLanguageInfo = {
  id: string;
  label: string;
  languageCode: string;
  displayCode: string;
  localeName: string;
};

export type CurrentLanguageIndicatorStatus = {
  enabled: boolean;
  mode: CurrentLanguageIndicatorMode;
  current?: InputLanguageInfo;
  installed: InputLanguageInfo[];
  caretAvailable: boolean;
  source: string;
  message: string;
  debug: CurrentLanguageIndicatorDebug;
};

export type CurrentLanguageIndicatorMode = "caretOverlay" | "screenCorner" | "focusedWindowCorner";

export type CurrentLanguageIndicatorSize = "small" | "medium" | "large";

export type CurrentLanguageIndicatorDebug = {
  foregroundWindow: string;
  foregroundThreadId?: number;
  keyboardLayout?: string;
  uiAutomation: string;
  win32Caret: string;
};

export type LanguageIndicatorPayload = {
  code: string;
  label: string;
  localeName: string;
  mode: CurrentLanguageIndicatorMode;
  size: CurrentLanguageIndicatorSize;
  boundsVisible: boolean;
  x: number;
  y: number;
  caretAvailable: boolean;
};

export type AppSettings = {
  runOnStartup: boolean;
  runHighPriority: boolean;
  closeBehavior: CloseBehavior;
  enableTrayIcon: boolean;
  nativeNotificationEnabled: boolean;
  dismissMirroredWindowsNotifications: boolean;
  notificationSoundEnabled: boolean;
  notificationSoundPreset: string;
  notificationOverlayPlacement: OverlayPlacement;
  notificationOverlayMonitor: string;
  notificationOverlayDebugVisible: boolean;
  notificationOverlayBoundsVisible: boolean;
  eyeRestReminder: {
    enabled: boolean;
    intervalMinutes: number;
  };
  capsLockLanguageSwitch: {
    enabled: boolean;
    preserveCapsLockWith: CapsLockFallbackHotkey;
  };
  currentLanguageIndicator: {
    enabled: boolean;
    mode: CurrentLanguageIndicatorMode;
    placement: OverlayPlacement;
    size: CurrentLanguageIndicatorSize;
    boundsVisible: boolean;
  };
};

export type NotificationSoundPreset = {
  id: string;
  label: string;
  file: string;
};
