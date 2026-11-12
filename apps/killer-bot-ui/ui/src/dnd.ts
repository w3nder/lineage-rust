import type { DragEvent } from "react";
import type { ShortcutKind } from "./model";

const MIME = "application/x-unkbot";

export type Dragged = { kind: ShortcutKind; id: number; from?: number };

export function startDrag(event: DragEvent, payload: Dragged) {
  event.dataTransfer.setData(MIME, JSON.stringify(payload));
  event.dataTransfer.effectAllowed = "copyMove";
}

export function acceptsDrag(event: DragEvent): boolean {
  return event.dataTransfer.types.includes(MIME);
}

export function readDrag(event: DragEvent): Dragged | null {
  try {
    const parsed = JSON.parse(event.dataTransfer.getData(MIME)) as Dragged;
    return typeof parsed.id === "number" && typeof parsed.kind === "string" ? parsed : null;
  } catch {
    return null;
  }
}
