import type { InventoryItem, Player, RecentMatch, Server, Shop, Stats } from './types';

export const PLAYER: Player = {
  username: 'Raph',
  tag: '#0043',
  level: 17,
  rank: 'Silver Arena',
  xp: 7420,
  xpMax: 10000,
  status: 'Online',
  region: 'Europe',
  avatarInitials: 'RF',
  avatarSrc: '/ddxu1sx-a83c7f94-3973-4b5b-9fe8-0da619cddef1.png',
};

export const STATS: Stats = {
  matchesPlayed: 86,
  wins: 34,
  eliminations: 412,
  winRate: '39.5%',
  currentStreak: 3,
  playTime: '28h 40m',
};

export const SERVER: Server = {
  name: 'Europe Alpha',
  status: 'Online',
  activePlayers: 128,
  ping: '24ms',
  build: 'Alpha 0.1.0',
};

export const RECENT_MATCHES: RecentMatch[] = [
  { id: 1, result: 'Victory', map: 'Arena Core', eliminations: 12, duration: '08:42' },
  { id: 2, result: 'Defeat', map: 'Rust Yard', eliminations: 5, duration: '06:13' },
  { id: 3, result: 'Victory', map: 'Neon Pit', eliminations: 9, duration: '07:55' },
  { id: 4, result: 'Defeat', map: 'Scrap Zone', eliminations: 3, duration: '04:21' },
  { id: 5, result: 'Victory', map: 'Iron Gate', eliminations: 11, duration: '09:18' },
];

export const INVENTORY: InventoryItem[] = [
  { id: 1, name: 'Rust Blade', rarity: 'Equipped', type: 'weapon' },
  { id: 2, name: 'Pulse Rifle', rarity: 'Rare', type: 'weapon' },
  { id: 3, name: 'Arena Jacket', rarity: 'Common', type: 'armor' },
];

export const SHOP: Shop = {
  balance: 500,
  featuredItem: 'Neon Trail',
  price: 250,
};
