export type Gauge = { current: number; max: number };

export type Kind = "monster" | "npc" | "player" | "item";

export type Nearby = {
  id: number;
  name: string;
  title: string;
  npcId: number;
  itemId: number;
  count: number;
  addName: string;
  className: string;
  clanId: number;
  clan: string;
  kind: Kind;
  x: number;
  y: number;
  z: number;
  distance: number;
  dead: boolean;
};

export type Target = { id: number; name: string; kind: Kind; hpPercent: number | null; dead: boolean; distance: number };

export type Stats = {
  pAtk: number;
  mAtk: number;
  pDef: number;
  mDef: number;
  accuracy: number;
  evasion: number;
  critical: number;
  speed: number;
  atkSpd: number;
  castSpd: number;
  str: number;
  dex: number;
  con: number;
  int: number;
  wit: number;
  men: number;
  sp: number;
  load: number;
  maxLoad: number;
};

export type Character = {
  name: string;
  title: string;
  className: string;
  level: number;
  hp: Gauge;
  mp: Gauge;
  cp: Gauge;
  xpPercent: number;
  karma: number;
  pvp: number;
  pk: number;
  recommendHave: number;
  recommendLeft: number;
  adena: number;
  position: [number, number, number];
  stats: Stats;
};

export type ChatChannel = "all" | "system" | "bot" | "local" | "trade" | "party" | "clan" | "alliance" | "shout" | "whisper" | "hero";

export type ChatLine = { at: number; channel: ChatChannel; sender: string; text: string };

export type Session = { id: number; label: string; state: "offline" | "connecting" | "world" };

export type ServerEntry = { id: number; name: string; address: string; players: number; maxPlayers: number; online: boolean };

export type CharacterEntry = { slot: number; name: string; className: string; level: number };

export type Screen = "accounts" | "connecting" | "servers" | "characters" | "disconnected" | "world";

export type Item = { id: number; itemId: number; name: string; addName: string; grade: string | null; known: boolean; count: number; equipped: boolean; bodyPart: number };
export type Skill = { id: number; level: number; name: string; passive: boolean; ready: boolean };
export type Buff = { skillId: number; level: number; name: string; seconds: number };
export type Member = { id: number; name: string; hp: Gauge; mp: Gauge };
export type ClanMember = { name: string; level: number; className: string; online: boolean };
export type Clan = { name: string; leader: string; level: number; castleId: number; castle: string | null; hideoutId: number; ally: string; atWar: boolean; members: ClanMember[] };
export type Henna = { slots: number; used: number; bonuses: [string, number][] };
export type Macro = { id: number; name: string; description: string; acronym: string; icon: number; lines: string[] };
export type MacroRun = { id: number; line: number };
export type ShortcutKind = "item" | "skill" | "action" | "macro" | "recipe";
export type Shortcut = { slot: number; kind: ShortcutKind; id: number; itemId: number; level: number; name: string; icon: string; count: number | null; ready: boolean; acronym: string; shot: boolean; auto: boolean };
export type Friend = { name: string; online: boolean };
export type Loot = { source: string; itemId: number; name: string; addName: string; count: number };
export type Gains = { expPerHour: number; spPerHour: number; adenaPerHour: number; adenaTotal: number; badMobs: number };
export type TargetInfo = {
  name: string;
  title: string;
  objectId: number;
  npcId: number | null;
  npcName: string | null;
  attackable: boolean | null;
  hp: Gauge | null;
  dead: boolean;
  distance: number;
  position: [number, number, number];
  clanId: number;
  clan: string | null;
  ally: string | null;
  pvpFlag: number;
  karma: number;
  abnormalMask: string;
  kind: "player" | "npc" | "item";
  level: number;
  className: string | null;
  mp: Gauge | null;
  cp: Gauge | null;
  equipment: Equip[];
  nameColor: string;
  hero: boolean;
  running: boolean;
  sitting: boolean;
  inCombat: boolean;
  store: boolean;
  runSpeed: number;
  pAtkSpeed: number;
  mAtkSpeed: number;
  recommendations: number;
  casting: string | null;
  targetName: string | null;
  itemCount: number | null;
};
export type Equip = { slot: string; itemId: number; name: string; addName: string; grade: string | null; known: boolean };
export type Death = { town: boolean; hideout: boolean; castle: boolean; siegeHq: boolean; fortress: boolean };
export type Question = { index: number; kind: string; text: string };

export type HtmlNode =
  | { t: "text"; text: string }
  | { t: "br" }
  | { t: "font"; color: string | null; children: HtmlNode[] }
  | { t: "link"; action: string | null; children: HtmlNode[] }
  | { t: "center"; children: HtmlNode[] }
  | { t: "table"; width: number | null; rows: { width: number | null; children: HtmlNode[] }[][] }
  | { t: "img"; src: string; width: number; height: number }
  | { t: "button"; value: string; action: string | null; width: number; height: number }
  | { t: "edit"; var: string; width: number; height: number; multi: boolean };

