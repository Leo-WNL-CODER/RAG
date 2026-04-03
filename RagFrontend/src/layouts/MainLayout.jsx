import React from 'react';
import { useTheme } from '../context/ThemeContext';
import { useAuth } from '../context/AuthContext';
import { LogOut, BookOpen } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { ThemeToggle } from '../components/ThemeToggle';

export const MainLayout = ({ children }) => {
  const { auth, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/');
  };

  return (
    <div className="min-h-screen bg-white dark:bg-gray-950 transition-colors duration-200">
      <nav className="border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-gray-950/80 backdrop-blur-md sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            <div className="flex items-center gap-2 cursor-pointer" onClick={() => navigate(auth.isAuthenticated ? '/dashboard' : '/')}>
              <BookOpen className="text-indigo-600 dark:text-indigo-400" size={28} />
              <span className="text-xl font-bold text-gray-900 dark:text-white">RAG-AI</span>
            </div>
            
            <div className="flex items-center gap-4">
              <ThemeToggle />
              {auth.isAuthenticated && (
                <button
                  onClick={handleLogout}
                  className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-all duration-200"
                >
                  <LogOut size={18} />
                  <span>Logout</span>
                </button>
              )}
            </div>
          </div>
        </div>
      </nav>
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {children}
      </main>
    </div>
  );
};
