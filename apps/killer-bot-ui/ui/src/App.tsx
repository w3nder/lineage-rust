import { useEffect, useRef, useState } from "react";
import { Banners } from "./Banners";
import { bridge } from "./bridge";
import { About, Setup, TargetInfoWindow } from "./Dialogs";
import { SPLASH_MIN_MS, Splash } from "./Splash";
import { Hotbar } from "./Hotbar";
import { MacroWindow } from "./MacroWindow";
import { AccountForm, Characters, Servers, Waiting } from "./Entry";
import { LangContext, load, store, useT, type Lang } from "./i18n";
import { EMPTY, StaticContext } from "./staticData";
import { MapView, type Filters } from "./MapView";
import { ActionsWindow } from "./ActionsWindow";
import { Converter, PacketFilters, PacketWindow } from "./Packets";
import { Debugger, ScriptEditor, ScriptState } from "./Scripts";
import { Learn, Multisell, Pet, Shop, Store, Trade, Warehouse } from "./GameWindows";
import { Menu, type Action } from "./Menu";
import { OptionsWindow } from "./Options";
import { NpcWindow } from "./Npc";
import type { Command, Frame, StaticData } from "./model";
import { Chat, Rail, Side, Top } from "./Parts";

type Window = "about" | "setup" | "targetInfo" | "pet" | "options" | "actions" | "macros" | "scriptEditor" | "debugger" | "packets" | "packetFilters" | "converter";

export function App() {
  const [lang, setLang] = useState<Lang>(() => load<Lang>("unkbot.lang", "pt"));
  const [theme, setTheme] = useState<"dark" | "light">(() => load("unkbot.theme", "dark"));
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.lang = lang === "pt" ? "pt-BR" : "en";
    store("unkbot.theme", theme);
    store("unkbot.lang", lang);
  }, [theme, lang]);
  return (
    <LangContext.Provider value={lang}>
      <Shell lang={lang} theme={theme} setLang={setLang} setTheme={setTheme} />
    </LangContext.Provider>
  );
}

