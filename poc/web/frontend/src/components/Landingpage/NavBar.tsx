import { FaBars } from 'react-icons/fa6';

type NavBarProps = {
  onOpenAuth: () => void;
};

export const NavBar = ({ onOpenAuth }: NavBarProps) => {
  return (
    <div className="fixed top-6 left-0 right-0 z-50 flex justify-center px-4 pointer-events-none">
      <nav className="w-full max-w-5xl bg-neutral-900 rounded-full h-16 flex items-center justify-between px-6 border border-neutral-800 shadow-xl pointer-events-auto">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 bg-amber-500 rounded-sm transform rotate-45 flex items-center justify-center">
            <div className="w-2 h-2 bg-neutral-900 transform -rotate-45" />
          </div>
          <span className="text-xl font-black text-white uppercase tracking-tighter">Zap Arena</span>
        </div>

        <div className="hidden md:flex items-center gap-8">
          {['Home', 'Intel', 'Community', 'Status'].map((link) => (
            <a key={link} href={`#${link.toLowerCase()}`} className="text-xs font-bold text-neutral-400 hover:text-amber-500 uppercase tracking-widest transition-colors">
              {link}
            </a>
          ))}
        </div>

        <button
          onClick={onOpenAuth}
          className="hidden md:block px-6 py-2 bg-amber-500 text-neutral-950 rounded-full font-black uppercase tracking-widest text-xs hover:bg-amber-400 transition-colors"
        >
          Play Alpha
        </button>

        <button className="md:hidden text-neutral-400 hover:text-white">
          <FaBars className="w-6 h-6" />
        </button>
      </nav>
    </div>
  );
};
