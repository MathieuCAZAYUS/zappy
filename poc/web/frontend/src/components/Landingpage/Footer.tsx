export const Footer = () => {
  return (
    <footer className="bg-neutral-950 py-16 border-t border-neutral-900">
      <div className="max-w-5xl mx-auto px-6 flex flex-col md:flex-row justify-between items-center gap-8">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 bg-amber-500 rounded-sm transform rotate-45 flex items-center justify-center">
            <div className="w-2 h-2 bg-neutral-950 transform -rotate-45" />
          </div>
          <span className="text-xl font-black text-white uppercase tracking-tighter">Zap Arena</span>
        </div>

        <div className="flex gap-8">
          <a href="#" className="text-xs font-bold text-neutral-500 hover:text-amber-500 uppercase tracking-widest transition-colors">Discord</a>
          <a href="#" className="text-xs font-bold text-neutral-500 hover:text-amber-500 uppercase tracking-widest transition-colors">Twitter</a>
          <a href="#" className="text-xs font-bold text-neutral-500 hover:text-amber-500 uppercase tracking-widest transition-colors">Support</a>
        </div>

        <p className="text-neutral-600 text-xs font-medium">
          &copy; 2026 Zap Arena. Built with Rust.
        </p>
      </div>
    </footer>
  );
};
