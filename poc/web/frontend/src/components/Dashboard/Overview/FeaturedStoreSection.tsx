import { SHOP } from './mockData';

export function FeaturedStoreSection() {
  return (
    <div className="mt-15 bg-zinc-900 border border-zinc-800 pt-5 px-4 pb-4 rounded-2xl shadow-xl shadow-black/30">
      <div className="grid grid-cols-1 sm:grid-cols-[1fr_auto] items-center gap-4">
        <div className="bg-zinc-950 border border-zinc-800 rounded-xl px-5 py-4">
          <p className="text-[10px] text-zinc-500 uppercase tracking-[0.3em] font-black mb-1.5">Daily Featured</p>
          <div className="flex flex-wrap items-end gap-x-4 gap-y-1">
            <p className="text-2xl font-black text-amber-400 uppercase italic tracking-tight leading-none">{SHOP.featuredItem}</p>
            <p className="text-xs text-zinc-500 font-mono">Cost: {SHOP.price} &cent;</p>
          </div>
        </div>

        <button className="h-full min-h-[72px] sm:min-w-[170px] px-5 bg-amber-400 hover:bg-amber-300 text-zinc-950 text-xs font-black uppercase tracking-widest rounded-xl transition-colors shadow-lg shadow-black/30">
          Open Market
        </button>
      </div>
    </div>
  );
}
