import React from 'react';
import { motion } from 'framer-motion';
import { useNavigate } from 'react-router-dom';
import { FileText, MessageSquare, ShieldCheck, Zap, ArrowRight, Database, Cpu } from 'lucide-react';
import { useTheme } from '../context/ThemeContext';

const KnowledgeDNA = () => {
  return (
    <div className="absolute inset-0 overflow-hidden pointer-events-none opacity-60 dark:opacity-40">
      <div className="absolute right-10 md:right-20 top-0 bottom-0 w-32 md:w-48 h-full">
        {[...Array(30)].map((_, i) => (
          <React.Fragment key={i}>
            <motion.div
              className="absolute w-2 h-2 rounded-full bg-indigo-500 shadow-[0_0_12px_rgba(99,102,241,0.9)]"
              style={{ top: `${i * 4}%` }}
              animate={{
                left: ["0%", "100%", "0%"],
                scale: [1, 1.6, 1],
                opacity: [0.3, 1, 0.3],
              }}
              transition={{ duration: 5, repeat: Infinity, delay: i * 0.15, ease: "easeInOut" }}
            />
            <motion.div
              className="absolute w-2 h-2 rounded-full bg-purple-500 shadow-[0_0_12px_rgba(168,85,247,0.9)]"
              style={{ top: `${i * 4}%` }}
              animate={{
                left: ["100%", "0%", "100%"],
                scale: [1, 1.6, 1],
                opacity: [0.3, 1, 0.3],
              }}
              transition={{ duration: 5, repeat: Infinity, delay: i * 0.15, ease: "easeInOut" }}
            />
          </React.Fragment>
        ))}
      </div>
    </div>
  );
};

const RAGIllustration = () => (
  <motion.div
    initial={{ opacity: 0, x: 50 }}
    animate={{ opacity: 1, x: 0 }}
    transition={{ duration: 0.8, delay: 0.5 }}
    className="relative w-full max-w-lg aspect-square"
  >
    {/* Animated Connections */}
    <svg className="absolute inset-0 w-full h-full" viewBox="0 0 400 400">
      <motion.path
        d="M 100 100 Q 200 50 300 100"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        className="text-indigo-500/30"
        strokeDasharray="10 5"
        animate={{ strokeDashoffset: [-50, 0] }}
        transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
      />
      <motion.path
        d="M 100 300 Q 200 350 300 300"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        className="text-purple-500/30"
        strokeDasharray="10 5"
        animate={{ strokeDashoffset: [50, 0] }}
        transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
      />
    </svg>

    {/* Nodes */}
    <motion.div
      animate={{ y: [0, -10, 0] }}
      transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
      className="absolute top-1/4 left-1/4 p-4 bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-700 z-10"
    >
      <FileText className="text-indigo-600 dark:text-indigo-400" size={40} />
      <div className="absolute -bottom-1 -right-1 w-4 h-4 bg-green-500 rounded-full border-2 border-white dark:border-gray-800" />
    </motion.div>

    <motion.div
      animate={{ y: [0, 10, 0] }}
      transition={{ duration: 5, repeat: Infinity, ease: "easeInOut" }}
      className="absolute top-1/2 right-1/4 p-4 bg-white dark:bg-gray-800 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-700 z-10"
    >
      <Database className="text-purple-600 dark:text-purple-400" size={40} />
    </motion.div>

    <motion.div
      animate={{ scale: [1, 1.05, 1] }}
      transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
      className="absolute bottom-1/4 left-1/2 -translate-x-1/2 p-6 bg-indigo-600 rounded-3xl shadow-2xl z-20"
    >
      <Cpu className="text-white" size={48} />
      <motion.div
        animate={{ opacity: [0, 1, 0], scale: [0.8, 1.2, 0.8] }}
        transition={{ duration: 2, repeat: Infinity }}
        className="absolute inset-0 bg-white/20 rounded-3xl blur-lg"
      />
    </motion.div>

    {/* Glow background */}
    <div className="absolute inset-0 bg-gradient-to-tr from-indigo-500/10 to-purple-500/10 blur-[80px] rounded-full -z-10" />
  </motion.div>
);

const AnimatedText = ({ text, className }) => {
  const words = text.split(" ");
  return (
    <motion.div
      initial="hidden"
      animate="visible"
      variants={{
        visible: { transition: { staggerChildren: 0.12 } },
        hidden: {}
      }}
      className={`flex flex-wrap ${className}`}
    >
      {words.map((word, index) => (
        <motion.span
          key={index}
          variants={{
            visible: { opacity: 1, y: 0, transition: { type: "spring", damping: 12, stiffness: 100 } },
            hidden: { opacity: 0, y: 20 }
          }}
          className="mr-3 mb-2"
        >
          {word}
        </motion.span>
      ))}
    </motion.div>
  );
};

