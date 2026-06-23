import type { OverlayPlacement, ToolId } from "./types";

export const tools: Array<{
  id: ToolId;
  path: string;
  name: string;
  description: string;
  glyph: string;
  status: string;
}> = [
  {
    id: "persistent-notifications",
    path: "/notifications",
    name: "Persistent Notifications",
    description: "Keep Windows notifications visible as desktop cards until dismissed or handled.",
    glyph: "N",
    status: "Prototype",
  },
  {
    id: "eye-rest",
    path: "/eye-rest",
    name: "Eye Rest Reminder",
    description: "Show periodic 20-20-20 reminders for long desktop sessions.",
    glyph: "20",
    status: "Planned",
  },
  {
    id: "caps-lock-language-switch",
    path: "/caps-lock-language-switch",
    name: "Caps Lock Language Switch",
    description: "Use Caps Lock as a quick input-language switch while preserving clear lock behavior.",
    glyph: "C",
    status: "Spike",
  },
  {
    id: "current-language-indicator",
    path: "/current-language-indicator",
    name: "Current Language Indicator",
    description: "Show the active input-language marker near the typing caret when the language changes.",
    glyph: "IL",
    status: "Spike",
  },
  {
    id: "settings",
    path: "/settings",
    name: "Settings",
    description: "Suite-wide startup, privacy, and release controls.",
    glyph: "S",
    status: "Draft",
  },
];

export const overlayPlacements: Array<{
  id: OverlayPlacement;
  label: string;
  shortLabel: string;
}> = [
  { id: "topLeft", label: "Top left", shortLabel: "TL" },
  { id: "topCenter", label: "Top center", shortLabel: "TC" },
  { id: "topRight", label: "Top right", shortLabel: "TR" },
  { id: "middleLeft", label: "Middle left", shortLabel: "ML" },
  { id: "center", label: "Center", shortLabel: "C" },
  { id: "middleRight", label: "Middle right", shortLabel: "MR" },
  { id: "bottomLeft", label: "Bottom left", shortLabel: "BL" },
  { id: "bottomCenter", label: "Bottom center", shortLabel: "BC" },
  { id: "bottomRight", label: "Bottom right", shortLabel: "BR" },
];
