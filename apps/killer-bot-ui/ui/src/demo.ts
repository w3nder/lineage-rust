import type { ChatLine, Command, Frame, Nearby, World } from "./model";

function lcg(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1664525 + 1013904223) % 4294967296;
    return state / 4294967296;
  };
}

const NAMES = ["Lorel", "Helvetia", "Bandellos", "Randolf", "Pepper", "Sandra", "Gabrielle", "Macken", "Efryn", "Parman", "Kenfaz", "Taurin", "Alize", "Meiby", "Tamara", "Splendid", "Gradwin", "Lupin"];
const START: [number, number, number] = [83246, 148056, -3408];

function nearby(): Nearby[] {
  const random = lcg(7);
  const out: Nearby[] = [];
  let id = 1000;
  const add = (kind: Nearby["kind"], count: number, spread: number, names: boolean) => {
    for (let index = 0; index < count; index++) {
      const angle = random() * Math.PI * 2;
      const radius = Math.sqrt(random()) * spread;
      const x = START[0] + Math.cos(angle) * radius;
      const y = START[1] + Math.sin(angle) * radius;
      out.push({
        id: id++,
        name: names ? NAMES[Math.floor(random() * NAMES.length)] : kind === "monster" ? "Caught Gigantic Eye" : "Priest of Dawn",
        title: "",
        npcId: kind === "npc" || kind === "monster" ? 30000 + id : 0,
        itemId: kind === "item" ? 57 : 0,
        count: kind === "item" ? 100 : 0,
        addName: "",
        className: kind === "player" ? "Fortune Seeker" : "",
        clanId: 0,
        clan: "",
        kind,
        x,
        y,
        z: START[2],
        distance: radius,
        dead: false,
      });
    }
  };
  add("player", 60, 1400, true);
  add("npc", 18, 2000, false);
  add("monster", 1, 1800, false);
  add("item", 5, 1500, false);
  return out;
}

function chat(): ChatLine[] {
  return [
    { at: 1761148080, channel: "trade", sender: "AnnunciO", text: "3 AM+10 = 2 DKH" },
    { at: 1761148140, channel: "local", sender: "JmPlayerOf", text: "EU ANAZINHA BOPE BASE MAESTRO FOR PM" },
    { at: 1761148140, channel: "trade", sender: "ArochaJxAnuncio", text: "SB+15FC EN CANTIDAD = L+20/H+20/BOS+20 OU (PIX-NEQUI-BINANCE-ZELLE) SOLO RESPONDO WhatsApp" },
    { at: 1761148200, channel: "bot", sender: "Bot", text: "Rota de farm pausada. Aguardando comando." },
    { at: 1761148200, channel: "system", sender: "", text: "Você ganhou 1 840 de experiência." },
  ];
}

export function demoFrame(): Frame {
  const screen = (typeof window !== "undefined" ? new URLSearchParams(window.location.search).get("screen") : null) as Frame["screen"] | null;
  return {
    screen: screen ?? "world",
    sessions: [
      { id: 1, label: "unk", state: "offline" },
      { id: 2, label: "unk_buffs", state: "offline" },
      { id: 3, label: "Wender", state: "world" },
    ],
    selected: 3,
    status: "",
    saved: [{ label: "conta1", account: "conta1", character: "", host: "server.l2killer.com", port: 2106, hasPassword: true, online: false }],
    dataStatus: "",
    dataReady: true,
    servers: [{ id: 1, name: "Killer 70x", address: "192.0.2.10", players: 312, maxPlayers: 2000, online: true }],
    characters: [{ slot: 0, name: "Wender", className: "Fortune Seeker", level: 78 }, { slot: 1, name: "Luzal", className: "Spellsinger", level: 40 }],
    world: screen && screen !== "world" ? null : demoWorld(),
  };
}

