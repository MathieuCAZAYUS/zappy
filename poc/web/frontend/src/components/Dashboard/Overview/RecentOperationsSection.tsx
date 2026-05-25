import { RECENT_MATCHES } from './mockData';

export function RecentOperationsSection() {
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden shadow-xl shadow-black/30">
      <div className="px-5 py-4 border-b border-zinc-800 flex justify-between items-center bg-zinc-900">
        <div>
          <p className="text-xs text-amber-400 uppercase tracking-[0.3em] font-black mb-1">History</p>
          <h2 className="text-xl font-black text-white uppercase tracking-tight">Recent Operations</h2>
        </div>
        <button className="text-xs text-zinc-950 bg-amber-400 hover:bg-amber-300 active:bg-amber-500 px-3.5 py-2 rounded-lg uppercase tracking-wider font-black transition-colors duration-200">
          View All
        </button>
      </div>

      <div className="p-3 space-y-2">
        {RECENT_MATCHES.map((match, index) => {
          const isWin = match.result === 'Victory';
          const resultStyles = isWin
            ? 'bg-emerald-500/12 text-emerald-300 border-emerald-500/40 group-hover:bg-emerald-500/18 group-hover:border-emerald-400/70'
            : 'bg-rose-500/12 text-rose-300 border-rose-500/40 group-hover:bg-rose-500/18 group-hover:border-rose-400/70';
          const railStyles = isWin ? 'bg-emerald-400/80' : 'bg-rose-400/80';
          const hoverTint = isWin ? 'group-hover:bg-emerald-500/[0.04]' : 'group-hover:bg-rose-500/[0.04]';
          const hoverBorder = isWin ? 'hover:border-emerald-500/35' : 'hover:border-rose-500/35';

          return (
            <button
              key={match.id}
              className={`group relative w-full overflow-hidden bg-zinc-950 border border-zinc-800 rounded-xl text-left transition-[background-color,border-color,transform] duration-250 ease-out hover:-translate-y-px ${hoverBorder} ${hoverTint} focus:outline-none focus:ring-2 focus:ring-amber-400/70`}
              style={{ animation: `rise 0.45s ease-out ${index * 70}ms both` }}
            >
              <span className={`absolute inset-y-0 left-0 w-1 ${railStyles} transition-[width,opacity] duration-250 ease-out group-hover:w-1.5 group-hover:opacity-100`} />

              <div className="relative grid grid-cols-[auto_1fr_auto] items-center gap-3 p-3.5 pl-5">
                <div className={`w-12 h-9 rounded-md border flex items-center justify-center text-[10px] font-black uppercase tracking-widest transition-[background-color,border-color,color] duration-250 ease-out ${resultStyles}`}>
                  {isWin ? 'Win' : 'Loss'}
                </div>

                <div className="min-w-0">
                  <p className="text-sm font-black text-white uppercase tracking-wide truncate transition-colors duration-200 ease-in-out group-hover:text-zinc-100">
                    {match.result}
                  </p>
                  <p className="text-xs text-zinc-500 font-mono truncate">{match.map}</p>
                </div>

                <div className="text-right shrink-0">
                  <p className="text-sm font-black text-zinc-100">{match.eliminations} Elims</p>
                  <p className="text-xs text-zinc-500 font-mono">{match.duration}</p>
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