function Shell({ lang, theme, setLang, setTheme }: { lang: Lang; theme: "dark" | "light"; setLang: (lang: Lang) => void; setTheme: (theme: "dark" | "light") => void }) {
  const t = useT();
  const [frame, setFrame] = useState<Frame | null>(null);
  const [adding, setAdding] = useState(false);
  const [filters, setFilters] = useState<Filters>({ monster: true, npc: true, player: true, item: true });
  const [options, setOptions] = useState({ map: true, polygon: false });
  const [zRange, setZRange] = useState(500);
  const [open, setOpen] = useState<Window[]>([]);
  const [statics, setStatics] = useState<StaticData>(EMPTY);

  const [splashDone, setSplashDone] = useState(false);
  const started = useRef(Date.now());

  useEffect(() => bridge.subscribe(setFrame), []);
  const hasFrame = frame !== null;
  useEffect(() => {
    if (!hasFrame) return;
    const wait = Math.max(0, SPLASH_MIN_MS - (Date.now() - started.current));
    const timer = window.setTimeout(() => setSplashDone(true), wait);
    return () => window.clearTimeout(timer);
  }, [hasFrame]);
  const ready = frame?.dataReady ?? false;
  useEffect(() => {
    bridge.loadStatic().then(setStatics).catch(() => undefined);
  }, [ready]);

  const send = (command: Command) => bridge.send(command);
  const show = (window: Window) => setOpen((current) => (current.includes(window) ? current : [...current, window]));
  const hide = (window: Window) => setOpen((current) => current.filter((entry) => entry !== window));

  const act = (action: Action) => {
    const world = frame?.world ?? null;
    switch (action) {
      case "toggleBot":
        return send({ type: "toggleBot" });
      case "cancelTarget":
        return send({ type: "cancelTarget" });
      case "blacklistTarget":
        return world?.target && send({ type: "blacklist", id: world.target.id });
      case "targetInfo":
        return show("targetInfo");
      case "restart":
        return send({ type: "restart" });
      case "logout":
        return send({ type: "logout" });
      case "disconnect":
        return frame?.selected != null && send({ type: "closeSession", id: frame.selected });
      case "exit":
        return send({ type: "quit" });
      case "about":
        return show("about");
      case "setup":
        return show("setup");
      case "pet":
        return show("pet");
      case "options":
        return show("options");
      case "actions":
        return show("actions");
      case "macros":
        return show("macros");
      case "scriptEditor":
      case "setScript":
        return show("scriptEditor");
      case "scriptDebugger":
        return show("debugger");
      case "stopScript":
        return send({ type: "stopScript" });
      case "packets":
        return show("packets");
      case "startScript": {
        const stored = load<string>("unkbot.script.text", "");
        return stored.trim() ? send({ type: "runScript", text: stored }) : show("scriptEditor");
      }
      case "language":
        return setLang(lang === "pt" ? "en" : "pt");
      case "updates":
        return send({ type: "retryData" });
      default:
        return undefined;
    }
  };

  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (!event.altKey || event.ctrlKey || event.metaKey) return;
      if (event.code === "KeyB") {
        event.preventDefault();
        send({ type: "toggleBot" });
      }
      if (event.code === "KeyS") {
        event.preventDefault();
        send({ type: "stopScript" });
      }
      if (event.code === "KeyR") {
        event.preventDefault();
        const stored = load<string>("unkbot.script.text", "");
        if (stored.trim()) send({ type: "runScript", text: stored });
        else show("scriptEditor");
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  if (!frame) return <Splash leaving={false} />;
  const world = frame.world;
  const showForm = adding || frame.screen === "accounts";

  const body = () => {
    if (showForm) return <AccountForm status={frame.status} saved={frame.saved} send={send} onDone={() => setAdding(false)} />;
    if (frame.screen === "servers") return <Servers frame={frame} send={send} />;
    if (frame.screen === "characters") return <Characters frame={frame} send={send} />;
    if (frame.screen !== "world" || !world) return <Waiting frame={frame} send={send} />;
    return (
      <div className="work">
        <Rail world={world} send={send} />
        <main className="center" style={{ position: "relative" }}>
          <Banners world={world} send={send} />
          <Menu onAction={act} />
          <MapView
            world={world}
            filters={filters}
            onFilters={setFilters}
            options={options}
            onOptions={setOptions}
            zRange={zRange}
            onZRange={setZRange}
            onSelect={(id) => send({ type: "selectTarget", id })}
            onInfo={() => show("targetInfo")}
            onMove={(x, y, z) => send({ type: "moveTo", x, y, z })}
            send={send}
          />
          <Hotbar world={world} send={send} />
          <Chat lines={world.chat} send={send} />
        </main>
        <Side world={world} send={send} onInfo={() => show("targetInfo")} />
      </div>
    );
  };

  return (
    <StaticContext.Provider value={statics}>
    <div className="app">
      <Top
        frame={frame}
        theme={theme}
        lang={lang}
        onTheme={() => setTheme(theme === "dark" ? "light" : "dark")}
        onLang={() => setLang(lang === "pt" ? "en" : "pt")}
        send={(command) => {
          if (command.type === "selectSession") setAdding(false);
          send(command);
        }}
        onAdd={() => setAdding(true)}
        onSetup={() => show("setup")}
      />
      {body()}
      <footer className="foot">
        <span className="live">
          <span className="dot" style={{ background: world ? "var(--good)" : "var(--ghost)" }} />
          {world ? t("No mundo") : t("Fora do mundo")}
        </span>
        {world && world.script.kind !== "none" && <ScriptState script={world.script} send={send} />}
        {frame.dataStatus && <span>{frame.dataStatus}</span>}
        <span className="spacer" />
        {world?.ping != null && <span>Ping <span className="num">{world.ping} ms</span></span>}
        <span>unkbot.com · <span className="num">v{world?.version ?? "0.1.0"}</span></span>
      </footer>
      {world?.npc && <NpcWindow key={world.npc.objectId} npc={world.npc} send={send} />}
      {world && (
        <>
          <Shop world={world} send={send} />
          <Multisell world={world} send={send} />
          <Warehouse world={world} send={send} />
          <Learn world={world} send={send} />
          <Trade world={world} send={send} />
          <Store world={world} send={send} />
          {open.includes("macros") && <MacroWindow world={world} send={send} onClose={() => hide("macros")} />}
          {open.includes("actions") && <ActionsWindow send={send} onClose={() => hide("actions")} />}
          {open.includes("scriptEditor") && <ScriptEditor world={world} send={send} onClose={() => hide("scriptEditor")} />}
          {open.includes("debugger") && <Debugger world={world} send={send} onClose={() => hide("debugger")} />}
          {open.includes("packets") && <PacketWindow world={world} send={send} onClose={() => hide("packets")} onFilters={() => show("packetFilters")} onConverter={() => show("converter")} />}
          {open.includes("packetFilters") && <PacketFilters world={world} send={send} onClose={() => hide("packetFilters")} />}
          {open.includes("converter") && <Converter world={world} send={send} onClose={() => hide("converter")} />}
          {open.includes("options") && <OptionsWindow key="options" world={world} send={send} onClose={() => hide("options")} />}
          {open.includes("pet") && <Pet world={world} send={send} onClose={() => hide("pet")} />}
        </>
      )}
      {open.includes("about") && <About onClose={() => hide("about")} version={world?.version ?? "0.1.0"} />}
      {open.includes("setup") && <Setup frame={frame} lang={lang} theme={theme} onLang={setLang} onTheme={setTheme} onCheck={() => send({ type: "retryData" })} onClose={() => hide("setup")} />}
      {open.includes("targetInfo") && <TargetInfoWindow info={world?.targetInfo ?? null} onClose={() => hide("targetInfo")} />}
    </div>
    {!splashDone && <Splash leaving />}
    </StaticContext.Provider>
  );
}
