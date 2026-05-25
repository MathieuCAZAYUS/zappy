import { FaBoltLightning, FaClock, FaCrosshairs, FaTrophy } from 'react-icons/fa6';

import type { IconProps } from './types';

export const IconBolt = ({ className }: IconProps) => <FaBoltLightning className={className} />;

export const IconCrosshair = ({ className }: IconProps) => <FaCrosshairs className={className} />;

export const IconTrophy = ({ className }: IconProps) => <FaTrophy className={className} />;

export const IconClock = ({ className }: IconProps) => <FaClock className={className} />;
