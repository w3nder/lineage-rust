import { useMemo, useState, type ReactNode } from "react";
import { useT } from "./i18n";
import type { Command, World } from "./model";
import { useStatic } from "./staticData";
import { WindowFrame } from "./Windows";

type Json = Record<string, any>;
type Choice = [string, string];

const TRAITS: Choice[] = [
  ["always", "Always"], ["cp", "CP"], ["hp", "HP"], ["mp", "MP"], ["dead", "Dead"], ["charges", "Charges"], ["souls", "Souls"], ["death_penalty", "Death Penalty"],
  ["bleeding", "Bleeding"], ["poisoned", "Poisoned"], ["iced", "Iced"], ["shackled", "Shackled"], ["feared", "Feared"], ["stunned", "Stunned"], ["slept", "Slept"],
  ["muted", "Muted"], ["rooted", "Rooted"], ["paralyzed", "Paralyzed"], ["petrified", "Petrified"], ["ultimate_defense", "Ultimate Defense"],
];

function read(object: Json, path: string): any {
  return path.split(".").reduce((current, key) => (current === undefined || current === null ? undefined : current[key]), object);
}

function write(object: Json, path: string, value: unknown): Json {
  const [head, ...rest] = path.split(".");
  if (rest.length === 0) return { ...object, [head]: value };
  return { ...object, [head]: write(object[head] ?? {}, rest.join("."), value) };
}

type Edit = (path: string, value: unknown) => void;

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <label className="opt-row">
      <span className="opt-label">
        {label}
        {hint && <small className="dim">{hint}</small>}
      </span>
      {children}
    </label>
  );
}

function Bool({ o, path, label, edit, hint }: { o: Json; path: string; label: string; edit: Edit; hint?: string }) {
  return (
    <label className="opt-check">
      <input type="checkbox" checked={Boolean(read(o, path))} onChange={(event) => edit(path, event.target.checked)} />
      <span>
        {label}
        {hint && <small className="dim">{hint}</small>}
      </span>
    </label>
  );
}

function Num({ o, path, label, edit, hint, integer = true }: { o: Json; path: string; label: string; edit: Edit; hint?: string; integer?: boolean }) {
  const value = read(o, path);
  return (
    <Row label={label} hint={hint}>
      <input className="opt-input num" inputMode="decimal" value={value ?? ""} onChange={(event) => {
        const text = event.target.value.replace(integer ? /[^\d-]/g : /[^\d.-]/g, "");
        edit(path, text === "" || text === "-" ? 0 : Number(text));
      }} />
    </Row>
  );
}

function OptNum({ o, path, label, edit, hint }: { o: Json; path: string; label: string; edit: Edit; hint?: string }) {
  const value = read(o, path);
  return (
    <Row label={label} hint={hint}>
      <span className="opt-inline">
        <input type="checkbox" checked={value !== null && value !== undefined} onChange={(event) => edit(path, event.target.checked ? 50 : null)} />
        <input className="opt-input num" disabled={value === null || value === undefined} inputMode="numeric" value={value ?? ""} onChange={(event) => edit(path, Math.min(100, Number(event.target.value.replace(/\D/g, "")) || 0))} />
        <span className="dim">%</span>
      </span>
    </Row>
  );
}

function Txt({ o, path, label, edit, hint }: { o: Json; path: string; label: string; edit: Edit; hint?: string }) {
  return (
    <Row label={label} hint={hint}>
      <input className="opt-input" value={read(o, path) ?? ""} onChange={(event) => edit(path, event.target.value)} />
    </Row>
  );
}

function Pick({ o, path, label, edit, choices }: { o: Json; path: string; label: string; edit: Edit; choices: Choice[] }) {
  return (
    <Row label={label}>
      <select className="opt-input" value={read(o, path) ?? choices[0][0]} onChange={(event) => edit(path, event.target.value)}>
        {choices.map(([value, text]) => (
          <option key={value} value={value}>{text}</option>
        ))}
      </select>
    </Row>
  );
}

function Group({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="opt-group">
      <div className="lbl">{title}</div>
      <div className="opt-fields">{children}</div>
    </section>
  );
}

