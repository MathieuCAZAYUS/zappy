import type React from 'react';

export type ServerStatus = {
  status: 'Online' | 'Offline';
  players: number;
  region: string;
  build: string;
};

export type Feature = {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
};

export type NewsItem = {
  id: string;
  date: string;
  title: string;
  excerpt: string;
};
