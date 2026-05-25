import { PLAYER, SERVER } from './mockData';
import { Pill, ServerRow } from './OverviewHelpers';

export function HeroOverviewSection() {
  const xpPercentage = (PLAYER.xp / PLAYER.xpMax) * 100;

  return (
    <section className="rounded-[2.5rem] p-5 md:p-7 shadow-2xl shadow-black/40">
        <div className="grid grid-cols-1 lg:grid-cols-[1.25fr_0.85fr] gap-5">
        <div className="relative bg-zinc-950 border border-zinc-800 rounded-[2rem] p-6 md:p-8 flex flex-col justify-between min-h-[320px] overflow-hidden">
            <div className="absolute top-0 right-0 -mr-16 -mt-16 w-64 h-64 bg-amber-500/5 blur-[80px] rounded-full pointer-events-none" />
            <div className="relative z-10 flex flex-col sm:flex-row sm:items-center justify-between gap-6">
            <div className="flex items-center gap-5">
                <div className="shrink-0 w-24 h-24 rounded-full bg-zinc-900 border border-zinc-700/50 flex items-center justify-center text-3xl font-black text-zinc-500 shadow-xl overflow-hidden">
                {PLAYER.avatarSrc ? (
                    <img
                    src={PLAYER.avatarSrc}
                    alt={`${PLAYER.username} profile`}
                    className="h-full w-full object-cover object-top"
                    />
                ) : (
                    PLAYER.avatarInitials
                )}
                </div>
                <div className="flex flex-col justify-center">
                <div className="flex items-baseline gap-2 mb-1.5">
                    <h1 className="text-3xl md:text-4xl font-black text-white tracking-tight leading-none">
                    {PLAYER.username}
                    </h1>
                    <span className="text-zinc-500 font-mono text-sm">#{PLAYER.tag}</span>
                </div>
                <div className="flex flex-wrap items-center gap-2 mt-1">
                    <span className="px-2.5 py-1 rounded-md bg-zinc-900 border border-zinc-800 text-[11px] font-bold text-zinc-300 uppercase tracking-wider">
                    {PLAYER.region}
                    </span>
                    <span className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-zinc-900 border border-zinc-800 text-[11px] font-bold text-zinc-300 uppercase tracking-wider">
                    <span className="w-1.5 h-1.5 rounded-full bg-green-500" />
                    {PLAYER.status}
                    </span>
                </div>
                </div>
            </div>

            <div className="flex sm:flex-col gap-3 sm:items-end">
                <div className="flex items-center gap-3 px-3 py-2 rounded-xl bg-zinc-900/80 border border-zinc-800/80">
                <span className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold">Rank</span>
                <span className="text-sm text-amber-400 font-black uppercase tracking-wide">{PLAYER.rank}</span>
                </div>
                <div className="flex items-center gap-3 px-3 py-2 rounded-xl bg-zinc-900/80 border border-zinc-800/80">
                <span className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold">Level</span>
                <span className="text-sm text-white font-black">{PLAYER.level}</span>
                </div>
            </div>
            </div>
        <div className="relative z-10 mt-10 bg-zinc-900/40 border border-zinc-800/50 rounded-2xl p-4 md:p-5">
        <div className="flex justify-between items-end mb-3">
            <div>
            <p className="text-[10px] text-zinc-500 uppercase tracking-widest font-bold mb-1">XP Progress</p>
            <div className="flex items-baseline gap-1.5">
                <span className="text-lg font-black text-white">{PLAYER.xp.toLocaleString()}</span>
                <span className="text-xs text-zinc-500 font-mono">/ {PLAYER.xpMax.toLocaleString()} XP</span>
            </div>
            </div>
            <p className="text-2xl font-black text-amber-400">{Math.round(xpPercentage)}%</p>
        </div>

        <div className="w-full bg-zinc-950 h-2.5 rounded-full overflow-hidden border border-zinc-800/80 shadow-inner">
            <div
            className="h-full bg-gradient-to-r from-amber-500 to-yellow-400 rounded-full relative"
            style={{ width: `${xpPercentage}%` }}
            >
            <div className="absolute top-0 right-0 bottom-0 w-10 bg-gradient-to-r from-transparent to-white/20" />
            </div>
        </div>
        </div>
    </div>


        <div className="bg-yellow-200 text-zinc-950 rounded-[2rem] p-6 md:p-8 flex flex-col justify-between min-h-[320px] shadow-xl shadow-black/30">
          <div className="flex items-start justify-between gap-5">
            <div>
              <p className="text-xs uppercase tracking-[0.35em] font-black text-zinc-800 mb-3">Live Server</p>
              <h2 className="text-3xl font-black uppercase tracking-tight leading-none">{SERVER.name}</h2>
            </div>
            <span className="flex items-center gap-2 text-xs font-black bg-zinc-950 text-emerald-300 px-4 py-2 rounded-full uppercase tracking-wider">
              <span className="w-2 h-2 bg-emerald-300 rounded-full" />
              {SERVER.status}
            </span>
          </div>

          <div className="space-y-3 mt-10">
            <ServerRow label="Active Players" value={SERVER.activePlayers} />
            <ServerRow label="Ping" value={SERVER.ping} />
            <ServerRow label="Build" value={SERVER.build} />
          </div>
        </div>
      </div>
    </section>
  );
}
