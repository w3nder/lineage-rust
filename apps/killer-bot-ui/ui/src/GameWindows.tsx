import { useState } from "react";
import { grouped } from "./format";
import { useT } from "./i18n";
import { icons } from "./icons";
import { ItemName } from "./ItemName";
import type { Command, Entry, Offer, PetAction, World } from "./model";
import { WindowFrame } from "./Windows";

type Send = (command: Command) => void;

function amount(text: string, limit: number): number {
  const value = Math.max(0, Math.trunc(Number(text.trim()) || 0));
  return limit > 0 ? Math.min(value, limit) : value;
}

function Img({ src }: { src: string }) {
  const [broken, setBroken] = useState(false);
  return broken ? <span className="icon blank" /> : <img className="icon" src={src} alt="" loading="lazy" onError={() => setBroken(true)} />;
}

function Qty({ value, onChange, placeholder = "0" }: { value: string; onChange: (value: string) => void; placeholder?: string }) {
  return <input className="qty num" inputMode="numeric" value={value} placeholder={placeholder} onChange={(event) => onChange(event.target.value.replace(/\D/g, ""))} />;
}

function Line({ children, right }: { children: React.ReactNode; right?: React.ReactNode }) {
  return (
    <div className="row item" style={{ gridTemplateColumns: right === undefined ? "1fr" : "1fr auto" }}>
      <span className="cell">{children}</span>
      {right !== undefined && <span className="r">{right}</span>}
    </div>
  );
}

