import { IconBolt } from './Overview/icons';
import { PLAYER } from './Overview/mockData';
import { OverviewSections } from './Overview/OverviewSections';

type DashboardProps = {
  onSignOut: () => void;
};

export const Dashboard = ({ onSignOut }: DashboardProps) => {
  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-200 font-sans selection:bg-amber-500 selection:text-zinc-950 overflow-x-hidden">
      <nav className="border-b border-zinc-800 bg-zinc-950">
        <div className="max-w-7xl mx-auto px-4 md:px-8 h-20 flex items-center justify-between">
          <div className="flex items-center gap-8">
            <div className="flex items-center gap-3 text-amber-400 font-black tracking-widest italic text-xl">
              <span className="w-11 h-11 rounded-full bg-amber-400 text-zinc-950 flex items-center justify-center shadow-lg shadow-black/30">
                <IconBolt className="w-6 h-6" />
              </span>
              ZAP//ARENA
            </div>

            <div className="hidden md:flex items-center gap-2 text-xs font-black tracking-widest uppercase bg-zinc-900 border border-zinc-800 rounded-full p-1.5">
              <button className="text-zinc-950 bg-amber-400 px-5 py-2.5 rounded-full transition-colors">Overview</button>
              <button className="text-zinc-400 hover:text-white px-5 py-2.5 rounded-full transition-colors">Inventory</button>
              <button className="text-zinc-400 hover:text-white px-5 py-2.5 rounded-full transition-colors">Matches</button>
              <button className="text-zinc-400 hover:text-white px-5 py-2.5 rounded-full transition-colors">Store</button>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <div className="hidden sm:flex items-center gap-3 px-4 py-2.5 bg-zinc-900 border border-zinc-800 rounded-full">
              <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
              <span className="text-sm font-bold text-white tracking-wide">
                {PLAYER.username}<span className="text-zinc-500 font-normal">{PLAYER.tag}</span>
              </span>
            </div>
            <button
              onClick={onSignOut}
              className="bg-amber-400 hover:bg-amber-300 text-zinc-950 text-sm font-black px-6 py-3 rounded-full transition-colors uppercase tracking-wider shadow-lg shadow-black/30"
            >
              Back to Game
            </button>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto px-4 md:px-8 py-8 space-y-7">
        <OverviewSections />
      </main>
    </div>
  );
};