function Rules({ o, path, edit, make, columns }: { o: Json; path: string; edit: Edit; make: () => Json; columns: (rule: Json, set: (key: string, value: unknown) => void, index: number) => ReactNode }) {
  const t = useT();
  const rules: Json[] = read(o, path) ?? [];
  const change = (next: Json[]) => edit(path, next);
  return (
    <div className="rules">
      {rules.length === 0 && <div className="empty">{t("Nenhuma regra ainda")}</div>}
      {rules.map((rule, index) => (
        <div key={index} className="rule">
          <div className="rule-fields">{columns(rule, (key, value) => change(rules.map((entry, position) => (position === index ? { ...entry, [key]: value } : entry))), index)}</div>
          <div className="rule-tools">
            <button type="button" className="ico small" aria-label={t("Subir")} disabled={index === 0} onClick={() => change(move(rules, index, -1))}>↑</button>
            <button type="button" className="ico small" aria-label={t("Descer")} disabled={index === rules.length - 1} onClick={() => change(move(rules, index, 1))}>↓</button>
            <button type="button" className="ico small" aria-label={t("Remover")} onClick={() => change(rules.filter((_, position) => position !== index))}>×</button>
          </div>
        </div>
      ))}
      <button type="button" className="btn" onClick={() => change([...rules, make()])}>{t("Adicionar")}</button>
    </div>
  );
}

function move<T>(list: T[], index: number, by: number): T[] {
  const next = [...list];
  const target = index + by;
  if (target < 0 || target >= next.length) return next;
  [next[index], next[target]] = [next[target], next[index]];
  return next;
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="rule-field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function NumField({ rule, set, name, label, max }: { rule: Json; set: (key: string, value: unknown) => void; name: string; label: string; max?: number }) {
  return (
    <Field label={label}>
      <input className="opt-input num" value={rule[name] ?? 0} inputMode="numeric" onChange={(event) => set(name, Math.min(max ?? Number.MAX_SAFE_INTEGER, Number(event.target.value.replace(/\D/g, "")) || 0))} />
    </Field>
  );
}

function SelectField({ rule, set, name, label, choices }: { rule: Json; set: (key: string, value: unknown) => void; name: string; label: string; choices: Choice[] }) {
  return (
    <Field label={label}>
      <select className="opt-input" value={rule[name]} onChange={(event) => set(name, event.target.value)}>
        {choices.map(([value, text]) => <option key={value} value={value}>{text}</option>)}
      </select>
    </Field>
  );
}

function Check({ rule, set, name, label }: { rule: Json; set: (key: string, value: unknown) => void; name: string; label: string }) {
  return (
    <label className="rule-field check">
      <input type="checkbox" checked={Boolean(rule[name])} onChange={(event) => set(name, event.target.checked)} />
      <span>{label}</span>
    </label>
  );
}

function IdList({ o, path, edit, label, names }: { o: Json; path: string; edit: Edit; label: string; names: Map<number, string> }) {
  const t = useT();
  const [text, setText] = useState("");
  const ids: number[] = read(o, path) ?? [];
  return (
    <div className="idlist">
      <div className="lbl">{label}</div>
      <div className="idlist-add">
        <input className="opt-input num" inputMode="numeric" placeholder="ID" value={text} onChange={(event) => setText(event.target.value.replace(/\D/g, ""))} />
        <button type="button" className="btn" disabled={text === "" || ids.includes(Number(text))} onClick={() => { edit(path, [...ids, Number(text)]); setText(""); }}>{t("Adicionar")}</button>
      </div>
      <div className="chips-wrap">
        {ids.length === 0 && <span className="dim">{t("Lista vazia")}</span>}
        {ids.map((id) => (
          <span key={id} className="idchip">
            <span className="num">{id}</span>
            {names.get(id) && <span className="dim">{names.get(id)}</span>}
            <button type="button" aria-label={t("Remover")} onClick={() => edit(path, ids.filter((entry) => entry !== id))}>×</button>
          </span>
        ))}
      </div>
    </div>
  );
}

const SECTIONS = ["Ataque", "Combate", "Buffs e curas", "Toggles", "Itens", "Grupo", "Seguir", "Coleta", "Sem monstros", "Pet", "Morte", "Spoil", "Descanso", "Filtros", "Alertas", "Logout", "Polígono", "Jogadores", "Avançado"] as const;

