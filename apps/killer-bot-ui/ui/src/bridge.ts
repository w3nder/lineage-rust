import type { Command, Frame, StaticData } from "./model";
import { demoFrame, demoStatic, applyDemo } from "./demo";

export interface Bridge {
  subscribe(listener: (frame: Frame) => void): () => void;
  send(command: Command): void;
  loadStatic(): Promise<StaticData>;
}

function insideTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function tauriBridge(): Bridge {
  return {
    subscribe(listener) {
      let stop: (() => void) | undefined;
      let closed = false;
      import("@tauri-apps/api/event").then(({ listen }) =>
        listen<Frame>("snapshot", (event) => listener(event.payload)).then((unlisten) => {
          if (closed) unlisten();
          else stop = unlisten;
        }),
      );
      import("@tauri-apps/api/core").then(({ invoke }) => invoke<Frame | null>("snapshot").then((frame) => frame && listener(frame)));
      return () => {
        closed = true;
        stop?.();
      };
    },
    send(command) {
      import("@tauri-apps/api/core").then(({ invoke }) => (command.type === "quit" ? invoke("quit") : invoke("command", { command })));
    },
    loadStatic() {
      return import("@tauri-apps/api/core").then(({ invoke }) => invoke<StaticData>("static_data"));
    },
  };
}

function demoBridge(): Bridge {
  let current = demoFrame();
  const listeners = new Set<(frame: Frame) => void>();
  const publish = () => listeners.forEach((listener) => listener(current));
  setInterval(() => {
    current = applyDemo(current, null);
    publish();
  }, 1000);
  return {
    subscribe(listener) {
      listeners.add(listener);
      listener(current);
      return () => listeners.delete(listener);
    },
    send(command) {
      current = applyDemo(current, command);
      publish();
    },
    loadStatic() {
      return Promise.resolve(demoStatic());
    },
  };
}

export const bridge: Bridge = insideTauri() ? tauriBridge() : demoBridge();
