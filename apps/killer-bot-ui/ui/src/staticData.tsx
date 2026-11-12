import { createContext, useContext } from "react";
import type { StaticData } from "./model";

export const EMPTY: StaticData = { actions: [], classes: [], examples: [] };

export const StaticContext = createContext<StaticData>(EMPTY);

export function useStatic(): StaticData {
  return useContext(StaticContext);
}
