/// <reference types="vite/client" />
import axios from "axios";

const baseURL = import.meta.env.VITE_API_URL || 'http://localhost:3001';

const api = axios.create({
  baseURL: baseURL,
  withCredentials: true, // Crucial for sending/receiving cookies
});

export default api;
