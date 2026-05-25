import { BotIcon, ShieldIcon, UserIcon, UsersIcon } from './icons';
import { MOCK_SERVER } from './mockData';

type HeroSectionProps = {
  onOpenAuth: () => void;
};

export const HeroSection = ({ onOpenAuth }: HeroSectionProps) => {
  return (
    <section id="home" className="min-h-screen bg-neutral-950 pt-32 pb-20 px-6 flex items-center">
      <div className="max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
        <div className="flex flex-col items-start text-left space-y-8 z-10">
          <h1 className="text-5xl md:text-7xl font-black text-white uppercase tracking-tighter leading-[1.1]">
            Survive The <br />
            <span className="text-amber-500">Zap Arena</span>
          </h1>

          <p className="text-lg md:text-xl text-neutral-400 font-medium max-w-lg leading-relaxed">
            A fast, brutal, multiplayer Rust-powered arena where players fight, survive, and evolve. Solid performance, zero compromises.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 w-full sm:w-auto pt-4">
            <button
              onClick={onOpenAuth}
              className="px-8 py-4 bg-amber-500 text-neutral-950 font-black uppercase tracking-widest rounded-lg hover:bg-amber-400 transition-colors"
            >
              Deploy Now
            </button>
            <button className="px-8 py-4 bg-neutral-900 text-white font-black uppercase tracking-widest rounded-lg border border-neutral-800 hover:border-neutral-600 transition-colors">
              View Intel
            </button>
          </div>
        </div>

        <div className="relative w-full max-w-lg mx-auto lg:max-w-none">
          <div className="bg-neutral-900 border border-neutral-800 rounded-2xl overflow-hidden shadow-2xl flex flex-col h-[500px]">
            <div className="bg-neutral-950 px-4 py-3 flex items-center justify-between border-b border-neutral-800">
              <div className="flex items-center gap-2">
                <div className="w-2.5 h-2.5 rounded-full bg-neutral-700"></div>
                <div className="w-2.5 h-2.5 rounded-full bg-neutral-700"></div>
                <div className="w-2.5 h-2.5 rounded-full bg-neutral-700"></div>
              </div>
              <span className="text-[10px] font-mono text-neutral-500 uppercase tracking-widest">ARENA_SYS_HUD_V1.4</span>
            </div>

            <div className="p-6 flex-1 flex flex-col gap-6 overflow-hidden">
              <div className="bg-neutral-950 rounded-lg p-4 border border-neutral-800">
                <div className="flex justify-between items-center mb-4">
                  <span className="text-xs font-mono text-neutral-400 uppercase">Core Server</span>
                  <span className="text-xs font-mono text-amber-500 uppercase">{MOCK_SERVER.region} / {MOCK_SERVER.players} PLYRS</span>
                </div>
                <div className="h-2 w-full bg-neutral-800 rounded-full overflow-hidden">
                  <div className="h-full bg-amber-500 w-[50%]"></div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 flex-1">
                <div className="bg-neutral-950 rounded-lg p-4 border border-neutral-800 flex flex-col justify-between group hover:border-amber-500 transition-colors">
                  <div className="text-neutral-400 group-hover:text-amber-500 transition-colors"><UsersIcon /></div>
                  <div>
                    <h4 className="text-white text-sm font-bold uppercase mt-4 mb-1">Multiplayer</h4>
                    <p className="text-[10px] font-mono text-neutral-500">Latency: 12ms</p>
                  </div>
                </div>

                <div className="bg-neutral-950 rounded-lg p-4 border border-neutral-800 flex flex-col justify-between group hover:border-amber-500 transition-colors">
                  <div className="text-neutral-400 group-hover:text-amber-500 transition-colors"><BotIcon /></div>
                  <div>
                    <h4 className="text-white text-sm font-bold uppercase mt-4 mb-1">AI Swarm</h4>
                    <p className="text-[10px] font-mono text-neutral-500">Active Entities: 42</p>
                  </div>
                </div>

                <div className="bg-neutral-950 rounded-lg p-4 border border-neutral-800 flex flex-col justify-between group hover:border-amber-500 transition-colors">
                  <div className="text-neutral-400 group-hover:text-amber-500 transition-colors"><ShieldIcon /></div>
                  <div>
                    <h4 className="text-white text-sm font-bold uppercase mt-4 mb-1">Fair Play Guard</h4>
                    <p className="text-[10px] font-mono text-neutral-500">Status: Enforcing</p>
                  </div>
                </div>

                <div className="bg-neutral-950 rounded-lg p-4 border border-neutral-800 flex flex-col justify-between group hover:border-amber-500 transition-colors">
                  <div className="text-neutral-400 group-hover:text-amber-500 transition-colors"><UserIcon /></div>
                  <div>
                    <h4 className="text-white text-sm font-bold uppercase mt-4 mb-1">Progression</h4>
                    <p className="text-[10px] font-mono text-neutral-500">Sync: Online</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
