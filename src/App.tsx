import { BrowserRouter as Router, Routes, Route, NavLink } from "react-router-dom";
import { Search, FolderOpen, Settings, Gamepad2 } from "lucide-react";

// Pages
import SearchPage from "./pages/SearchPage";
import LibraryPage from "./pages/LibraryPage";
import SettingsPage from "./pages/SettingsPage";

function App() {
  return (
    <Router>
      <div className="flex h-screen bg-[#0f0f0f] text-white">
        {/* Sidebar */}
        <aside className="w-16 bg-[#1a1a1a] flex flex-col items-center py-4 border-r border-[#333]">
          {/* Logo */}
          <div className="mb-8">
            <Gamepad2 className="w-8 h-8 text-blue-400" />
          </div>

          {/* Navigation */}
          <nav className="flex flex-col gap-2">
            <NavLink
              to="/"
              className={({ isActive }) =>
                `p-3 rounded-lg transition-colors ${
                  isActive
                    ? "bg-blue-500/20 text-blue-400"
                    : "text-gray-400 hover:text-white hover:bg-[#252525]"
                }`
              }
              title="搜索素材"
            >
              <Search className="w-5 h-5" />
            </NavLink>

            <NavLink
              to="/library"
              className={({ isActive }) =>
                `p-3 rounded-lg transition-colors ${
                  isActive
                    ? "bg-blue-500/20 text-blue-400"
                    : "text-gray-400 hover:text-white hover:bg-[#252525]"
                }`
              }
              title="素材库"
            >
              <FolderOpen className="w-5 h-5" />
            </NavLink>

            <NavLink
              to="/settings"
              className={({ isActive }) =>
                `p-3 rounded-lg transition-colors ${
                  isActive
                    ? "bg-blue-500/20 text-blue-400"
                    : "text-gray-400 hover:text-white hover:bg-[#252525]"
                }`
              }
              title="设置"
            >
              <Settings className="w-5 h-5" />
            </NavLink>
          </nav>
        </aside>

        {/* Main Content */}
        <main className="flex-1 overflow-auto">
          <Routes>
            <Route path="/" element={<SearchPage />} />
            <Route path="/library" element={<LibraryPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App;
