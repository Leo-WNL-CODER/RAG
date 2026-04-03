import React, { createContext, useContext, useState, useEffect } from 'react';
import Cookies from 'js-cookie';

const AuthContext = createContext(null);

export const AuthProvider = ({ children }) => {
  const getSafeUser = () => {
    const user = localStorage.getItem('user');
    if (!user || user === "undefined") return null;
    try {
      return JSON.parse(user);
    } catch (e) {
      return null;
    }
  };

  const [auth, setAuth] = useState({
    user: getSafeUser(),
    // Rely on localStorage 'user' as a hint, since HttpOnly cookies aren't visible to JS
    isAuthenticated: !!localStorage.getItem('user'),
  });

  const login = (user) => {
    localStorage.setItem('user', JSON.stringify(user));
    setAuth({ user, isAuthenticated: true });
  };

  const logout = () => {
    // Attempt to remove cookies (will only work if not HttpOnly)
    Cookies.remove('accessToken');
    Cookies.remove('refreshToken');
    localStorage.removeItem('user');
    setAuth({ user: null, isAuthenticated: false });
  };

  // We don't poll for cookies here because they are likely HttpOnly and invisible to js-cookie
  
  return (
    <AuthContext.Provider value={{ auth, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error('useAuth must be used within an AuthProvider');
  return context;
};
