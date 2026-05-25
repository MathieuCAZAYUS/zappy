import type React from 'react';

import { IconClock, IconCrosshair, IconTrophy } from './icons';
import { STATS } from './mockData';

export function StatsOverviewSection() {
  return (
    <section className="grid grid-cols-1 xl:grid-cols-[0.9fr_1.6fr] gap-4">
      <div className="relative overflow-hidden bg-amber-400 text-zinc-950 border border-amber-300 rounded-[1.75rem] p-6 md:p-7 shadow-xl shadow-black/30">
        <div className="absolute right-5 top-5 text-zinc-950/15">
          <IconTrophy className="w-24 h-24" />
        </div>

        <div className="relative">
          <p className="text-xs uppercase tracking-[0.35em] font-black text-zinc-800 mb-3">Combat Rating</p>
          <div className="flex items-end justify-between gap-6">
            <div>
              <p className="text-6xl md:text-7xl font-black tracking-tight leading-none">{STATS.winRate}</p>
              <p className="mt-3 text-sm font-black uppercase tracking-widest text-zinc-800">
                {STATS.wins} wins from {STATS.matchesPlayed} matches
              </p>
            </div>
            <div className="hidden sm:block text-right">
              <p className="text-[10px] uppercase tracking-widest font-black text-zinc-800 mb-1">Current Streak</p>
              <p className="text-4xl font-black leading-none">{STATS.currentStreak}</p>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard label="Matches" value={STATS.matchesPlayed} detail="Total played" />
        <StatCard label="Wins" value={STATS.wins} detail="Ranked victories" icon={<IconTrophy className="w-5 h-5" />} />
        <StatCard label="Elims" value={STATS.eliminations} detail="Arena takedowns" icon={<IconCrosshair className="w-5 h-5" />} />
        <StatCard label="Play Time" value={STATS.playTime} detail="Season total" icon={<IconClock className="w-5 h-5" />} />
      </div>
    </section>
  );
}

function StatCard({
  label,
  value,
  detail,
  icon,
}: {
  label: string;
  value: string | number;
  detail: string;
  icon?: React.ReactNode;
}) {
  return (
    <div className="min-h-[150px] bg-zinc-900 border border-zinc-800 rounded-[1.5rem] p-5 flex flex-col justify-between shadow-xl shadow-black/25 hover:border-zinc-700 transition-colors">
      <div className="flex items-center justify-between gap-3">
        <p className="text-[10px] text-zinc-500 uppercase tracking-widest font-black">{label}</p>
        {icon && (
          <span className="w-9 h-9 rounded-full bg-zinc-950 border border-zinc-800 text-amber-400 flex items-center justify-center">
            {icon}
          </span>
        )}
      </div>

      <div>
        <p className="text-3xl md:text-4xl font-black text-white tracking-tight leading-none">{value}</p>
        <p className="mt-2 text-xs text-zinc-500 font-mono">{detail}</p>
      </div>
    </div>
  );
}
