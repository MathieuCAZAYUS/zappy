import type { StatIcon } from './types';

export function Pill({ label, value, strong = false }: { label: string; value: string | number; strong?: boolean }) {
  return (
    <div className={`${strong ? 'bg-rose-700 text-zinc-950' : 'bg-zinc-900 text-white border border-zinc-800'} rounded-full px-5 py-3 min-w-[130px] text-center shadow-lg shadow-black/20`}>
      <p className={`${strong ? 'text-zinc-800' : 'text-zinc-500'} text-[10px] uppercase tracking-widest font-black mb-0.5`}>{label}</p>
      <p className="text-sm font-black uppercase tracking-tight">{value}</p>
    </div>
  );
}

export function ServerRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="flex justify-between items-center gap-4 bg-zinc-950 text-zinc-100 rounded-full px-5 py-4 shadow-lg shadow-black/20">
      <span className="text-xs uppercase tracking-widest font-black text-zinc-500">{label}</span>
      <span className="font-mono text-sm font-black text-white">{value}</span>
    </div>
  );
}

export function StatPill({
  label,
  value,
  icon,
  highlight = false,
}: {
  label: string;
  value: string | number;
  icon?: StatIcon;
  highlight?: boolean;
}) {
  return (
    <div className={`${highlight ? 'bg-amber-400 text-zinc-950' : 'bg-zinc-950 text-white border border-zinc-800'} rounded-full px-5 py-4 flex items-center justify-between gap-4 min-h-[82px] shadow-lg shadow-black/20`}>
      <div>
        <p className={`${highlight ? 'text-zinc-800' : 'text-zinc-500'} text-[10px] uppercase tracking-widest font-black mb-1`}>{label}</p>
        <p className="text-2xl font-black tracking-tight">{value}</p>
      </div>
      {icon && <span className={`${highlight ? 'text-zinc-800' : 'text-zinc-600'} shrink-0`}>{icon}</span>}
    </div>
  );
}

export function RarityBadge({ rarity }: { rarity: string }) {
  const classes =
    rarity === 'Equipped'
      ? 'bg-amber-400 text-zinc-950'
      : rarity === 'Rare'
        ? 'bg-blue-400 text-zinc-950'
        : 'bg-zinc-800 text-zinc-400';

  return (
    <span className={`text-[10px] uppercase tracking-wider font-black px-3 py-1.5 rounded-full shrink-0 ${classes}`}>
      {rarity}
    </span>
  );
}
