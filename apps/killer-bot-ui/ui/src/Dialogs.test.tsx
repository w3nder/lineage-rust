import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { TargetInfoWindow } from "./Dialogs";
import type { TargetInfo } from "./model";

afterEach(cleanup);

const player: TargetInfo = {
  name: "Gatoso",
  title: "",
  objectId: 1306574188,
  npcId: null,
  npcName: null,
  attackable: null,
  hp: { current: 900, max: 1000 },
  dead: false,
  distance: 164,
  position: [46988, 51490, -2976],
  clanId: 50763,
  clan: "Killers",
  ally: null,
  pvpFlag: 0,
  karma: 0,
  abnormalMask: "0x0",
  kind: "player",
  level: 78,
  className: "Phoenix Knight",
  mp: { current: 400, max: 500 },
  cp: { current: 0, max: 0 },
  equipment: [
    { slot: "Right hand", itemId: 6370, name: "Draconic Bow", addName: "", grade: "S", known: true },
    { slot: "Chest", itemId: 9999999, name: "Item #9999999", addName: "", grade: null, known: false },
  ],
  nameColor: "#ffffff",
  hero: false,
  running: true,
  sitting: false,
  inCombat: false,
  store: false,
  runSpeed: 140,
  pAtkSpeed: 415,
  mAtkSpeed: 230,
  recommendations: 0,
  casting: null,
  targetName: null,
  itemCount: null,
};

describe("TargetInfoWindow", () => {
  it("asks for a target when there is none", () => {
    render(<TargetInfoWindow info={null} onClose={() => undefined} />);
    expect(screen.getByText("Nenhum alvo")).toBeTruthy();
  });

  it("shows class, level, equipped item names with grade, and the clan name instead of its id", () => {
    render(<TargetInfoWindow info={player} onClose={() => undefined} />);
    expect(screen.getByText(/Lv 78/)).toBeTruthy();
    expect(screen.getByText(/Phoenix Knight/)).toBeTruthy();
    expect(screen.getByText("Draconic Bow")).toBeTruthy();
    expect(screen.getByText("S")).toBeTruthy();
    expect(screen.getByText("Killers")).toBeTruthy();
    expect(screen.queryByText("#50763")).toBeNull();
  });

  it("falls back to the clan id only while the name is unknown, and marks nameless items", () => {
    render(<TargetInfoWindow info={{ ...player, clan: null }} onClose={() => undefined} />);
    expect(screen.getByText("#50763")).toBeTruthy();
    expect(document.querySelector(".item-name.unknown")).not.toBeNull();
  });

  it("hides the combat rows for a ground item and shows its amount", () => {
    const item: TargetInfo = { ...player, kind: "item", equipment: [], itemCount: 1500, className: null };
    render(<TargetInfoWindow info={item} onClose={() => undefined} />);
    expect(screen.getByText(/^1\s500$/)).toBeTruthy();
    expect(screen.queryByText("Em combate")).toBeNull();
  });
});
