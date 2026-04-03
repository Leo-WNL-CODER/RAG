import React from 'react';
import { BrowserRouter, Route, Routes, Navigate } from 'react-router-dom';
import { AuthProvider } from './context/AuthContext';
import { ThemeProvider } from './context/ThemeContext';
import { Landing } from './pages/Landing';
import { SignIn } from './pages/SignIn';
import { SignUp } from './pages/SignUp';
import { Dashboard } from './pages/Dashboard';
import { MainLayout } from './layouts/MainLayout';
import { ProtectedRoute, PublicRoute } from './components/RouteGuards';

function App() {
  return (
    <AuthProvider>
      <ThemeProvider>
        <BrowserRouter>
          <Routes>
          <Route path="/" element={
            <PublicRoute>
              <MainLayout>
                <Landing />
              </MainLayout>
            </PublicRoute>
          } />

            <Route path="/signin" element={
              <PublicRoute>
                <MainLayout>
                  <SignIn />
                </MainLayout>
              </PublicRoute>
            } />
            <Route path="/signup" element={
              <PublicRoute>
                <MainLayout>
                  <SignUp />
                </MainLayout>
              </PublicRoute>
            } />

            <Route path="/dashboard" element={
              <ProtectedRoute>
                <MainLayout>
                  <Dashboard />
                </MainLayout>
              </ProtectedRoute>
            } />

            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </BrowserRouter>
      </ThemeProvider>
    </AuthProvider>
  );
}

export default App;