export type Npc = { objectId: number; name: string; title: string; nodes: HtmlNode[] };

export type Entry = { objectId: number; itemId: number; name: string; addName: string; grade: string | null; count: number; enchant: number; price: number; storePrice: number };
export type ShopSide = { listId: number; items: Entry[] };
export type Piece = { itemId: number; name: string; addName: string; grade: string | null; count: number; enchant: number; have: number };
export type Offer = { entryId: number; stackable: boolean; products: Piece[]; ingredients: Piece[]; can: boolean };
export type Windows = {
  shop: { adena: number; buy: ShopSide | null; sell: ShopSide | null } | null;
  multisell: { listId: number; finished: boolean; adena: number; offers: Offer[] } | null;
  warehouse: { deposit: boolean; adena: number; items: Entry[] } | null;
  learn: { mode: number; sp: number; skills: { id: number; name: string; nextLevel: number; spCost: number }[] } | null;
  trade: { partner: string; mine: Entry[]; theirs: Entry[]; tradable: Entry[] } | null;
  quests: { questId: number; name: string; step: string | null; cond: number }[];
  store: { kind: "sell" | "buy" | "manageSell" | "manageBuy"; adena: number; message: string; packageSale: boolean; items: Entry[] } | null;
  pet: {
    name: string;
    level: number;
    hp: [number, number];
    mp: [number, number];
    fed: [number, number];
    exp: [number, number];
    load: [number, number];
    stats: [string, number][];
    items: { id: number; itemId: number; name: string; addName: string; count: number; equipped: boolean }[];
  } | null;
};
export type PetAction = { id: number; name: string; icon: string };

export type Variable = { name: string; value: string; kind: string };
export type ScriptTask = { handler: boolean; line: number; waitLeftMs: number | null };
export type Script = {
  tasks: ScriptTask[];
  kind: "none" | "running" | "waiting" | "finished" | "failed";
  status: string;
  paused: boolean;
  message: string;
  variables: Variable[];
  files: string[];
  loaded: { seq: number; name: string; text: string } | null;
  log: string[];
};
export type GameAction = { id: number; name: string; icon: string; group: number };
export type StaticData = { actions: GameAction[]; classes: [number, string][]; examples: { title: string; body: string }[] };

export type PacketRow = { number: number; client: boolean; time: string; size: number; name: string; opcode: string };
export type PacketFilterEntry = { id: number; name: string };
export type Packets = {
  recording: boolean;
  total: number;
  shown: number;
  rows: PacketRow[];
  selected: number | null;
  detail: string;
  status: string;
  filters: { server: PacketFilterEntry[]; client: PacketFilterEntry[]; whitelist: boolean };
  filterOn: boolean;
  hideServer: boolean;
  hideClient: boolean;
  converter: [string, string][];
  help: string;
};

export type World = {
  packets: Packets;
  script: Script;
  options: Record<string, unknown>;
  savedOptions: Record<string, unknown> | null;
  catalog: { skills: [number, string][]; items: [number, string][]; npcs: [number, string][]; listedItems: [number, string][]; classes: [number, string][] };
  windows: Windows;
  petActions: PetAction[];
  npc: Npc | null;
  targetInfo: TargetInfo | null;
  inventory: Item[];
  skills: Skill[];
  buffs: Buff[];
  party: Member[];
  friends: Friend[];
  henna: Henna | null;
  ping: number | null;
  macros: Macro[];
  macroRun: MacroRun | null;
  shortcuts: Shortcut[];
  clan: Clan | null;
  clanId: number;
  loot: Loot[];
  gains: Gains;
  death: Death | null;
  questions: Question[];
  botOn: boolean;
  character: Character;
  target: Target | null;
  nearby: Nearby[];
  chat: ChatLine[];
  uptime: string;
  deaths: number;
  adenaPerHour: number;
  xpPerHour: number;
  version: string;
};

export type Saved = { label: string; account: string; character: string; host: string; port: number; hasPassword: boolean; online: boolean };

export type Frame = {
  screen: Screen;
  sessions: Session[];
  selected: number | null;
  status: string;
  servers: ServerEntry[];
  characters: CharacterEntry[];
  saved: Saved[];
  dataStatus: string;
  dataReady: boolean;
  world: World | null;
};

