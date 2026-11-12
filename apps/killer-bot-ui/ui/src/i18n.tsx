import { createContext, useContext } from "react";

export type Lang = "pt" | "en";

const EN: Record<string, string> = {};

export function register(entries: Record<string, string>) {
  Object.assign(EN, entries);
}

export const LangContext = createContext<Lang>("pt");

export function useLang(): Lang {
  return useContext(LangContext);
}

export function useT(): (pt: string) => string {
  const lang = useContext(LangContext);
  return (pt) => (lang === "en" ? (EN[pt] ?? pt) : pt);
}

export function load<T>(key: string, fallback: T): T {
  try {
    const raw = window.localStorage.getItem(key);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

export function store(key: string, value: unknown) {
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    return;
  }
}
