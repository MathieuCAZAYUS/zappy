import type { IconType } from 'react-icons';
import { FaBoxesStacked, FaChevronRight, FaCoins, FaGun, FaShieldHalved } from 'react-icons/fa6';

import { INVENTORY, SHOP } from './mockData';
import { RarityBadge } from './OverviewHelpers';

export function LoadoutSection() {
  const equippedItem = INVENTORY.find((item) => item.rarity === 'Equipped') ?? INVENTORY[0];
  const reserveItems = INVENTORY.filter((item) => item.id !== equippedItem.id);

  return (
    <div className="bg-zinc-900 border border-zinc-800 p-5 rounded-[1.75rem] shadow-xl shadow-black/30">
      <div className="flex justify-between items-start gap-5 mb-5">
        <div>
          <p className="text-xs text-amber-400 uppercase tracking-[0.3em] font-black mb-1">Gear</p>
          <h2 className="text-xl font-black text-white uppercase tracking-tight">Loadout</h2>
        </div>
        <div className="flex items-center gap-2 text-zinc-950 font-mono text-sm bg-amber-400 px-4 py-2 rounded-full font-black shadow-lg shadow-black/20">
          <FaCoins className="w-4 h-4" />
          {SHOP.balance}
        </div>
      </div>

      <div className="bg-zinc-950 border border-zinc-800 rounded-[1.5rem] p-5 mb-4">
        <div className="flex items-start justify-between gap-4 mb-8">
          <GearIcon type={equippedItem.type} className="w-12 h-12 rounded-2xl bg-amber-400 text-zinc-950 shadow-lg shadow-black/30" />
          <RarityBadge rarity={equippedItem.rarity} />
        </div>

        <p className="text-[10px] text-zinc-500 uppercase tracking-widest font-black mb-2">Primary Slot</p>
        <h3 className="text-3xl font-black text-white uppercase tracking-tight leading-none">{equippedItem.name}</h3>
        <div className="mt-5 grid grid-cols-2 gap-3">
          <LoadoutMeta label="Class" value={equippedItem.type} />
          <LoadoutMeta label="Condition" value="Ready" />
        </div>
      </div>

      <div className="grid gap-3 mb-5">
        {reserveItems.map((item) => (
          <button
            key={item.id}
            className="group w-full grid grid-cols-[auto_1fr_auto] items-center gap-4 p-4 bg-zinc-950 border border-zinc-800 rounded-[1.25rem] hover:border-zinc-700 transition-colors text-left"
          >
            <GearIcon type={item.type} className="w-10 h-10 rounded-xl bg-zinc-900 border border-zinc-800 text-zinc-400 group-hover:text-amber-400 transition-colors" />
            <div className="min-w-0">
              <p className="text-sm font-black text-zinc-100 uppercase tracking-wide truncate">{item.name}</p>
              <p className="text-xs text-zinc-500 font-mono capitalize">{item.type}</p>
            </div>
            <div className="flex items-center gap-3">
              <RarityBadge rarity={item.rarity} />
              <FaChevronRight className="w-3 h-3 text-zinc-600 group-hover:text-amber-400 transition-colors" />
            </div>
          </button>
        ))}
      </div>

      <button className="w-full py-3.5 bg-zinc-800 hover:bg-zinc-700 text-sm font-black uppercase tracking-widest text-zinc-200 rounded-[1rem] transition-colors border border-zinc-700">
        Manage Loadout
      </button>
    </div>
  );
}

function GearIcon({ type, className }: { type: string; className: string }) {
  const Icon: IconType = type === 'armor' ? FaShieldHalved : type === 'weapon' ? FaGun : FaBoxesStacked;

  return (
    <span className={`${className} flex items-center justify-center shrink-0`}>
      <Icon className="w-5 h-5" />
    </span>
  );
}

function LoadoutMeta({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl px-3 py-2">
      <p className="text-[9px] text-zinc-600 uppercase tracking-widest font-black mb-1">{label}</p>
      <p className="text-xs text-zinc-200 uppercase tracking-wide font-black">{value}</p>
    </div>
  );
}