export type Command =
  | { type: "toggleBot" }
  | { type: "selectTarget"; id: number }
  | { type: "moveTo"; x: number; y: number; z: number }
  | { type: "say"; channel: ChatChannel; text: string }
  | { type: "selectSession"; id: number }
  | { type: "closeSession"; id: number }
  | { type: "addAccount"; host: string; port: number; account: string; password: string; character: string; remember: boolean }
  | { type: "connectSaved"; label: string }
  | { type: "forgetSaved"; label: string }
  | { type: "retryData" }
  | { type: "useItem"; id: number }
  | { type: "dropItem"; id: number; count: number }
  | { type: "destroyItem"; id: number; count: number }
  | { type: "crystallize"; id: number; count: number }
  | { type: "useSkill"; id: number }
  | { type: "restartPoint"; kind: number }
  | { type: "answer"; index: number; accept: boolean }
  | { type: "requestClan" }
  | { type: "clearStats" }
  | { type: "cancelTarget" }
  | { type: "sit" }
  | { type: "stand" }
  | { type: "restart" }
  | { type: "logout" }
  | { type: "ignoreItem"; itemId: number }
  | { type: "ignoreNpc"; npcId: number }
  | { type: "blacklist"; id: number }
  | { type: "partyInvite"; name: string }
  | { type: "partyLeave" }
  | { type: "partyKick"; name: string }
  | { type: "partyLeader"; name: string }
  | { type: "partyDismiss" }
  | { type: "clanInvite"; id: number }
  | { type: "friendInvite"; name: string }
  | { type: "tradeRequest"; id: number }
  | { type: "quit" }
  | { type: "npcAction"; action: string }
  | { type: "bypass"; command: string }
  | { type: "closeNpc" }
  | { type: "buy"; listId: number; orders: [number, number][] }
  | { type: "sell"; listId: number; orders: [number, number, number][] }
  | { type: "closeShop" }
  | { type: "multiSellChoose"; listId: number; entryId: number; amount: number }
  | { type: "closeMultisell" }
  | { type: "warehouse"; deposit: boolean; orders: [number, number][] }
  | { type: "closeWarehouse" }
  | { type: "learnSkill"; skillId: number; level: number; mode: number }
  | { type: "closeLearn" }
  | { type: "tradeAdd"; objectId: number; count: number }
  | { type: "tradeConfirm" }
  | { type: "tradeCancel" }
  | { type: "requestQuests" }
  | { type: "abortQuest"; questId: number }
  | { type: "storeBuy"; orders: [number, number][] }
  | { type: "storeSell"; orders: [number, number][] }
  | { type: "storeSetSell"; packageSale: boolean; offers: [number, number, number][] }
  | { type: "storeSetBuy"; offers: [number, number, number][] }
  | { type: "storeMessage"; buy: boolean; text: string }
  | { type: "closeStore" }
  | { type: "petGive"; objectId: number; count: number }
  | { type: "petTake"; objectId: number; count: number }
  | { type: "action"; id: number }
  | { type: "talk"; id: number }
  | { type: "requestFriends" }
  | { type: "runMacro"; id: number }
  | { type: "stopMacro" }
  | { type: "useShortcut"; slot: number }
  | { type: "registerShortcut"; slot: number; kind: ShortcutKind; id: number }
  | { type: "deleteShortcut"; slot: number }
  | { type: "toggleAutoShot"; itemId: number; enable: boolean }
  | { type: "saveMacro"; id: number; name: string; acronym: string; description: string; icon: number; lines: string[] }
  | { type: "deleteMacro"; id: number }
  | { type: "deleteFriend"; name: string }
  | { type: "setOptions"; options: Record<string, unknown>; save: boolean }
  | { type: "addPolygonPoint"; x: number; y: number }
  | { type: "runScript"; text: string }
  | { type: "stopScript" }
  | { type: "pauseScript"; paused: boolean }
  | { type: "saveScript"; name: string; text: string }
  | { type: "loadScript"; name: string }
  | { type: "pkRecord"; on: boolean }
  | { type: "pkClear" }
  | { type: "pkSave"; onlyVisible: boolean }
  | { type: "pkLoad" }
  | { type: "pkSelect"; number: number }
  | { type: "pkSearch"; query: string; client: boolean; byName: boolean; direction: "first" | "next" | "prev" }
  | { type: "pkInject"; hex: string; toClient: boolean }
  | { type: "pkParse"; format: string }
  | { type: "pkHelp" }
  | { type: "pkView"; filterOn: boolean; hideServer: boolean; hideClient: boolean }
  | { type: "pkFilters"; server: number[]; client: number[]; whitelist: boolean }
  | { type: "pkFiltersSave" }
  | { type: "pkFiltersLoad" }
  | { type: "pkConvert"; text: string }
  | { type: "chooseServer"; id: number }
  | { type: "chooseCharacter"; slot: number }
  | { type: "requestNewCharacter" }
  | { type: "deleteCharacter"; slot: number }
  | { type: "createCharacter"; name: string; race: number; female: boolean; mage: boolean; hairStyle: number; hairColor: number; face: number };
