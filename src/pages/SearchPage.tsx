import { useState } from "react";
import { Search, Download, ExternalLink, Loader2, Check, BookmarkPlus, ImageIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface Asset {
  id: string;
  title: string;
  author: string;
  url: string;
  preview_url: string | null;
  price: string | null;
  license: string;
  short_text: string;
  tags: string[];
  asset_type: string;
  source: string;
}

interface DownloadTask {
  id: string;
  asset_id: string;
  asset_title: string;
  url: string;
  local_path: string;
  status: string;
  progress: number;
  total_bytes: number;
  downloaded_bytes: number;
  error: string | null;
}

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [results, setResults] = useState<Asset[]>([]);
  const [selectedSources, setSelectedSources] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloadingIds, setDownloadingIds] = useState<Set<string>>(new Set());
  const [downloadedIds, setDownloadedIds] = useState<Set<string>>(new Set());
  const [savedIds, setSavedIds] = useState<Set<string>>(new Set());
  const [imgErrors, setImgErrors] = useState<Set<string>>(new Set());

  const sources = [
    { id: "itch.io", name: "itch.io", color: "bg-pink-500" },
    { id: "opengameart", name: "OpenGameArt", color: "bg-green-500" },
    { id: "kenney", name: "Kenney", color: "bg-blue-500" },
  ];

  const handleSearch = async () => {
    if (!query.trim()) return;
    
    setIsSearching(true);
    setError(null);
    setResults([]);
    
    try {
      const params = {
        query: query.trim(),
        sources: selectedSources.length > 0 ? selectedSources : null,
        page: 1,
      };
      
      const assets = await invoke<Asset[]>("search_assets", { params });
      setResults(assets);
      
      if (assets.length === 0) {
        setError("未找到相关素材，尝试换个关键词");
      }
    } catch (err) {
      console.error("Search error:", err);
      setError("搜索出错，请稍后重试");
    } finally {
      setIsSearching(false);
    }
  };

  const handleDownload = async (asset: Asset) => {
    setDownloadingIds(prev => new Set(prev).add(asset.id));
    
    try {
      const task = await invoke<DownloadTask>("start_download", { asset });
      
      console.log("Download started:", task);
      setDownloadedIds(prev => new Set(prev).add(asset.id));
      setSavedIds(prev => new Set(prev).add(asset.id)); // 同时标记为已保存
    } catch (err) {
      console.error("Download error:", err);
    } finally {
      setDownloadingIds(prev => {
        const next = new Set(prev);
        next.delete(asset.id);
        return next;
      });
    }
  };

  const handleSaveToLibrary = async (asset: Asset) => {
    try {
      await invoke("save_to_library", { asset });
      setSavedIds(prev => new Set(prev).add(asset.id));
    } catch (err) {
      console.error("Save error:", err);
    }
  };

  const handleImageError = (id: string) => {
    setImgErrors(prev => new Set(prev).add(id));
  };

  const toggleSource = (sourceId: string) => {
    setSelectedSources(prev => 
      prev.includes(sourceId)
        ? prev.filter(s => s !== sourceId)
        : [...prev, sourceId]
    );
  };

  const getSourceColor = (source: string) => {
    switch (source) {
      case "itch.io": return "bg-pink-500/20 text-pink-400";
      case "opengameart": return "bg-green-500/20 text-green-400";
      case "kenney": return "bg-blue-500/20 text-blue-400";
      default: return "bg-gray-500/20 text-gray-400";
    }
  };

  const isValidImageUrl = (url: string | null): boolean => {
    if (!url) return false;
    if (imgErrors.has(url)) return false;
    return url.startsWith("http") && !url.includes("placeholder");
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <header className="p-6 border-b border-[#333]">
        <h1 className="text-2xl font-bold mb-4">搜索素材</h1>
        
        {/* Search Bar */}
        <div className="flex gap-3">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              placeholder="输入关键词搜索游戏素材... 例如: pixel art, character, tileset"
              className="w-full pl-10 pr-4 py-3 bg-[#1a1a1a] border border-[#333] rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors"
            />
          </div>
          <button
            onClick={handleSearch}
            disabled={isSearching || !query.trim()}
            className="px-6 py-3 bg-blue-500 hover:bg-blue-600 disabled:bg-blue-500/50 rounded-lg font-medium transition-colors flex items-center gap-2"
          >
            {isSearching ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Search className="w-5 h-5" />
            )}
            {isSearching ? "搜索中..." : "搜索"}
          </button>
        </div>

        {/* Source Filters */}
        <div className="flex gap-2 mt-3">
          <span className="text-sm text-gray-400 mr-2">来源：</span>
          {sources.map(source => (
            <button
              key={source.id}
              onClick={() => toggleSource(source.id)}
              className={`px-3 py-1 rounded-full text-sm transition-colors ${
                selectedSources.includes(source.id)
                  ? source.color + " text-white"
                  : "bg-[#1a1a1a] text-gray-400 hover:bg-[#252525]"
              }`}
            >
              {source.name}
            </button>
          ))}
          {selectedSources.length > 0 && (
            <button
              onClick={() => setSelectedSources([])}
              className="px-3 py-1 text-sm text-gray-500 hover:text-white"
            >
              清除
            </button>
          )}
        </div>
      </header>

      {/* Results */}
      <div className="flex-1 overflow-auto p-6">
        {/* Empty State */}
        {!isSearching && results.length === 0 && !error && (
          <div className="flex flex-col items-center justify-center h-full text-gray-500">
            <Search className="w-16 h-16 mb-4 opacity-50" />
            <p className="text-lg">输入关键词开始搜索</p>
            <p className="text-sm mt-2">支持 itch.io、OpenGameArt、Kenney 等多个素材网站</p>
            <div className="mt-4 flex gap-2 flex-wrap justify-center">
              <button onClick={() => setQuery("pixel art")} className="px-3 py-1 bg-[#1a1a1a] rounded-full text-sm hover:bg-[#252525]">pixel art</button>
              <button onClick={() => setQuery("character")} className="px-3 py-1 bg-[#1a1a1a] rounded-full text-sm hover:bg-[#252525]">character</button>
              <button onClick={() => setQuery("tileset")} className="px-3 py-1 bg-[#1a1a1a] rounded-full text-sm hover:bg-[#252525]">tileset</button>
              <button onClick={() => setQuery("ui pack")} className="px-3 py-1 bg-[#1a1a1a] rounded-full text-sm hover:bg-[#252525]">ui pack</button>
            </div>
          </div>
        )}

        {/* Loading State */}
        {isSearching && (
          <div className="flex flex-col items-center justify-center h-full">
            <Loader2 className="w-12 h-12 animate-spin text-blue-500 mb-4" />
            <p className="text-gray-400">正在搜索多个素材网站...</p>
          </div>
        )}

        {/* Error State */}
        {error && !isSearching && (
          <div className="flex flex-col items-center justify-center h-full text-gray-500">
            <p className="text-lg">{error}</p>
            <button onClick={() => setQuery("")} className="mt-2 text-blue-400 hover:underline">清空搜索</button>
          </div>
        )}

        {/* Result Stats */}
        {results.length > 0 && (
          <div className="mb-4 text-sm text-gray-400">
            找到 {results.length} 个素材
          </div>
        )}

        {/* Result Grid */}
        {results.length > 0 && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {results.map((asset) => (
              <div
                key={asset.id}
                className="bg-[#1a1a1a] rounded-lg overflow-hidden border border-[#333] hover:border-blue-500/50 transition-colors group"
              >
                {/* Preview */}
                <div className="aspect-video bg-[#252525] relative overflow-hidden">
                  {isValidImageUrl(asset.preview_url) ? (
                    <img
                      src={asset.preview_url!}
                      alt={asset.title}
                      className="w-full h-full object-cover"
                      onError={() => handleImageError(asset.id)}
                      loading="lazy"
                    />
                  ) : (
                    <div className="w-full h-full flex flex-col items-center justify-center text-gray-600">
                      <ImageIcon className="w-12 h-12 mb-2" />
                      <span className="text-xs">{asset.source}</span>
                    </div>
                  )}
                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                    <button
                      onClick={() => handleSaveToLibrary(asset)}
                      disabled={savedIds.has(asset.id)}
                      className={`p-2 rounded-lg transition-colors ${
                        savedIds.has(asset.id)
                          ? "bg-green-500"
                          : "bg-yellow-500 hover:bg-yellow-600"
                      }`}
                      title="保存到素材库"
                    >
                      {savedIds.has(asset.id) ? (
                        <Check className="w-5 h-5" />
                      ) : (
                        <BookmarkPlus className="w-5 h-5" />
                      )}
                    </button>
                    <button
                      onClick={() => handleDownload(asset)}
                      disabled={downloadingIds.has(asset.id) || downloadedIds.has(asset.id)}
                      className={`p-2 rounded-lg transition-colors ${
                        downloadedIds.has(asset.id)
                          ? "bg-green-500"
                          : downloadingIds.has(asset.id)
                          ? "bg-gray-500"
                          : "bg-blue-500 hover:bg-blue-600"
                      }`}
                    >
                      {downloadedIds.has(asset.id) ? (
                        <Check className="w-5 h-5" />
                      ) : downloadingIds.has(asset.id) ? (
                        <Loader2 className="w-5 h-5 animate-spin" />
                      ) : (
                        <Download className="w-5 h-5" />
                      )}
                    </button>
                    <a
                      href={asset.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="p-2 bg-[#333] rounded-lg hover:bg-[#444] transition-colors"
                    >
                      <ExternalLink className="w-5 h-5" />
                    </a>
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
                  {asset.short_text && (
                    <p className="text-xs text-gray-500 mt-1 line-clamp-2">
                      {asset.short_text}
                    </p>
                  )}
                  <div className="flex items-center justify-between mt-2">
                    <span className={`text-xs px-2 py-1 rounded ${getSourceColor(asset.source)}`}>
                      {asset.source}
                    </span>
                    <span className={`text-xs font-medium ${
                      asset.price === "Free" || asset.price === "$0" ? "text-green-400" : "text-yellow-400"
                    }`}>
                      {asset.price || "Free"}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
