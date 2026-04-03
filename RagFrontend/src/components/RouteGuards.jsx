import React from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

export const ProtectedRoute = ({ children }) => {
  const { auth } = useAuth();
  
  if (!auth.isAuthenticated) {
    return <Navigate to="/signin" replace />;
  }

  return children;
};

export const PublicRoute = ({ children }) => {
  const { auth } = useAuth();

  if (auth.isAuthenticated) {
    return <Navigate to="/dashboard" replace />;
  }

  return children;
};
