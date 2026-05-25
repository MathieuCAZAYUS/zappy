import type React from 'react';

export type Player = {
  username: string;
  tag: string;
  level: number;
  rank: string;
  xp: number;
  xpMax: number;
  status: string;
  region: string;
  avatarInitials: string;
  avatarSrc?: string;
};

export type Stats = {
  matchesPlayed: number;
  wins: number;
  eliminations: number;
  winRate: string;
  currentStreak: number;
  playTime: string;
};

export type Server = {
  name: string;
  status: string;
  activePlayers: number;
  ping: string;
  build: string;
};

export type RecentMatch = {
  id: number;
  result: string;
  map: string;
  eliminations: number;
  duration: string;
};

export type InventoryItem = {
  id: number;
  name: string;
  rarity: string;
  type: string;
};

export type Shop = {
  balance: number;
  featuredItem: string;
  price: number;
};

export type IconProps = {
  className?: string;
};

export type StatIcon = React.ReactNode;