export function OptionsWindow({ world, send, onClose }: { world: World; send: (command: Command) => void; onClose: () => void }) {
  const t = useT();
  const [draft, setDraft] = useState<Json>(() => structuredClone(world.options as Json));
  const [section, setSection] = useState<(typeof SECTIONS)[number]>("Ataque");
  const edit: Edit = (path, value) => setDraft((current) => write(current, path, value));
  const dirty = JSON.stringify(draft) !== JSON.stringify(world.options);
  const skills = useMemo<Choice[]>(() => [["0", "—"], ...world.catalog.skills.map(([id, name]): Choice => [String(id), name])], [world.catalog.skills]);
  const items = useMemo<Choice[]>(() => [["0", "—"], ...world.catalog.items.map(([id, name]): Choice => [String(id), name])], [world.catalog.items]);
  const npcNames = useMemo(() => new Map(world.catalog.npcs), [world.catalog.npcs]);
  const listedItemNames = useMemo(() => new Map([...world.catalog.items, ...world.catalog.listedItems]), [world.catalog.items, world.catalog.listedItems]);
  const classes = useStatic().classes;
  const o = draft;
  const pick = (value: number, choices: Choice[]): Choice[] => (choices.some(([id]) => id === String(value)) ? choices : [...choices, [String(value), `#${value}`]]);

  const body = (): ReactNode => {
    switch (section) {
      case "Ataque":
        return (
          <>
            <Group title={t("Ataque")}>
              <Bool o={o} edit={edit} path="attack.active_attack" label={t("Atacar automaticamente")} />
              <Bool o={o} edit={edit} path="attack.active_target" label={t("Escolher alvos automaticamente")} />
              <Bool o={o} edit={edit} path="attack.cancel_target_on_dead" label={t("Cancelar o alvo ao morrer")} />
              <Bool o={o} edit={edit} path="attack.move_before_attack" label={t("Chegar perto antes de atacar")} />
              <Num o={o} edit={edit} path="attack.move_range" label={t("Distância para atacar")} integer={false} />
              <Bool o={o} edit={edit} path="attack.smart_move_before_attack" label={t("Chegar perto de forma inteligente")} />
              <Bool o={o} edit={edit} path="attack.move_before_targeting" label={t("Andar antes de escolher o alvo")} />
              <Bool o={o} edit={edit} path="attack.auto_unstuck" label={t("Destravar sozinho")} />
              <Bool o={o} edit={edit} path="attack.auto_blacklist" label={t("Pôr na lista negra quem não dá para atacar")} />
              <Num o={o} edit={edit} path="advanced.blacklist_tries" label={t("Tentativas antes da lista negra")} />
            </Group>
            <Group title={t("Escolha do alvo")}>
              <Pick o={o} edit={edit} path="targeting.kill_steal" label={t("Alvos já atacados por outros")} choices={[["dont", t("Não atacar")], ["only", t("Só esses")], ["both", t("Todos")]]} />
              <Pick o={o} edit={edit} path="targeting.in_box" label={t("Dentro do polígono")} choices={[["in", t("Dentro")], ["not_in", t("Fora")], ["both", t("Ambos")]]} />
              <Pick o={o} edit={edit} path="targeting.alive" label={t("Estado")} choices={[["alive", t("Vivos")], ["dead", t("Mortos")], ["both", t("Ambos")]]} />
              <Pick o={o} edit={edit} path="targeting.attackable" label={t("Atacáveis")} choices={[["attackable", t("Atacáveis")], ["invincible", t("Invencíveis")], ["both", t("Ambos")]]} />
              <Pick o={o} edit={edit} path="targeting.kind" label={t("Tipo")} choices={[["npcs", "NPCs"], ["players", t("Jogadores")], ["both", t("Ambos")]]} />
              <Bool o={o} edit={edit} path="targeting.protect_priority" label={t("Proteger primeiro quem está sendo atacado")} />
            </Group>
          </>
        );
      case "Combate":
        return (
          <Rules
            o={o}
            edit={edit}
            path="combat"
            make={() => ({ on: true, page: 1, slot: 1, trait_: "always", comparison: "at_least", percent: 60, delay_ms: 1, mp_above: 100 })}
            columns={(rule, set) => (
              <>
                <Check rule={rule} set={set} name="on" label={t("Ativa")} />
                <NumField rule={rule} set={set} name="page" label={t("Página")} max={10} />
                <NumField rule={rule} set={set} name="slot" label="Slot" max={12} />
                <SelectField rule={rule} set={set} name="trait_" label={t("Condição")} choices={TRAITS} />
                <SelectField rule={rule} set={set} name="comparison" label="" choices={[["at_least", t("pelo menos")], ["at_most", t("no máximo")]]} />
                <NumField rule={rule} set={set} name="percent" label="%" max={100} />
                <NumField rule={rule} set={set} name="delay_ms" label={t("Espera (ms)")} />
                <NumField rule={rule} set={set} name="mp_above" label={t("MP acima de")} />
              </>
            )}
          />
        );
      case "Buffs e curas":
        return (
          <Rules
            o={o}
            edit={edit}
            path="buffs_heals"
            make={() => ({ skill_id: 0, on: true, trait_: "always", names: "", below_percent: 60, delay_seconds: 1, mp_above: 100, need_target: false })}
            columns={(rule, set) => (
              <>
                <Check rule={rule} set={set} name="on" label={t("Ativa")} />
                <Field label="Skill">
                  <select className="opt-input" value={String(rule.skill_id)} onChange={(event) => set("skill_id", Number(event.target.value))}>
                    {pick(rule.skill_id, skills).map(([value, text]) => <option key={value} value={value}>{text}</option>)}
                  </select>
                </Field>
                <SelectField rule={rule} set={set} name="trait_" label={t("Quando")} choices={TRAITS} />
                <NumField rule={rule} set={set} name="below_percent" label={t("Abaixo de %")} max={100} />
                <Field label={t("Nomes (separe por vírgula)")}>
                  <input className="opt-input" value={rule.names} onChange={(event) => set("names", event.target.value)} />
                </Field>
                <NumField rule={rule} set={set} name="delay_seconds" label={t("Espera (s)")} />
                <NumField rule={rule} set={set} name="mp_above" label={t("MP acima de")} />
                <Check rule={rule} set={set} name="need_target" label={t("Precisa de alvo")} />
              </>
            )}
          />
        );
      case "Toggles":
        return (
          <Rules
            o={o}
            edit={edit}
            path="toggles"
            make={() => ({ skill_id: 0, on: true, trait_: "hp", below_percent: 100, above_percent: 100 })}
            columns={(rule, set) => (
              <>
                <Check rule={rule} set={set} name="on" label={t("Ativa")} />
                <Field label="Skill">
                  <select className="opt-input" value={String(rule.skill_id)} onChange={(event) => set("skill_id", Number(event.target.value))}>
                    {pick(rule.skill_id, skills).map(([value, text]) => <option key={value} value={value}>{text}</option>)}
                  </select>
                </Field>
                <SelectField rule={rule} set={set} name="trait_" label={t("Medir")} choices={[["hp", "HP"], ["mp", "MP"]]} />
                <NumField rule={rule} set={set} name="below_percent" label={t("Ligar abaixo de %")} max={100} />
                <NumField rule={rule} set={set} name="above_percent" label={t("Desligar acima de %")} max={100} />
              </>
            )}
          />
        );
      case "Itens":
        return (
          <>
            <Group title={t("Shots automáticos")}>
              <Row label={t("Item")}>
                <select className="opt-input" value={String(read(o, "auto_shots.item_id") ?? 0)} onChange={(event) => edit("auto_shots.item_id", Number(event.target.value))}>
                  {pick(read(o, "auto_shots.item_id") ?? 0, items).map(([value, text]) => <option key={value} value={value}>{text}</option>)}
                </select>
              </Row>
              <Bool o={o} edit={edit} path="auto_shots.active" label={t("Usar automaticamente")} />
            </Group>
            <Rules
              o={o}
              edit={edit}
              path="items"
              make={() => ({ item_id: 0, on: true, trait_: "always", below_percent: 60, delay_ms: 100 })}
              columns={(rule, set) => (
                <>
                  <Check rule={rule} set={set} name="on" label={t("Ativa")} />
                  <Field label="Item">
                    <select className="opt-input" value={String(rule.item_id)} onChange={(event) => set("item_id", Number(event.target.value))}>
                      {pick(rule.item_id, items).map(([value, text]) => <option key={value} value={value}>{text}</option>)}
                    </select>
                  </Field>
                  <SelectField rule={rule} set={set} name="trait_" label={t("Quando")} choices={TRAITS} />
                  <NumField rule={rule} set={set} name="below_percent" label={t("Abaixo de %")} max={100} />
                  <NumField rule={rule} set={set} name="delay_ms" label={t("Espera (ms)")} />
                </>
              )}
            />
          </>
        );
      case "Grupo":
        return (
          <>
            <Group title={t("Convites")}>
              <Bool o={o} edit={edit} path="party.accept_party" label={t("Aceitar convite de grupo")} />
              <Txt o={o} edit={edit} path="party.accept_party_names" label={t("De quem")} hint={t("nomes separados por vírgula")} />
              <Bool o={o} edit={edit} path="party.accept_party_clan" label={t("Aceitar de qualquer um do clã")} />
              <Bool o={o} edit={edit} path="party.accept_party_alliance" label={t("Aceitar de qualquer um da aliança")} />
              <Bool o={o} edit={edit} path="party.auto_invite" label={t("Convidar para o grupo")} />
              <Txt o={o} edit={edit} path="party.auto_invite_names" label={t("Quem convidar")} />
              <Bool o={o} edit={edit} path="party.drop_party_if_leader" label={t("Dissolver se eu for o líder")} />
              <Pick o={o} edit={edit} path="party.loot_type" label={t("Distribuição de drops")} choices={[["finders_keepers", "Finders Keepers"], ["random", "Random"], ["random_including_spoil", "Random including Spoil"], ["by_turn", "By Turn"], ["by_turn_including_spoil", "By Turn including Spoil"]]} />
            </Group>
            <Group title={t("Ressurreição")}>
              <Bool o={o} edit={edit} path="party.accept_rez" label={t("Aceitar ressurreição")} />
              <Txt o={o} edit={edit} path="party.accept_rez_names" label={t("De quem")} />
              <Bool o={o} edit={edit} path="party.accept_rez_party" label={t("Do grupo")} />
              <Bool o={o} edit={edit} path="party.accept_rez_clan" label={t("Do clã")} />
              <Bool o={o} edit={edit} path="party.accept_rez_alliance" label={t("Da aliança")} />
            </Group>
            <Group title={t("Buffs no grupo")}>
              <Num o={o} edit={edit} path="party.buff_range" label={t("Alcance dos buffs")} integer={false} />
              <Bool o={o} edit={edit} path="party.buff_with_control" label={t("Buffar com Ctrl")} />
              <Bool o={o} edit={edit} path="party.buff_with_shift" label={t("Buffar com Shift")} />
              <Bool o={o} edit={edit} path="party.oop_members" label={t("Ajudar quem está fora do grupo")} />
              <Txt o={o} edit={edit} path="party.oop_names" label={t("Quem")} />
            </Group>
          </>
        );
      case "Seguir":
        return (
          <Group title={t("Seguir um jogador")}>
            <Bool o={o} edit={edit} path="follow.active_follow" label={t("Seguir")} />
            <Txt o={o} edit={edit} path="follow.name" label={t("Nome")} />
            <Num o={o} edit={edit} path="follow.distance" label={t("Distância")} integer={false} />
            <Pick o={o} edit={edit} path="follow.style" label={t("Estilo")} choices={[["walker", "Walker"], ["l2_net", "L2Net"]]} />
            <Bool o={o} edit={edit} path="follow.attack" label={t("Atacar o que ele ataca")} />
            <Bool o={o} edit={edit} path="follow.instant_attack" label={t("Atacar na hora")} />
            <Bool o={o} edit={edit} path="follow.target" label={t("Pegar o mesmo alvo")} />
          </Group>
        );
      case "Coleta":
        return (
          <Group title={t("Coletar drops")}>
            <Bool o={o} edit={edit} path="pickup.pickup" label={t("Pegar itens do chão")} />
            <Bool o={o} edit={edit} path="pickup.after_attack" label={t("Só depois do ataque")} />
            <Num o={o} edit={edit} path="pickup.range" label={t("Alcance")} integer={false} />
            <Bool o={o} edit={edit} path="pickup.only_mine" label={t("Só o que é meu")} />
            <Num o={o} edit={edit} path="advanced.pickup_timeout" label={t("Tempo limite (s)")} />
          </Group>
        );
      case "Sem monstros":
        return (
          <Group title={t("Quando não há monstros")}>
            <Bool o={o} edit={edit} path="no_mobs.move_to_loc" label={t("Voltar para o ponto")} />
            <Bool o={o} edit={edit} path="no_mobs.out_of_combat" label={t("Só fora de combate")} />
            <Num o={o} edit={edit} path="no_mobs.leash" label={t("Distância máxima")} integer={false} />
            <Num o={o} edit={edit} path="no_mobs.x" label="X" />
            <Num o={o} edit={edit} path="no_mobs.y" label="Y" />
            <Num o={o} edit={edit} path="no_mobs.z" label="Z" />
            <button type="button" className="btn" onClick={() => { edit("no_mobs.x", world.character.position[0]); edit("no_mobs.y", world.character.position[1]); edit("no_mobs.z", world.character.position[2]); }}>{t("Usar minha posição")}</button>
          </Group>
        );
      case "Pet":
        return (
          <Group title="Pet">
            <Bool o={o} edit={edit} path="pet.auto_assist" label={t("Pet ajuda sozinho")} />
            <Bool o={o} edit={edit} path="pet.solo_attack" label={t("Pet ataca sozinho")} />
            <Bool o={o} edit={edit} path="pet.summon_auto_assist" label={t("Summon ajuda sozinho")} />
            <Bool o={o} edit={edit} path="pet.summon_instant_attack" label={t("Summon ataca na hora")} />
          </Group>
        );
      case "Morte":
        return (
          <Group title={t("Quando morrer")}>
            <Bool o={o} edit={edit} path="dead.toggle_botting" label={t("Desligar o bot")} />
            <Bool o={o} edit={edit} path="dead.return_enabled" label={t("Voltar para uma cidade")} />
            <Num o={o} edit={edit} path="dead.return_delay_seconds" label={t("Espera para voltar (s)")} />
            <Pick o={o} edit={edit} path="dead.return_point" label={t("Ponto de retorno")} choices={[["0", t("Cidade")], ["1", "Clan Hall"], ["2", t("Castelo")], ["3", "Siege HQ"], ["4", t("Fortaleza")]]} />
            <Bool o={o} edit={edit} path="dead.logout_enabled" label={t("Sair do jogo")} />
            <Num o={o} edit={edit} path="dead.logout_delay_seconds" label={t("Espera para sair (s)")} />
          </Group>
        );
      case "Spoil":
        return (
          <Group title="Spoil">
            <Bool o={o} edit={edit} path="spoil.auto_spoil" label={t("Spoil automático")} />
            <Bool o={o} edit={edit} path="spoil.auto_sweep" label={t("Sweep automático")} />
            <Bool o={o} edit={edit} path="spoil.plunder" label="Plunder" />
            <Bool o={o} edit={edit} path="spoil.spoil_crush" label="Spoil Crush" />
            <Bool o={o} edit={edit} path="spoil.until_success" label={t("Repetir até conseguir")} />
            <Num o={o} edit={edit} path="spoil.mp_above" label={t("MP acima de")} />
          </Group>
        );
      case "Descanso":
        return (
          <Group title={t("Sentar para descansar")}>
            <OptNum o={o} edit={edit} path="rest.below_hp" label={t("Sentar com HP abaixo de")} />
            <OptNum o={o} edit={edit} path="rest.below_mp" label={t("Sentar com MP abaixo de")} />
            <OptNum o={o} edit={edit} path="rest.until_hp" label={t("Levantar com HP acima de")} />
            <OptNum o={o} edit={edit} path="rest.until_mp" label={t("Levantar com MP acima de")} />
            <Bool o={o} edit={edit} path="rest.follow_rest" label={t("Descansar quando o líder descansar")} />
            <Txt o={o} edit={edit} path="rest.follow_rest_name" label={t("Líder")} />
          </Group>
        );
      case "Filtros":
        return (
          <>
            <Group title={t("Monstros")}>
              <Bool o={o} edit={edit} path="do_not.attack_only_listed" label={t("Atacar só os da lista")} />
              <Bool o={o} edit={edit} path="do_not.ignore_summons" label={t("Ignorar summons")} />
              <Bool o={o} edit={edit} path="do_not.ignore_chests" label={t("Ignorar baús")} />
              <Bool o={o} edit={edit} path="do_not.ignore_raidbosses" label={t("Ignorar raid bosses")} />
            </Group>
            <IdList o={o} edit={edit} path="do_not.npcs" label={t("NPCs")} names={npcNames} />
            <Group title={t("Itens")}>
              <Bool o={o} edit={edit} path="do_not.pick_only_listed" label={t("Pegar só os da lista")} />
              <Bool o={o} edit={edit} path="do_not.ignore_meshless" label={t("Ignorar itens sem modelo")} />
              <Bool o={o} edit={edit} path="do_not.ignore_unknown_items" label={t("Ignorar itens desconhecidos")} />
            </Group>
            <IdList o={o} edit={edit} path="do_not.items" label={t("Itens")} names={listedItemNames} />
          </>
        );
      case "Alertas":
        return (
          <>
            <Group title={t("Guerra")}>
              <Bool o={o} edit={edit} path="alerts.one_way_war" label={t("Guerra de mão única")} />
              <Bool o={o} edit={edit} path="alerts.two_way_war" label={t("Guerra dos dois lados")} />
              <Bool o={o} edit={edit} path="alerts.minus_one_way_war" label={t("Guerra de mão única contra mim")} />
            </Group>
            <Group title={t("Jogadores e clãs")}>
              <Bool o={o} edit={edit} path="alerts.player" label={t("Jogador por perto")} />
              <Txt o={o} edit={edit} path="alerts.player_names" label={t("Nomes")} />
              <Bool o={o} edit={edit} path="alerts.player_ignore_party" label={t("Ignorar o grupo")} />
              <Bool o={o} edit={edit} path="alerts.clan" label={t("Clã por perto")} />
              <Txt o={o} edit={edit} path="alerts.clan_names" label={t("Clãs")} />
              <Bool o={o} edit={edit} path="alerts.clan_ignore_party" label={t("Ignorar o grupo")} />
            </Group>
            <Group title={t("Vida")}>
              <OptNum o={o} edit={edit} path="alerts.hp_below" label={t("HP abaixo de")} />
              <OptNum o={o} edit={edit} path="alerts.mp_below" label={t("MP abaixo de")} />
              <OptNum o={o} edit={edit} path="alerts.cp_below" label={t("CP abaixo de")} />
            </Group>
            <Group title={t("Mensagens")}>
              <Bool o={o} edit={edit} path="alerts.private_message" label={t("Mensagem privada")} />
              <Bool o={o} edit={edit} path="alerts.white_chat" label={t("Chat geral")} />
              <Bool o={o} edit={edit} path="alerts.friend_chat" label={t("Chat de amigos")} />
            </Group>
          </>
        );
      case "Logout":
        return (
          <>
            <Group title={t("Sair do jogo quando")}>
              <Bool o={o} edit={edit} path="logout.one_way_war" label={t("Guerra de mão única")} />
              <Bool o={o} edit={edit} path="logout.two_way_war" label={t("Guerra dos dois lados")} />
              <Bool o={o} edit={edit} path="logout.minus_one_way_war" label={t("Guerra de mão única contra mim")} />
              <Bool o={o} edit={edit} path="logout.player" label={t("Aparecer um jogador")} />
              <Txt o={o} edit={edit} path="logout.player_names" label={t("Nomes")} />
              <Bool o={o} edit={edit} path="logout.clan" label={t("Aparecer um clã")} />
              <Txt o={o} edit={edit} path="logout.clan_names" label={t("Clãs")} />
            </Group>
            <Group title={t("Vida")}>
              <OptNum o={o} edit={edit} path="logout.hp_below" label={t("HP abaixo de")} />
              <OptNum o={o} edit={edit} path="logout.mp_below" label={t("MP abaixo de")} />
              <OptNum o={o} edit={edit} path="logout.cp_below" label={t("CP abaixo de")} />
            </Group>
          </>
        );
      case "Polígono": {
        const points: { x: number; y: number }[] = read(o, "polygon.points") ?? [];
        const around = (sides: number) => {
          const [cx, cy] = world.character.position;
          const radius = 500;
          edit("polygon.points", Array.from({ length: sides }, (_, index) => {
            const angle = ((45 + (360 * index) / sides) * Math.PI) / 180;
            return { x: cx + Math.round(radius * Math.cos(angle)), y: cy + Math.round(radius * Math.sin(angle)) };
          }));
        };
        return (
          <>
            <Group title={t("Área de caça")}>
              <Num o={o} edit={edit} path="polygon.z_range" label={t("Diferença de altura (Z)")} />
              <div className="opt-inline-buttons">
                <button type="button" className="btn" onClick={() => around(4)}>{t("Quadrado ao meu redor")}</button>
                <button type="button" className="btn" onClick={() => around(6)}>{t("Hexágono ao meu redor")}</button>
                <button type="button" className="btn" onClick={() => edit("polygon.points", [])}>{t("Limpar")}</button>
              </div>
              <p className="hint">{t("Sem pontos (ou com menos de três) o bot caça em qualquer lugar. No mapa, ligue Polígono e clique para adicionar pontos.")}</p>
            </Group>
            <div className="table">
              {points.map((point, index) => (
                <div key={index} className="row item" style={{ gridTemplateColumns: "28px 1fr 1fr auto" }}>
                  <span className="num dim">{index + 1}</span>
                  <input className="opt-input num" value={point.x} onChange={(event) => edit("polygon.points", points.map((entry, position) => (position === index ? { ...entry, x: Number(event.target.value.replace(/[^\d-]/g, "")) || 0 } : entry)))} />
                  <input className="opt-input num" value={point.y} onChange={(event) => edit("polygon.points", points.map((entry, position) => (position === index ? { ...entry, y: Number(event.target.value.replace(/[^\d-]/g, "")) || 0 } : entry)))} />
                  <button type="button" className="ico small" aria-label={t("Remover")} onClick={() => edit("polygon.points", points.filter((_, position) => position !== index))}>×</button>
                </div>
              ))}
              <button type="button" className="btn" onClick={() => edit("polygon.points", [...points, { x: world.character.position[0], y: world.character.position[1] }])}>{t("Adicionar minha posição")}</button>
            </div>
          </>
        );
      }
      case "Jogadores":
        return (
          <>
            <Group title={t("Ordem de alvo entre jogadores")}>
              <Num o={o} edit={edit} path="player_sorting.max_z_difference" label={t("Diferença máxima de altura")} />
              <Num o={o} edit={edit} path="player_sorting.max_distance" label={t("Distância máxima")} />
              <Num o={o} edit={edit} path="player_sorting.equidistant_threshold" label={t("Empate de distância")} />
            </Group>
            <div className="lbl">{t("Prioridade por classe")}</div>
            <Rules
              o={o}
              edit={edit}
              path="player_sorting.priorities"
              make={() => ({ class_id: 0, priority: 1 })}
              columns={(rule, set) => (
                <>
                  <Field label={t("Classe")}>
                    <select className="opt-input" value={String(rule.class_id)} onChange={(event) => set("class_id", Number(event.target.value))}>
                      {(classes.some(([id]) => id === rule.class_id) ? classes : [...classes, [rule.class_id, `#${rule.class_id}`] as [number, string]]).map(([id, className]) => <option key={id} value={String(id)}>{className}</option>)}
                    </select>
                  </Field>
                  <NumField rule={rule} set={set} name="priority" label={t("Prioridade")} />
                </>
              )}
            />
          </>
        );
      case "Avançado":
        return (
          <Group title={t("Avançado")}>
            <Txt o={o} edit={edit} path="advanced.window_title" label={t("Título da janela")} />
            <Num o={o} edit={edit} path="advanced.anti_ks_delay" label={t("Espera anti-KS (s)")} />
            <Num o={o} edit={edit} path="advanced.autofollow_delay" label={t("Espera ao seguir (s)")} />
          </Group>
        );
    }
  };

  return (
    <WindowFrame
      title={t("Opções do bot")}
      width={820}
      onClose={onClose}
      footer={
        <>
          <span className="grow dim">{dirty ? t("Há mudanças que ainda não foram aplicadas") : ""}</span>
          <button type="button" className="btn" disabled={!world.savedOptions} title={t("Trazer as opções guardadas desta conta")} onClick={() => world.savedOptions && setDraft(structuredClone(world.savedOptions as Json))}>{t("Carregar guardadas")}</button>
          <button type="button" className="btn" onClick={() => setDraft(structuredClone(world.options as Json))}>{t("Desfazer")}</button>
          <button type="button" className="btn" onClick={() => send({ type: "setOptions", options: draft, save: true })}>{t("Aplicar e guardar")}</button>
          <button type="button" className="btn primary" disabled={!dirty} onClick={() => send({ type: "setOptions", options: draft, save: false })}>{t("Aplicar")}</button>
        </>
      }
    >
      <div className="opt-layout">
        <nav className="opt-nav" aria-label={t("Seções")}>
          {SECTIONS.map((name) => (
            <button key={name} type="button" className={`ptab${section === name ? " on" : ""}`} onClick={() => setSection(name)}>{t(name)}</button>
          ))}
        </nav>
        <div className="opt-body">{body()}</div>
      </div>
    </WindowFrame>
  );
}