export function Shop({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const shop = world.windows.shop;
  const [sellTab, setSellTab] = useState(false);
  const [qty, setQty] = useState<Record<number, string>>({});
  if (!shop) return null;
  const showSell = (sellTab && shop.sell !== null) || shop.buy === null;
  const side = showSell ? shop.sell : shop.buy;
  if (!side) return null;
  const rows = side.items.map((item) => {
    const key = showSell ? item.objectId : item.itemId;
    const limit = showSell ? item.count : item.count > 0 ? item.count : 0;
    return { item, key, count: amount(qty[key] ?? "", limit) };
  });
  const picked = rows.filter((row) => row.count > 0);
  const total = picked.reduce((sum, row) => sum + row.item.price * row.count, 0);
  const close = () => {
    setQty({});
    send({ type: "closeShop" });
  };
  const confirm = () => {
    if (showSell) send({ type: "sell", listId: side.listId, orders: picked.map((row) => [row.item.objectId, row.item.itemId, row.count]) });
    else send({ type: "buy", listId: side.listId, orders: picked.map((row) => [row.item.itemId, row.count]) });
    close();
  };
  return (
    <WindowFrame
      title={t("Loja")}
      width={500}
      onClose={close}
      footer={
        <>
          <span className="grow dim">
            {showSell ? t("Você recebe") : t("Total")} <b className="num" style={{ color: "var(--warn)" }}>{grouped(total)}</b> Adena
          </span>
          <button type="button" className="btn primary" disabled={picked.length === 0} onClick={confirm}>{showSell ? t("Vender") : t("Comprar")}</button>
        </>
      }
    >
      <div className="head-line">
        <span>Adena <b className="num">{grouped(shop.adena)}</b></span>
        {shop.buy && shop.sell && (
          <div className="subtabs" style={{ width: 200 }}>
            <button type="button" className={`ptab${!showSell ? " on" : ""}`} onClick={() => setSellTab(false)}>{t("Comprar")}</button>
            <button type="button" className={`ptab${showSell ? " on" : ""}`} onClick={() => setSellTab(true)}>{t("Vender")}</button>
          </div>
        )}
      </div>
      <div className="table">
        {rows.map(({ item, key }) => (
          <div key={key} className="row item" style={{ gridTemplateColumns: "1fr auto 72px" }}>
            <span className="cell">
              <Img src={icons.item(item.itemId)} />
              <span className="cut"><ItemName item={item} /></span>
              {item.count > 1 && <span className="dim num">({grouped(item.count)})</span>}
            </span>
            <span className="num r dim">{grouped(item.price)}</span>
            <Qty value={qty[key] ?? ""} onChange={(value) => setQty({ ...qty, [key]: value })} />
          </div>
        ))}
      </div>
    </WindowFrame>
  );
}

function Piece({ piece }: { piece: { name: string; addName: string; grade: string | null; enchant: number; count: number } }) {
  return (
    <>
      <ItemName item={piece} />
      {piece.count > 1 && <span className="dim num"> ×{grouped(piece.count)}</span>}
    </>
  );
}

export function Multisell({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const state = world.windows.multisell;
  const [filter, setFilter] = useState("");
  const [amounts, setAmounts] = useState<Record<number, string>>({});
  if (!state) return null;
  const needle = filter.trim().toLowerCase();
  const offers = state.offers.filter((offer: Offer) => needle === "" || offer.products.some((product) => product.name.toLowerCase().includes(needle)));
  return (
    <WindowFrame title={t("Troca")} width={560} onClose={() => send({ type: "closeMultisell" })}>
      <div className="head-line">
        <span>Adena <b className="num">{grouped(state.adena)}</b></span>
        <input className="say" style={{ maxWidth: 220, height: 32 }} placeholder={t("filtrar")} value={filter} onChange={(event) => setFilter(event.target.value)} />
        {!state.finished && <span className="dim">{t("carregando…")}</span>}
      </div>
      {offers.length === 0 && <div className="empty">{t("Nada por aqui")}</div>}
      {offers.map((offer) => (
        <div key={offer.entryId} className="offer">
          <div className="offer-title">{offer.products.map((product, index) => <span key={index}>{index > 0 && " + "}<Piece piece={product} /></span>)}</div>
          {offer.ingredients.map((ingredient, index) => (
            <div key={index} className="offer-line" style={{ color: ingredient.have >= ingredient.count ? undefined : "var(--bad)" }}>
              <Img src={icons.item(ingredient.itemId)} />
              <span className="cut"><Piece piece={ingredient} /></span>
              <span className="num dim">({grouped(ingredient.have)})</span>
            </div>
          ))}
          <div className="offer-actions">
            {offer.stackable && <Qty value={amounts[offer.entryId] ?? ""} placeholder="1" onChange={(value) => setAmounts({ ...amounts, [offer.entryId]: value })} />}
            <button
              type="button"
              className="btn primary"
              disabled={!offer.can}
              onClick={() => send({ type: "multiSellChoose", listId: state.listId, entryId: offer.entryId, amount: offer.stackable ? Math.max(1, amount(amounts[offer.entryId] ?? "", 0)) : 1 })}
            >
              {t("Trocar")}
            </button>
          </div>
        </div>
      ))}
    </WindowFrame>
  );
}

export function Warehouse({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const state = world.windows.warehouse;
  const [qty, setQty] = useState<Record<number, string>>({});
  if (!state) return null;
  const orders: [number, number][] = state.items
    .map((item): [number, number] => [item.objectId, item.count > 1 ? amount(qty[item.objectId] ?? "", item.count) : Number((qty[item.objectId] ?? "") === "1")])
    .filter(([, count]) => count > 0);
  const close = () => {
    setQty({});
    send({ type: "closeWarehouse" });
  };
  return (
    <WindowFrame
      title={state.deposit ? t("Armazém: guardar") : t("Armazém: retirar")}
      width={480}
      onClose={close}
      footer={
        <button
          type="button"
          className="btn primary"
          disabled={orders.length === 0}
          onClick={() => {
            send({ type: "warehouse", deposit: state.deposit, orders });
            close();
          }}
        >
          {state.deposit ? t("Guardar") : t("Retirar")}
        </button>
      }
    >
      <div className="head-line"><span>Adena <b className="num">{grouped(state.adena)}</b></span></div>
      <div className="table">
        {state.items.map((item: Entry) => (
          <div key={item.objectId} className="row item" style={{ gridTemplateColumns: "1fr 72px" }}>
            <span className="cell">
              <Img src={icons.item(item.itemId)} />
              <span className="cut"><ItemName item={item} /></span>
              {item.count > 1 && <span className="dim num">({grouped(item.count)})</span>}
            </span>
            <Qty value={qty[item.objectId] ?? ""} placeholder={item.count > 1 ? "0" : "1"} onChange={(value) => setQty({ ...qty, [item.objectId]: value })} />
          </div>
        ))}
      </div>
    </WindowFrame>
  );
}

export function Learn({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const state = world.windows.learn;
  if (!state) return null;
  return (
    <WindowFrame title={t("Aprender skills")} width={480} onClose={() => send({ type: "closeLearn" })}>
      <div className="head-line"><span>SP <b className="num">{grouped(state.sp)}</b></span></div>
      {state.skills.length === 0 && <div className="empty">{t("Nenhuma skill disponível agora.")}</div>}
      <div className="table">
        {state.skills.map((skill) => (
          <div key={skill.id} className="row item" style={{ gridTemplateColumns: "1fr auto auto" }}>
            <span className="cell">
              <Img src={icons.skill(skill.id, skill.nextLevel)} />
              <span className="cut">{skill.name}</span>
              <span className="dim num">Lv {skill.nextLevel}</span>
            </span>
            <span className="num r" style={{ color: skill.spCost > state.sp ? "var(--bad)" : undefined }}>{grouped(skill.spCost)} SP</span>
            <button type="button" className="btn" onClick={() => send({ type: "learnSkill", skillId: skill.id, level: skill.nextLevel, mode: state.mode })}>{t("Aprender")}</button>
          </div>
        ))}
      </div>
    </WindowFrame>
  );
}

export function Trade({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const state = world.windows.trade;
  const [qty, setQty] = useState<Record<number, string>>({});
  if (!state) return null;
  const list = (items: Entry[]) =>
    items.length === 0 ? (
      <div className="empty" style={{ padding: 12 }}>—</div>
    ) : (
      items.map((item, index) => (
        <Line key={index} right={<span className="num">{grouped(item.count)}</span>}>
          <Img src={icons.item(item.itemId)} />
          <span className="cut"><ItemName item={item} /></span>
        </Line>
      ))
    );
  return (
    <WindowFrame
      title={`${t("Troca com")} ${state.partner}`}
      width={600}
      onClose={() => send({ type: "tradeCancel" })}
      footer={
        <>
          <button type="button" className="btn" onClick={() => send({ type: "tradeCancel" })}>{t("Cancelar")}</button>
          <button type="button" className="btn primary" onClick={() => send({ type: "tradeConfirm" })}>{t("Confirmar")}</button>
        </>
      }
    >
      <div className="two">
        <div>
          <div className="lbl">{t("Seus itens")}</div>
          {list(state.mine)}
        </div>
        <div>
          <div className="lbl">{t("Itens dele(a)")}</div>
          {list(state.theirs)}
        </div>
      </div>
      <div className="lbl">{t("Adicionar item")}</div>
      <div className="table" style={{ maxHeight: 200, overflow: "auto" }}>
        {state.tradable.map((item) => (
          <div key={item.objectId} className="row item" style={{ gridTemplateColumns: "1fr 72px auto" }}>
            <span className="cell">
              <Img src={icons.item(item.itemId)} />
              <span className="cut"><ItemName item={item} /></span>
              {item.count > 1 && <span className="dim num">({grouped(item.count)})</span>}
            </span>
            <Qty value={qty[item.objectId] ?? ""} placeholder={item.count > 1 ? "0" : "1"} onChange={(value) => setQty({ ...qty, [item.objectId]: value })} />
            <button
              type="button"
              className="btn"
              onClick={() => {
                const count = item.count > 1 ? amount(qty[item.objectId] ?? "", item.count) : 1;
                if (count > 0) {
                  send({ type: "tradeAdd", objectId: item.objectId, count });
                  setQty({ ...qty, [item.objectId]: "" });
                }
              }}
            >
              {t("Adicionar")}
            </button>
          </div>
        ))}
      </div>
    </WindowFrame>
  );
}

export function Store({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const store = world.windows.store;
  const [qty, setQty] = useState<Record<number, string>>({});
  const [prices, setPrices] = useState<Record<number, string>>({});
  const [message, setMessage] = useState("");
  const [pack, setPack] = useState(false);
  if (!store) return null;
  const manage = store.kind === "manageSell" || store.kind === "manageBuy";
  const buying = store.kind === "manageBuy" || store.kind === "buy";
  const keyOf = (item: Entry) => (store.kind === "manageBuy" ? item.itemId : item.objectId);
  const close = () => {
    setQty({});
    setPrices({});
    send({ type: "closeStore" });
  };
  const title = { manageSell: t("Minha loja (vender)"), manageBuy: t("Minha loja (comprar)"), sell: t("Loja privada: comprar"), buy: t("Loja privada: vender") }[store.kind];
  const wanted = (item: Entry) => {
    const typed = amount(qty[keyOf(item)] ?? "", store.kind === "manageBuy" ? 0 : item.count);
    return store.kind === "sell" && store.packageSale ? item.count : typed;
  };
  const picked = store.items.map((item) => ({ item, count: wanted(item) })).filter((row) => row.count > 0);
  const total = picked.reduce((sum, row) => sum + row.item.price * row.count, 0);
  const priceOf = (item: Entry) => amount(prices[keyOf(item)] ?? String(item.storePrice || ""), 0);
  const confirm = () => {
    if (store.kind === "manageSell") {
      if (message.trim()) send({ type: "storeMessage", buy: false, text: message.trim() });
      send({ type: "storeSetSell", packageSale: pack, offers: picked.map((row) => [row.item.objectId, row.count, priceOf(row.item)]) });
    } else if (store.kind === "manageBuy") {
      if (message.trim()) send({ type: "storeMessage", buy: true, text: message.trim() });
      send({ type: "storeSetBuy", offers: picked.map((row) => [row.item.itemId, row.count, priceOf(row.item)]) });
    } else if (store.kind === "sell") send({ type: "storeBuy", orders: picked.map((row) => [row.item.objectId, row.count]) });
    else send({ type: "storeSell", orders: picked.map((row) => [row.item.objectId, row.count]) });
    close();
  };
  return (
    <WindowFrame
      title={title}
      width={520}
      onClose={close}
      footer={
        <>
          {!manage && <span className="grow dim">{t("Total")} <b className="num" style={{ color: "var(--warn)" }}>{grouped(total)}</b></span>}
          <button type="button" className="btn primary" disabled={picked.length === 0} onClick={confirm}>
            {manage ? t("Abrir loja") : buying ? t("Vender") : t("Comprar")}
          </button>
        </>
      }
    >
      <div className="head-line"><span>Adena <b className="num">{grouped(store.adena)}</b></span></div>
      {store.message && <div className="npc-title">{store.message}</div>}
      <div className="table">
        {store.items.map((item) => (
          <div key={keyOf(item)} className="row item" style={{ gridTemplateColumns: manage ? "1fr 72px 92px" : "1fr auto 72px" }}>
            <span className="cell">
              <Img src={icons.item(item.itemId)} />
              <span className="cut"><ItemName item={item} /></span>
              {item.count > 1 && <span className="dim num">({grouped(item.count)})</span>}
            </span>
            {manage ? (
              <>
                <Qty value={qty[keyOf(item)] ?? ""} onChange={(value) => setQty({ ...qty, [keyOf(item)]: value })} />
                <Qty value={prices[keyOf(item)] ?? String(item.storePrice || "")} placeholder={t("preço")} onChange={(value) => setPrices({ ...prices, [keyOf(item)]: value })} />
              </>
            ) : (
              <>
                <span className="num r dim">{grouped(item.price)}</span>
                <Qty value={store.kind === "sell" && store.packageSale ? String(item.count) : (qty[keyOf(item)] ?? "")} onChange={(value) => setQty({ ...qty, [keyOf(item)]: value })} />
              </>
            )}
          </div>
        ))}
      </div>
      {manage && (
        <>
          {store.kind === "manageSell" && (
            <label className="check"><input type="checkbox" checked={pack} onChange={(event) => setPack(event.target.checked)} />{t("Venda em pacote")}</label>
          )}
          <input className="say" style={{ height: 34 }} placeholder={t("Mensagem da loja")} value={message} onChange={(event) => setMessage(event.target.value)} />
        </>
      )}
    </WindowFrame>
  );
}

function bar(label: string, [current, max]: [number, number], color: string) {
  return (
    <div key={label} className="pet-bar">
      <span className="lbl">{label}</span>
      <div className="track"><div className="fill" style={{ width: `${max > 0 ? Math.min(100, (current / max) * 100) : 0}%`, background: color }} /></div>
      <span className="num dim">{grouped(current)}/{grouped(max)}</span>
    </div>
  );
}

export function Pet({ world, send, onClose }: { world: World; send: Send; onClose: () => void }) {
  const t = useT();
  const pet = world.windows.pet;
  const [give, setGive] = useState<number | null>(null);
  const [take, setTake] = useState<number | null>(null);
  const [giveQty, setGiveQty] = useState("");
  const [takeQty, setTakeQty] = useState("");
  const mine = world.inventory.filter((item) => !item.equipped && item.itemId !== 57);
  const pick = (text: string, available: number) => {
    const value = Math.trunc(Number(text.trim()) || 0);
    return value > 0 ? Math.min(value, available) : available;
  };
  return (
    <WindowFrame title="Pet" width={520} onClose={onClose}>
      {!pet && <div className="empty">{t("Nenhum pet ou summon invocado.")}</div>}
      {pet && (
        <>
          <div className="who"><b>{pet.name}</b><div>Lv <span className="num">{pet.level}</span></div></div>
          <div className="pet-bars">
            {bar("HP", pet.hp, "var(--bad)")}
            {bar("MP", pet.mp, "var(--blue)")}
            {bar("Food", pet.fed, "var(--warn)")}
            {bar("XP", pet.exp, "var(--good)")}
            {bar("Load", pet.load, "var(--muted)")}
          </div>
          <div className="grid2">
            {pet.stats.map(([label, value]) => (
              <div key={label} className="stat"><span>{label}</span><span className="num">{grouped(value)}</span></div>
            ))}
          </div>
          <div className="slots">
            {world.petActions.map((action: PetAction) => (
              <button key={action.id} type="button" className="slot" title={action.name} onClick={() => send({ type: "action", id: action.id })}>
                <img src={icons.texture(action.icon)} alt={action.name} />
              </button>
            ))}
          </div>
          <div className="two">
            <div>
              <div className="lbl">{t("Meu inventário")}</div>
              <div className="table" style={{ maxHeight: 150, overflow: "auto" }}>
                {mine.map((item) => (
                  <div key={item.id} className={`row item${give === item.id ? " on" : ""}`} onClick={() => setGive(item.id)}>
                    <span className="cell"><Img src={icons.item(item.itemId)} /><span className="cut"><ItemName item={item} /></span><span className="num dim">×{grouped(item.count)}</span></span>
                  </div>
                ))}
              </div>
              <div className="inline-actions">
                <Qty value={giveQty} onChange={setGiveQty} placeholder={t("qtd")} />
                <button type="button" className="btn" disabled={give === null} onClick={() => {
                  const item = mine.find((entry) => entry.id === give);
                  if (item) send({ type: "petGive", objectId: item.id, count: pick(giveQty, item.count) });
                }}>{t("Dar")}</button>
              </div>
            </div>
            <div>
              <div className="lbl">{t("Inventário do pet")}</div>
              <div className="table" style={{ maxHeight: 150, overflow: "auto" }}>
                {pet.items.map((item) => (
                  <div key={item.id} className={`row item${take === item.id ? " on" : ""}`} onClick={() => setTake(item.id)}>
                    <span className="cell"><Img src={icons.item(item.itemId)} /><span className="cut"><ItemName item={item} /></span><span className="num dim">×{grouped(item.count)}</span></span>
                  </div>
                ))}
              </div>
              <div className="inline-actions">
                <Qty value={takeQty} onChange={setTakeQty} placeholder={t("qtd")} />
                <button type="button" className="btn" disabled={take === null} onClick={() => {
                  const item = pet.items.find((entry) => entry.id === take);
                  if (item) send({ type: "petTake", objectId: item.id, count: pick(takeQty, item.count) });
                }}>{t("Pegar")}</button>
              </div>
            </div>
          </div>
        </>
      )}
    </WindowFrame>
  );
}