function demoWorld(): World {
  return {
    packets: { recording: false, total: 0, shown: 0, rows: [], selected: null, detail: "", status: "...", filters: { server: [], client: [], whitelist: false }, filterOn: false, hideServer: false, hideClient: false, converter: [], help: "" },
    script: typeof window !== "undefined" && new URLSearchParams(window.location.search).has("script") ? { tasks: [{ handler: false, line: 3, waitLeftMs: 7200 }], kind: "waiting", status: "Esperando", paused: false, message: "rodando", variables: [], files: [], loaded: null, log: ["[00:00:01] script iniciado", "[00:00:01] › use_skill 1323", "[00:00:01] Blessing enviado"] } : { tasks: [], kind: "none", status: "Sem script", paused: false, message: "", variables: [], files: [], loaded: null, log: [] },
    options: { attack: { active_attack: true, move_range: 150 }, follow: { distance: 150, style: "walker" }, combat: [], buffs_heals: [], toggles: [], items: [], auto_shots: { item_id: 0, active: false }, polygon: { points: [], z_range: 1000 }, do_not: { npcs: [], items: [] }, rest: {}, party: {}, pickup: {}, no_mobs: {}, pet: {}, dead: {}, spoil: {}, advanced: {}, targeting: {}, alerts: {}, logout: {}, player_sorting: { priorities: [] } },
    savedOptions: null,
    catalog: { skills: [[56, "Power Shot"], [1204, "Wind Walk"]], items: [[1060, "Lesser Healing Potion"], [1463, "Soulshot: D-grade"]], npcs: [], listedItems: [], classes: [] },
    windows: { shop: null, multisell: null, warehouse: null, learn: null, trade: null, quests: [], store: null, pet: null },
    petActions: [],
    npc: null,
    targetInfo: null,
    inventory: [
      { id: 1, itemId: 57, name: "Adena", addName: "", grade: null, known: true, count: 1521972686, equipped: false, bodyPart: 0 },
      { id: 2, itemId: 1060, name: "Lesser Healing Potion", addName: "", grade: null, known: true, count: 120, equipped: false, bodyPart: 0 },
      { id: 3, itemId: 2368, name: "Elven Long Sword", addName: "", grade: "D", known: true, count: 1, equipped: true, bodyPart: 128 },
      { id: 4, itemId: 2378, name: "Maxkiller Breastplate", addName: "Weapon Master", grade: "S", known: true, count: 1, equipped: true, bodyPart: 64 },
      { id: 5, itemId: 1463, name: "Item #9104", addName: "", grade: null, known: false, count: 800, equipped: false, bodyPart: 0 },
    ],
    skills: [
      { id: 56, level: 3, name: "Power Shot", passive: false, ready: true },
      { id: 1204, level: 1, name: "Wind Walk", passive: false, ready: true },
      { id: 7, level: 1, name: "Weapon Mastery", passive: true, ready: true },
    ],
    buffs: [{ skillId: 1204, level: 2, name: "Wind Walk", seconds: 1260 }, { skillId: 1068, level: 3, name: "Might", seconds: 840 }],
    party: [],
    friends: [],
    ping: 42,
    macros: [{ id: 1, name: "Buff rápido", description: "", acronym: "BF", icon: 3, lines: ["/useskill Blessing of Noblesse", "/delay 2", "/useshortcut 1 2"] }],
    macroRun: null,
    shortcuts: [
      { slot: 0, kind: "skill", id: 1204, itemId: 0, level: 2, name: "Wind Walk", icon: "", count: null, ready: true, acronym: "", shot: false, auto: false },
      { slot: 1, kind: "macro", id: 1, itemId: 0, level: 3, name: "Buff rápido", icon: "", count: null, ready: true, acronym: "BF", shot: false, auto: false },
    ],
    henna: { slots: 3, used: 2, bonuses: [["STR", 3], ["CON", -3]] },
    clan: null,
    clanId: 0,
    loot: [{ source: "Wolf", itemId: 57, name: "Adena", addName: "", count: 312 }],
    gains: { expPerHour: 0, spPerHour: 0, adenaPerHour: 184000, adenaTotal: 521000, badMobs: 2 },
    death: null,
    questions: [],
    botOn: true,
    character: {
      name: "Wender",
      title: "Weeeee",
      className: "Fortune Seeker",
      level: 78,
      hp: { current: 10380, max: 10380 },
      mp: { current: 2225, max: 2225 },
      cp: { current: 3521, max: 3521 },
      xpPercent: 100,
      karma: 0,
      pvp: 12074,
      pk: 0,
      recommendHave: 240,
      recommendLeft: 9,
      adena: 1521972686,
      position: START,
      stats: {
        pAtk: 4937,
        mAtk: 744,
        pDef: 2234,
        mDef: 2410,
        accuracy: 127,
        evasion: 132,
        critical: 217,
        speed: 117,
        atkSpd: 436,
        castSpd: 252,
        str: 49,
        dex: 32,
        con: 39,
        int: 18,
        wit: 12,
        men: 27,
        sp: 2054918772,
        load: 161975,
        maxLoad: 434700,
      },
    },
    target: null,
    nearby: nearby(),
    chat: chat(),
    uptime: "02:41:18",
    deaths: 0,
    adenaPerHour: 184000,
    xpPerHour: 0,
    version: "0.1.0",
  };
}

export function applyDemo(frame: Frame, command: Command | null): Frame {
  if (command === null) return frame;
  if (command.type === "selectSession") return { ...frame, selected: command.id };
  const world = frame.world;
  if (!world) return frame;
  return { ...frame, world: applyWorld(world, command) };
}

function applyWorld(world: World, command: Command): World {
  switch (command.type) {
    case "toggleBot":
      return { ...world, botOn: !world.botOn };
    case "selectTarget": {
      const found = world.nearby.find((entry) => entry.id === command.id);
      if (!found) return world;
      return {
        ...world,
        target: { id: found.id, name: found.name, kind: found.kind, hpPercent: found.kind === "player" ? null : 100, dead: false, distance: Math.round(found.distance) },
      };
    }
    case "moveTo":
      return { ...world, character: { ...world.character, position: [command.x, command.y, command.z] } };
    case "say":
      return { ...world, chat: [...world.chat, { at: Math.floor(Date.now() / 1000), channel: command.channel, sender: world.character.name, text: command.text }] };
    default:
      return world;
  }
}

export function demoStatic(): import("./model").StaticData {
  return {
    actions: [],
    classes: [],
    examples: [{ title: "Farm simples", body: "while true\n  attack\n  wait 1200\nend\n" }],
  };
}
