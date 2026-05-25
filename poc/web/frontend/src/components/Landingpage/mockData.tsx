import { BotIcon, CalendarIcon, ShieldIcon, ShoppingCartIcon, UserIcon, UsersIcon } from './icons';
import type { Feature, NewsItem, ServerStatus } from './types';

export const MOCK_SERVER: ServerStatus = {
  status: 'Online',
  players: 128,
  region: 'Europe',
  build: 'Alpha v0.1.4',
};

export const MOCK_NEWS: NewsItem[] = [
  {
    id: '1',
    date: 'OCT 24, 2026',
    title: 'Welcome to Zap Arena',
    excerpt: 'The arena doors are officially open for alpha testing. Grab your gear and jump into the fray.',
  },
  {
    id: '2',
    date: 'OCT 28, 2026',
    title: 'Account System Coming Soon',
    excerpt: 'We are rolling out persistent player profiles. Your stats, loadouts, and history will soon be saved.',
  },
  {
    id: '3',
    date: 'NOV 02, 2026',
    title: 'Community Features Planned',
    excerpt: 'Clans, leaderboards, and scheduled weekend tournaments are in the pipeline. Stay tuned.',
  },
];

export const MOCK_FEATURES: Feature[] = [
  { id: 'f1', title: 'Real-time Multiplayer', description: 'Fluid, low-latency combat built on a robust Rust backend. Every frame counts and every hit registers instantly.', icon: <UsersIcon /> },
  { id: 'f2', title: 'AI Enemies', description: 'Survive against relentless, dynamic AI threats that adapt to your strategies and pressure the arena organically.', icon: <BotIcon /> },
  { id: 'f3', title: 'Player Accounts', description: 'Track your stats, customize your loadout, and build your legacy across seasons with persistent profiles.', icon: <UserIcon /> },
  { id: 'f4', title: 'Community Events', description: 'Join massive weekend brawls, clan wars, and special game modes dictated entirely by the community.', icon: <CalendarIcon /> },
  { id: 'f5', title: 'Future Item Shop', description: 'Customize your gladiator. Earn in-game currency to unlock exclusive cosmetic skins and weapon wraps.', icon: <ShoppingCartIcon /> },
  { id: 'f6', title: 'Fair Play Guard', description: 'Active admin monitoring and strict server-side validation to ensure a completely cheat-free battleground.', icon: <ShieldIcon /> },
];

export const CARD_THEMES = [
  { border: 'hover:border-amber-500', shadow: 'hover:shadow-[0_0_30px_rgba(245,158,11,0.25)]', text: 'text-amber-500', iconBg: 'group-hover:bg-amber-500/10', iconBorder: 'group-hover:border-amber-500/30' },
  { border: 'hover:border-cyan-500', shadow: 'hover:shadow-[0_0_30px_rgba(6,182,212,0.25)]', text: 'text-cyan-500', iconBg: 'group-hover:bg-cyan-500/10', iconBorder: 'group-hover:border-cyan-500/30' },
  { border: 'hover:border-rose-500', shadow: 'hover:shadow-[0_0_30px_rgba(244,63,94,0.25)]', text: 'text-rose-500', iconBg: 'group-hover:bg-rose-500/10', iconBorder: 'group-hover:border-rose-500/30' },
  { border: 'hover:border-emerald-500', shadow: 'hover:shadow-[0_0_30px_rgba(16,185,129,0.25)]', text: 'text-emerald-500', iconBg: 'group-hover:bg-emerald-500/10', iconBorder: 'group-hover:border-emerald-500/30' },
  { border: 'hover:border-purple-500', shadow: 'hover:shadow-[0_0_30px_rgba(168,85,247,0.25)]', text: 'text-purple-500', iconBg: 'group-hover:bg-purple-500/10', iconBorder: 'group-hover:border-purple-500/30' },
  { border: 'hover:border-blue-500', shadow: 'hover:shadow-[0_0_30px_rgba(59,130,246,0.25)]', text: 'text-blue-500', iconBg: 'group-hover:bg-blue-500/10', iconBorder: 'group-hover:border-blue-500/30' },
];
