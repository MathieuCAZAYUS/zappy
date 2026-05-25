import { MOCK_NEWS } from './mockData';

export const NewsSection = () => {
  return (
    <section id="intel" className="py-32 bg-neutral-900 border-t border-neutral-800">
      <div className="max-w-5xl mx-auto px-6">
        <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-16 gap-6">
          <div>
            <h2 className="text-3xl md:text-4xl font-black text-white uppercase tracking-tighter mb-4">Field Intel</h2>
            <div className="w-12 h-1 bg-amber-500"></div>
          </div>
          <button className="text-xs font-bold text-amber-500 uppercase tracking-widest hover:text-white transition-colors py-2 px-4 bg-neutral-950 rounded-full border border-neutral-800">
            View All Comm Logs
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {MOCK_NEWS.map((news) => (
            <div key={news.id} className="bg-neutral-950 rounded-2xl p-8 border border-neutral-800 flex flex-col h-full hover:border-neutral-600 transition-colors">
              <span className="text-[10px] font-mono text-amber-500 mb-4 tracking-widest">{news.date}</span>
              <h3 className="text-lg font-bold text-white mb-3 uppercase tracking-wide">{news.title}</h3>
              <p className="text-neutral-400 text-sm font-medium flex-1 leading-relaxed">{news.excerpt}</p>
              <button className="mt-8 text-left text-xs font-bold text-white uppercase tracking-widest hover:text-amber-500 transition-colors w-fit">
                Read More &rarr;
              </button>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};
