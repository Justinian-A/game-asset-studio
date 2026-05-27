import { useState, useEffect } from "react";
import { Settings, FolderOpen, Key, Globe, Info, Download, HardDrive } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

export default function SettingsPage() {
  const [storagePath, setStoragePath] = useState("D:\\GameAssetStudio\\downloads");
  const [itchApiKey, setItchApiKey] = useState("");
  const [exportFormat, setExportFormat] = useState("unity");

  useEffect(() => {
    // Load saved settings
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      // TODO: Load settings from database or config file
      const savedPath = localStorage.getItem("storagePath");
      if (savedPath) {
        setStoragePath(savedPath);
      }
    } catch (err) {
      console.error("Failed to load settings:", err);
    }
  };

  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择素材存储路径",
        defaultPath: storagePath,
      });
      
      if (selected) {
        setStoragePath(selected as string);
        localStorage.setItem("storagePath", selected as string);
        // TODO: Save to Tauri backend
      }
    } catch (err) {
      console.error("Failed to open folder dialog:", err);
    }
  };

  const handleSaveSettings = () => {
    localStorage.setItem("storagePath", storagePath);
    localStorage.setItem("itchApiKey", itchApiKey);
    localStorage.setItem("exportFormat", exportFormat);
    // TODO: Save to Tauri backend
    alert("设置已保存");
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <header className="p-6 border-b border-[#333]">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Settings className="w-6 h-6" />
            设置
          </h1>
          <button
            onClick={handleSaveSettings}
            className="px-4 py-2 bg-blue-500 hover:bg-blue-600 rounded-lg text-sm transition-colors"
          >
            保存设置
          </button>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-2xl space-y-6">
          {/* Storage Settings */}
          <section className="bg-[#1a1a1a] rounded-lg p-6 border border-[#333]">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <FolderOpen className="w-5 h-5 text-blue-400" />
              存储设置
            </h2>
            <div>
              <label className="block text-sm text-gray-400 mb-2">
                素材存储路径
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={storagePath}
                  onChange={(e) => setStoragePath(e.target.value)}
                  className="flex-1 px-4 py-2 bg-[#0f0f0f] border border-[#333] rounded-lg text-white focus:outline-none focus:border-blue-500"
                />
                <button 
                  onClick={handleBrowse}
                  className="px-4 py-2 bg-[#252525] border border-[#333] rounded-lg hover:bg-[#333] transition-colors"
                >
                  浏览
                </button>
              </div>
              <p className="text-xs text-gray-500 mt-2">
                建议使用D盘，避免占用系统盘空间
              </p>
              <div className="mt-3 p-3 bg-[#0f0f0f] rounded-lg">
                <p className="text-xs text-gray-400 flex items-center gap-2">
                  <HardDrive className="w-4 h-4" />
                  当前下载位置: <span className="text-blue-400">{storagePath}</span>
                </p>
              </div>
            </div>
          </section>

          {/* API Settings */}
          <section className="bg-[#1a1a1a] rounded-lg p-6 border border-[#333]">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Key className="w-5 h-5 text-blue-400" />
              API 配置
            </h2>
            <div>
              <label className="block text-sm text-gray-400 mb-2">
                itch.io API Key
              </label>
              <input
                type="password"
                value={itchApiKey}
                onChange={(e) => setItchApiKey(e.target.value)}
                placeholder="输入你的 itch.io API Key（可选）"
                className="w-full px-4 py-2 bg-[#0f0f0f] border border-[#333] rounded-lg text-white focus:outline-none focus:border-blue-500"
              />
              <p className="text-xs text-gray-500 mt-2">
                在 itch.io 账户设置中获取 API Key（可选，用于访问更多功能）
              </p>
            </div>
          </section>

          {/* Source Settings */}
          <section className="bg-[#1a1a1a] rounded-lg p-6 border border-[#333]">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Globe className="w-5 h-5 text-blue-400" />
              素材来源
            </h2>
            <div className="space-y-3">
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  defaultChecked
                  className="w-4 h-4 rounded border-[#333] bg-[#0f0f0f] text-blue-500 focus:ring-blue-500"
                />
                <span>itch.io</span>
                <span className="text-xs text-gray-500 ml-auto">最大的独立游戏素材市场</span>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  defaultChecked
                  className="w-4 h-4 rounded border-[#333] bg-[#0f0f0f] text-blue-500 focus:ring-blue-500"
                />
                <span>OpenGameArt.org</span>
                <span className="text-xs text-gray-500 ml-auto">开源游戏素材社区</span>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  defaultChecked
                  className="w-4 h-4 rounded border-[#333] bg-[#0f0f0f] text-blue-500 focus:ring-blue-500"
                />
                <span>Kenney.nl</span>
                <span className="text-xs text-gray-500 ml-auto">全部免费</span>
              </label>
            </div>
          </section>

          {/* Export Settings */}
          <section className="bg-[#1a1a1a] rounded-lg p-6 border border-[#333]">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Download className="w-5 h-5 text-blue-400" />
              导出设置
            </h2>
            <div>
              <label className="block text-sm text-gray-400 mb-2">
                默认导出格式
              </label>
              <select
                value={exportFormat}
                onChange={(e) => setExportFormat(e.target.value)}
                className="w-full px-4 py-2 bg-[#0f0f0f] border border-[#333] rounded-lg text-white focus:outline-none focus:border-blue-500"
              >
                <option value="unity">Unity (SpriteSheet + .meta)</option>
                <option value="godot">Godot (SpriteFrames)</option>
                <option value="generic">通用 PNG 序列帧</option>
              </select>
              <p className="text-xs text-gray-500 mt-2">
                选择导出素材时的默认格式
              </p>
            </div>
          </section>

          {/* About */}
          <section className="bg-[#1a1a1a] rounded-lg p-6 border border-[#333]">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Info className="w-5 h-5 text-blue-400" />
              关于
            </h2>
            <div className="text-sm text-gray-400 space-y-2">
              <p>Game Asset Studio v1.0.0</p>
              <p>2D游戏素材聚合搜索与管理工具</p>
              <p>支持 itch.io、OpenGameArt、Kenney 等多个素材网站</p>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
