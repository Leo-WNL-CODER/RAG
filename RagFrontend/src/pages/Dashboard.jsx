import { useState, useRef } from "react";
import api from "../api";
import { Upload, Send, FileText, CheckCircle2, AlertCircle, Loader2, X, RefreshCw } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { cn } from "../lib/utils";
import { useAuth } from "../context/AuthContext";

export const Dashboard = () => {
  const { logout } = useAuth();
  const [selectedFile, setSelectedFile] = useState(null);
  const [query, setQuery] = useState("");
  const [queryResponse, setQueryResponse] = useState("");
  const [isUploading, setIsUploading] = useState(false);
  const [isQuerying, setIsQuerying] = useState(false);
  const [uploadSuccess, setUploadSuccess] = useState(false);
  const [error, setError] = useState("");
  const fileInputRef = useRef(null);

  const allowedExtensions = ['pdf', 'docx', 'txt'];
  const maxFileSize = 10 * 1024 * 1024; // 10MB

  const validateFile = (file) => {
    const extension = file.name.split('.').pop().toLowerCase();
    if (!allowedExtensions.includes(extension)) {
      return `Invalid file type. Allowed: ${allowedExtensions.join(', ')}`;
    }
    if (file.size > maxFileSize) {
      return "File too large. Maximum size is 10MB.";
    }
    return null;
  };

  const handleFileChange = (event) => {
    const file = event.target.files[0];
    if (!file) return;

    const validationError = validateFile(file);
    if (validationError) {
      setError(validationError);
      setSelectedFile(null);
      return;
    }

    setSelectedFile(file);
    setUploadSuccess(false);
    setError("");
    setQueryResponse("");
  };

  const handleUpload = async (e) => {
    e?.preventDefault();
    if (!selectedFile) {
      setError('Please select a file first.');
      return;
    }

    setIsUploading(true);
    setError("");

    const formData = new FormData();
    formData.append('myFile', selectedFile);
    console.log("Uploading file...");
    try {
      const response = await api.post('/parseDoc', formData);

      if (response.status === 202) {
        setUploadSuccess(true);
        setError("");
      } else {
        console.log("Upload failed");
        throw new Error('Upload failed');
      }
    } catch (err) {
      console.error("Upload error:", err);
      setError(err.response?.data?.message || 'Error uploading file. Please try again.');
      if (err.response?.status === 401) {
        logout();
      }
    } finally {
      setIsUploading(false);
    }
  };

  const handleUserQuery = async (e) => {
    e?.preventDefault();
    if (!query.trim()) return;
    if (!uploadSuccess) {
      setError('Please upload a document first.');
      return;
    }

    setIsQuerying(true);
    setError("");

    try {
      const response = await api.get(`/userQuery?q=${encodeURIComponent(query)}`);
      setQueryResponse(response.data);
    } catch (err) {
      setError('Error processing query. Please try again.');
    } finally {
      setIsQuerying(false);
    }
  };

  const resetAll = () => {
    setSelectedFile(null);
    setQuery("");
    setQueryResponse("");
    setUploadSuccess(false);
    setError("");
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  return (
    <div className="max-w-5xl mx-auto space-y-8">
      <header className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">Workspace</h1>
          <p className="text-gray-500 dark:text-gray-400">Manage your documents and start querying</p>
        </div>
        {(uploadSuccess || selectedFile) && (
          <button
            onClick={resetAll}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white bg-gray-100 dark:bg-gray-800 rounded-lg transition-all"
          >
            <RefreshCw size={16} />
            Reset Workspace
          </button>
        )}
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Sidebar: Upload */}
        <div className="lg:col-span-1 space-y-6">
          <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-6 shadow-sm">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
              <Upload size={20} className="text-indigo-600 dark:text-indigo-400" />
              Upload Document
            </h2>

            <div
              onClick={() => !uploadSuccess && fileInputRef.current?.click()}
              className={cn(
                "relative border-2 border-dashed rounded-xl p-8 transition-all duration-200 flex flex-col items-center justify-center text-center cursor-pointer",
                uploadSuccess
                  ? "border-green-500/50 bg-green-50/30 dark:bg-green-900/10 cursor-default"
                  : "border-gray-300 dark:border-gray-700 hover:border-indigo-500/50 hover:bg-gray-50 dark:hover:bg-gray-800/50"
              )}
            >
              <input
                type="file"
                ref={fileInputRef}
                onChange={handleFileChange}
                className="hidden"
                disabled={uploadSuccess}
              />

              {uploadSuccess ? (
                <CheckCircle2 className="text-green-500 mb-3" size={40} />
              ) : selectedFile ? (
                <FileText className="text-indigo-600 dark:text-indigo-400 mb-3" size={40} />
              ) : (
                <Upload className="text-gray-400 mb-3" size={40} />
              )}

              <p className="text-sm font-medium text-gray-900 dark:text-white">
                {selectedFile ? selectedFile.name : "Click to select file"}
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                PDF, DOCX, or TXT up to 10MB
              </p>
            </div>

            {selectedFile && !uploadSuccess && (
              <button
                type="button"
                onClick={handleUpload}
                disabled={isUploading}
                className="w-full mt-4 flex justify-center items-center gap-2 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-semibold transition-all disabled:opacity-70"
              >
                {isUploading ? <Loader2 className="animate-spin" size={18} /> : "Upload & Process"}
              </button>
            )}

            <AnimatePresence>
              {error && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: 'auto' }}
                  exit={{ opacity: 0, height: 0 }}
                  className="mt-4 p-3 bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-800/50 rounded-lg flex gap-2"
                >
                  <AlertCircle className="text-red-600 dark:text-red-400 shrink-0" size={18} />
                  <span className="text-xs text-red-700 dark:text-red-400">{error}</span>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>

        {/* Main: Chat */}
        <div className="lg:col-span-2 space-y-6">
          <div className="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 shadow-sm flex flex-col min-h-[500px]">
            <div className="p-4 border-b border-gray-100 dark:border-gray-800 flex items-center justify-between">
              <h2 className="font-semibold text-gray-900 dark:text-white">AI Assistant</h2>
              <div className="flex items-center gap-2">
                <span className={cn(
                  "w-2 h-2 rounded-full",
                  uploadSuccess ? "bg-green-500" : "bg-gray-300 dark:bg-gray-700"
                )} />
                <span className="text-xs text-gray-500 dark:text-gray-400">
                  {uploadSuccess ? "Knowledge base active" : "Waiting for document"}
                </span>
              </div>
            </div>

            <div className="flex-1 p-6 overflow-y-auto space-y-6">
              {!uploadSuccess ? (
                <div className="h-full flex flex-col items-center justify-center text-center opacity-60">
                  <div className="w-16 h-16 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center mb-4">
                    <FileText className="text-gray-400" size={32} />
                  </div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white">No active document</h3>
                  <p className="text-sm text-gray-500 max-w-xs mt-1">
                    Upload a file to enable the chat interface and start asking questions.
                  </p>
                </div>
              ) : queryResponse ? (
                <motion.div
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="space-y-4"
                >
                  <div className="flex justify-end">
                    <div className="bg-indigo-600 text-white rounded-2xl rounded-tr-none px-4 py-2.5 max-w-[80%] shadow-sm">
                      <p className="text-sm">{query}</p>
                    </div>
                  </div>
                  <div className="flex justify-start">
                    <div className="bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200 rounded-2xl rounded-tl-none px-5 py-4 max-w-[90%] shadow-sm border border-gray-200 dark:border-gray-700">
                      <p className="text-sm leading-relaxed whitespace-pre-wrap">{queryResponse}</p>
                    </div>
                  </div>
                </motion.div>
              ) : (
                <div className="h-full flex flex-col items-center justify-center text-center opacity-60">
                  <p className="text-sm text-gray-500">Document ready. Ask anything!</p>
                </div>
              )}
            </div>

            <div className="p-4 border-t border-gray-100 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-900/50 rounded-b-2xl">
              <form onSubmit={handleUserQuery} className="relative">
                <input
                  type="text"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder={uploadSuccess ? "Ask a question about your document..." : "Upload a document to start chatting"}
                  disabled={!uploadSuccess || isQuerying}
                  className="w-full pl-4 pr-12 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all outline-none disabled:opacity-60"
                />
                <button
                  type="submit"
                  disabled={!uploadSuccess || !query.trim() || isQuerying}
                  className="absolute right-2 top-1.5 p-1.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-all disabled:opacity-50 disabled:hover:bg-indigo-600"
                >
                  {isQuerying ? <Loader2 className="animate-spin" size={20} /> : <Send size={20} />}
                </button>
              </form>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
