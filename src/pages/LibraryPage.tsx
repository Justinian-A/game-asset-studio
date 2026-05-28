import { useState, useEffect } from "react";
import { FolderOpen, Grid, List, Search, Tag, Star, Trash2, ExternalLink, Download, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface AssetRecord {
  id: number;
  source_id: string;
  title: string;
  author: string;
  url: string;
  preview_url: string | null;
  local_path: string | null;
  price: string | null;
  license: string;
  short_text: string;
  tags: string;
  asset_type: string;
  source: string;
  downloaded: boolean;
  favorite: boolean;
  created_at: string;
}

type ViewMode = "grid" | "list";
type FilterMode = "all" | "favorites" | "downloaded";

export default function LibraryPage() {
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [filterMode, setFilterMode] = useState<FilterMode>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [assets, setAssets] = useState<AssetRecord[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [stats, setStats] = useState<[number, number, number]>([0, 0, 0]);

  useEffect(() => {
    loadAssets();
    loadStats();
  }, [filterMode]);

  const loadAssets = async () => {
    setIsLoading(true);
    try {
      let result: AssetRecord[];
      
      if (searchQuery.trim()) {
        result = await invoke<AssetRecord[]>("search_library", { query: searchQuery.trim() });
      } else if (filterMode === "favorites") {
        result = await invoke<AssetRecord[]>("get_favorites");
      } else if (filterMode === "downloaded") {
        result = await invoke<AssetRecord[]>("get_downloaded_assets");
      } else {
        result = await invoke<AssetRecord[]>("get_library_assets", { offset: 0, limit: 100 });
      }
      
      setAssets(result);
    } catch (err) {
      console.error("Failed to load assets:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const result = await invoke<[number, number, number]>("get_library_stats");
      setStats(result);
    } catch (err) {
      console.error("Failed to load stats:", err);
    }
  };

  const handleSearch = () => {
    loadAssets();
  };

  const handleToggleFavorite = async (id: number, currentFavorite: boolean) => {
    try {
      await invoke("toggle_favorite", { id, favorite: !currentFavorite });
      setAssets(prev => prev.map(a => a.id === id ? { ...a, favorite: !currentFavorite } : a));
      loadStats();
    } catch (err) {
      console.error("Failed to toggle favorite:", err);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await invoke("delete_from_library", { id });
      setAssets(prev => prev.filter(a => a.id !== id));
      loadStats();
    } catch (err) {
      console.error("Failed to delete asset:", err);
    }
  };

  const getSourceColor = (source: string) => {
    switch (source) {
      case "itch.io": return "bg-pink-500/20 text-pink-400";
      case "opengameart": return "bg-green-500/20 text-green-400";
      case "kenney": return "bg-blue-500/20 text-blue-400";
      default: return "bg-gray-500/20 text-gray-400";
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <header className="p-6 border-b border-[#333]">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-bold">素材库</h1>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setViewMode("grid")}
              className={`p-2 rounded-lg transition-colors ${
                viewMode === "grid"
                  ? "bg-blue-500/20 text-blue-400"
                  : "text-gray-400 hover:text-white hover:bg-[#252525]"
              }`}
            >
              <Grid className="w-5 h-5" />
            </button>
            <button
              onClick={() => setViewMode("list")}
              className={`p-2 rounded-lg transition-colors ${
                viewMode === "list"
                  ? "bg-blue-500/20 text-blue-400"
                  : "text-gray-400 hover:text-white hover:bg-[#252525]"
              }`}
            >
              <List className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Search */}
        <div className="relative max-w-2xl mx-auto">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400 pointer-events-none" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder="搜索本地素材..."
            className="w-full pl-12 pr-4 py-3 bg-[#1a1a1a] border border-[#333] rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors"
          />
        </div>

        {/* Filters */}
        <div className="flex gap-2 mt-3">
          <button
            onClick={() => setFilterMode("all")}
            className={`flex items-center gap-1 px-3 py-1 rounded-full text-sm transition-colors ${
              filterMode === "all"
                ? "bg-blue-500/20 text-blue-400"
                : "bg-[#1a1a1a] text-gray-400 hover:bg-[#252525]"
            }`}
          >
            <Tag className="w-3 h-3" /> 全部 ({stats[0]})
          </button>
          <button
            onClick={() => setFilterMode("favorites")}
            className={`flex items-center gap-1 px-3 py-1 rounded-full text-sm transition-colors ${
              filterMode === "favorites"
                ? "bg-yellow-500/20 text-yellow-400"
                : "bg-[#1a1a1a] text-gray-400 hover:bg-[#252525]"
            }`}
          >
            <Star className="w-3 h-3" /> 收藏 ({stats[2]})
          </button>
          <button
            onClick={() => setFilterMode("downloaded")}
            className={`flex items-center gap-1 px-3 py-1 rounded-full text-sm transition-colors ${
              filterMode === "downloaded"
                ? "bg-green-500/20 text-green-400"
                : "bg-[#1a1a1a] text-gray-400 hover:bg-[#252525]"
            }`}
          >
            <Download className="w-3 h-3" /> 已下载 ({stats[1]})
          </button>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {/* Loading State */}
        {isLoading && (
          <div className="flex items-center justify-center h-full">
            <Loader2 className="w-12 h-12 animate-spin text-blue-500" />
          </div>
        )}

        {/* Empty State */}
        {!isLoading && assets.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-gray-500">
            <FolderOpen className="w-16 h-16 mb-4 opacity-50" />
            <p className="text-lg">
              {searchQuery ? "未找到匹配的素材" : "素材库为空"}
            </p>
            <p className="text-sm mt-2">
              {searchQuery ? "尝试换个关键词" : "从搜索页面保存素材后，将在这里显示"}
            </p>
          </div>
        )}

        {/* Grid View */}
        {!isLoading && assets.length > 0 && viewMode === "grid" && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {assets.map((asset) => (
              <div
                key={asset.id}
                className="bg-[#1a1a1a] rounded-lg overflow-hidden border border-[#333] hover:border-blue-500/50 transition-colors group"
              >
                {/* Preview */}
                <div className="aspect-video bg-[#252525] relative overflow-hidden">
                  {asset.preview_url ? (
                    <img
                      src={asset.preview_url}
                      alt={asset.title}
                      className="w-full h-full object-cover"
                      onError={(e) => {
                        (e.target as HTMLImageElement).style.display = "none";
                      }}
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-gray-600">
                      <FolderOpen className="w-12 h-12" />
                    </div>
                  )}
                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                    <button
                      onClick={() => handleToggleFavorite(asset.id, asset.favorite)}
                      className={`p-2 rounded-lg transition-colors ${
                        asset.favorite ? "bg-yellow-500" : "bg-[#333] hover:bg-[#444]"
                      }`}
                    >
                      <Star className="w-5 h-5" />
                    </button>
                    <a
                      href={asset.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="p-2 bg-[#333] rounded-lg hover:bg-[#444] transition-colors"
                    >
                      <ExternalLink className="w-5 h-5" />
                    </a>
                    <button
                      onClick={() => handleDelete(asset.id)}
                      className="p-2 bg-red-500/80 rounded-lg hover:bg-red-500 transition-colors"
                    >
                      <Trash2 className="w-5 h-5" />
                    </button>
                  </div>
                </div>

                {/* Info */}
                <div className="p-3">
                  <h3 className="font-medium truncate" title={asset.title}>
                    {asset.title}
                  </h3>
                  <p className="text-sm text-gray-400 mt-1 truncate">
                    {asset.author}
                  </p>
                  <div className="flex items-center justify-between mt-2">
                    <span className={`text-xs px-2 py-1 rounded ${getSourceColor(asset.source)}`}>
                      {asset.source}
                    </span>
                    <span className="text-xs text-gray-500">
                      {asset.price || "Free"}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* List View */}
        {!isLoading && assets.length > 0 && viewMode === "list" && (
          <div className="space-y-2">
            {assets.map((asset) => (
              <div
                key={asset.id}
                className="bg-[#1a1a1a] rounded-lg border border-[#333] hover:border-blue-500/50 transition-colors p-4 flex gap-4"
              >
                {/* Preview */}
                <div className="w-20 h-20 bg-[#252525] rounded overflow-hidden flex-shrink-0">
                  {asset.preview_url ? (
                    <img
                      src={asset.preview_url}
                      alt={asset.title}
                      className="w-full h-full object-cover"
                      onError={(e) => {
                        (e.target as HTMLImageElement).style.display = "none";
                      }}
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-gray-600">
                      <FolderOpen className="w-8 h-8" />
                    </div>
                  )}
                </div>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <h3 className="font-medium truncate">{asset.title}</h3>
                  <p className="text-sm text-gray-400 truncate">{asset.author}</p>
                  {asset.short_text && (
                    <p className="text-xs text-gray-500 mt-1 truncate">{asset.short_text}</p>
                  )}
                  <div className="flex items-center gap-2 mt-2">
                    <span className={`text-xs px-2 py-1 rounded ${getSourceColor(asset.source)}`}>
                      {asset.source}
                    </span>
                    <span className="text-xs text-gray-500">{asset.price || "Free"}</span>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleToggleFavorite(asset.id, asset.favorite)}
                    className={`p-2 rounded-lg transition-colors ${
                      asset.favorite ? "bg-yellow-500/20 text-yellow-400" : "text-gray-400 hover:text-white hover:bg-[#252525]"
                    }`}
                  >
                    <Star className="w-4 h-4" />
                  </button>
                  <a
                    href={asset.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="p-2 text-gray-400 hover:text-white hover:bg-[#252525] rounded-lg transition-colors"
                  >
                    <ExternalLink className="w-4 h-4" />
                  </a>
                  <button
                    onClick={() => handleDelete(asset.id)}
                    className="p-2 text-gray-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