const FeatureCard = ({ icon: Icon, title, description, index, color = "indigo" }) => {
  const { theme } = useTheme();

  const colors = {
    indigo: "text-indigo-600 dark:text-indigo-400 bg-indigo-50 dark:bg-indigo-900/30",
    purple: "text-purple-600 dark:text-purple-400 bg-purple-50 dark:bg-purple-900/30",
    blue: "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/30",
    emerald: "text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-900/30",
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      // transition={{ delay: index * }}
      whileHover={{
        y: -12,
        scale: 1.05,
        boxShadow: theme === 'light'
          ? "0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)"
          : "0 20px 25px -5px rgba(255, 255, 255, 0.15), 0 10px 10px -5px rgba(255, 255, 255, 0.05)"
      }}
      className="p-6 bg-white/40 dark:bg-gray-800/40 backdrop-blur-md rounded-xl border border-gray-200/50 dark:border-gray-700/50 shadow-md dark:shadow-none transition-all duration-150"
    >
      <div className={`w-12 h-12 rounded-lg flex items-center justify-center mb-4 shadow-inner ${colors[color] || colors.indigo}`}>
        <Icon size={24} />
      </div>
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">{title}</h3>
      <p className="text-gray-600 dark:text-gray-400 text-sm leading-relaxed">{description}</p>
    </motion.div>
  );
};

export const Landing = () => {
  const navigate = useNavigate();
  const { theme } = useTheme();

  return (
    <div className="min-h-screen bg-white dark:bg-gray-950 transition-colors duration-500 relative overflow-hidden">
      <KnowledgeDNA />

      {/* Hero Section */}
      <section className="relative pt-20 pb-16 lg:pt-32 lg:pb-24 z-10">
        <div className="container mx-auto px-4">
          <div className="flex flex-col lg:flex-row items-center gap-12 lg:gap-16">
            {/* Left Content */}
            <div className="flex-1 text-left">
              <motion.div
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.5 }}
                className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo-50/50 dark:bg-indigo-900/20 backdrop-blur-sm border border-indigo-100/50 dark:border-indigo-800/50 text-indigo-600 dark:text-indigo-400 text-sm font-medium mb-8"
              >
                <Zap size={14} className="animate-pulse" />
                <span>Intelligent Document RAG System</span>
              </motion.div>

              <AnimatedText
                text="Understand Your Documents with AI"
                className="text-5xl lg:text-7xl font-bold text-gray-900 dark:text-white mb-6 tracking-tight leading-[1.1]"
              />

              <motion.p
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.8 }}
                className="text-xl text-gray-600 dark:text-gray-400 mb-10 leading-relaxed max-w-xl"
              >
                Unlock insights from your PDF, DOCX, and TXT files instantly.
                Our RAG engine connects your documents directly to LLMs for grounded, accurate answers.
              </motion.p>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 1.0 }}
                className="flex flex-col sm:flex-row gap-4"
              >
                <motion.button
                  whileHover={{ 
                    scale: 1.05, 
                    boxShadow: theme === 'light' 
                      ? "0 20px 25px -5px rgba(0, 0, 0, 0.2)" 
                      : "0 20px 25px -5px rgba(251, 191, 36, 0.3)" 
                  }}
                  whileTap={{ scale: 0.98 }}
                  onClick={() => navigate('/signup')}
                  className={`group px-8 py-4 rounded-xl font-bold text-lg transition-all duration-200 flex items-center justify-center gap-2 shadow-lg ${
                    theme === 'light' 
                      ? "bg-gray-900 text-amber-50 hover:bg-black" 
                      : "bg-amber-50 text-gray-900 hover:bg-amber-100"
                  }`}
                >
                  Start for Free
                  <ArrowRight className="group-hover:translate-x-1 transition-transform" size={20} />
                </motion.button>

                <motion.button
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={() => navigate('/signin')}
                  className="px-8 py-4 bg-white/50 dark:bg-gray-800/50 backdrop-blur-sm hover:bg-white dark:hover:bg-gray-800 text-gray-900 dark:text-white border border-gray-200 dark:border-gray-700 rounded-xl font-semibold text-lg transition-all duration-200"
                >
                  Sign In
                </motion.button>
              </motion.div>
            </div>

            {/* Right Illustration */}
            <div className="flex-1 flex justify-center lg:justify-end">
              <RAGIllustration />
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-gray-50/30 dark:bg-gray-900/20 transition-colors duration-500 relative z-20 backdrop-blur-[2px]">
        <div className="container mx-auto px-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            <FeatureCard index={0} icon={FileText} color="blue" title="Multi-format Support" description="Upload PDF, DOCX, and TXT files. Our advanced parser extracts every detail accurately." />
            <FeatureCard index={1} icon={MessageSquare} color="purple" title="Intelligent Chat" description="Ask complex questions and get grounded answers based on your specific document content." />
            <FeatureCard index={2} icon={ShieldCheck} color="emerald" title="Secure Processing" description="Your files are scanned for threats and processed in a secure environment." />
            <FeatureCard index={3} icon={Zap} color="indigo" title="Real-time RAG" description="Experience lightning-fast retrieval and generation using our optimized vector engine." />
          </div>
        </div>
      </section>

      <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-indigo-500/50 to-transparent" />
    </div>
  );
};
