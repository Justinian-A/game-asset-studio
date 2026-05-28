import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExternalLink, Check, Loader2, Key, AlertCircle } from "lucide-react";

interface SetupPageProps {
  onComplete: () => void;
}

export default function SetupPage({ onComplete }: SetupPageProps) {
  const [apiKey, setApiKey] = useState("");
  const [isValidating, setIsValidating] = useState(false);
  const [validationResult, setValidationResult] = useState<{
    valid: boolean;
    message: string;
    username?: string;
  } | null>(null);
  const [step, setStep] = useState<"welcome" | "apikey" | "complete">("welcome");

  const handleGetApiKey = async () => {
    const url = "https://itch.io/user/settings/api-keys";
    try {
      // 尝试使用 Tauri 打开
      await invoke("open_url", { url });
    } catch (err) {
      // 备用方案：复制到剪贴板
      try {
        await navigator.clipboard.writeText(url);
        alert("链接已复制到剪贴板，请在浏览器中打开");
      } catch (e) {
        alert("请手动访问: " + url);
      }
    }
  };

  const handleValidateApiKey = async () => {
    if (!apiKey.trim()) {
      setValidationResult({ valid: false, message: "请输入 API Key" });
      return;
    }

    setIsValidating(true);
    setValidationResult(null);

    try {
      const result = await invoke<{ valid: boolean; username?: string; error?: string }>(
        "validate_itch_api_key",
        { apiKey: apiKey.trim() }
      );

      if (result.valid) {
        setValidationResult({
          valid: true,
          message: `验证成功！欢迎, ${result.username}`,
          username: result.username,
        });
        
        // 保存 API Key
        await invoke("set_itch_api_key", { apiKey: apiKey.trim() });
        
        // 延迟跳转到完成页面
        setTimeout(() => {
          setStep("complete");
        }, 1500);
      } else {
        setValidationResult({
          valid: false,
          message: result.error || "API Key 无效，请检查后重试",
        });
      }
    } catch (err) {
      setValidationResult({
        valid: false,
        message: `验证失败: ${err}`,
      });
    } finally {
      setIsValidating(false);
    }
  };

  const handleSkip = () => {
    // 用户选择跳过，稍后可以在设置中配置
    onComplete();
  };

  const handleFinish = () => {
    onComplete();
  };

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50">
      <div className="bg-[#1a1a1a] rounded-xl border border-[#333] w-full max-w-2xl mx-4 overflow-hidden">
        {/* Header */}
        <div className="p-6 border-b border-[#333] bg-gradient-to-r from-blue-500/10 to-purple-500/10">
          <h1 className="text-2xl font-bold flex items-center gap-2">
            🎮 Game Asset Studio
          </h1>
          <p className="text-gray-400 mt-2">
            2D游戏素材聚合搜索与管理工具
          </p>
        </div>

        {/* Content */}
        <div className="p-6">
          {step === "welcome" && (
            <div className="space-y-6">
              <div className="text-center">
                <h2 className="text-xl font-semibold mb-4">欢迎使用 Game Asset Studio！</h2>
                <p className="text-gray-400">
                  本工具支持从 itch.io、OpenGameArt、Kenney 等网站搜索和下载游戏素材。
                </p>
              </div>

              <div className="bg-[#0f0f0f] rounded-lg p-4">
                <h3 className="font-medium mb-2 flex items-center gap-2">
                  <Key className="w-4 h-4 text-blue-400" />
                  为什么需要 API Key？
                </h3>
                <ul className="text-sm text-gray-400 space-y-2">
                  <li>• itch.io 的下载链接需要通过 API 获取</li>
                  <li>• API Key 确保你可以下载已拥有的素材</li>
                  <li>• 免费素材也需要先"获取"才能下载</li>
                  <li>• 你的 API Key 只保存在本地，不会上传</li>
                </ul>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setStep("apikey")}
                  className="flex-1 py-3 bg-blue-500 hover:bg-blue-600 rounded-lg font-medium transition-colors"
                >
                  配置 API Key
                </button>
                <button
                  onClick={handleSkip}
                  className="px-6 py-3 bg-[#252525] hover:bg-[#333] rounded-lg transition-colors"
                >
                  稍后配置
                </button>
              </div>
            </div>
          )}

          {step === "apikey" && (
            <div className="space-y-6">
              <div>
                <h2 className="text-xl font-semibold mb-4">配置 itch.io API Key</h2>
                
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm text-gray-400 mb-2">
                      步骤 1: 获取 API Key
                    </label>
                    <button
                      onClick={handleGetApiKey}
                      className="w-full py-3 bg-[#252525] hover:bg-[#333] rounded-lg transition-colors flex items-center justify-center gap-2"
                    >
                      <ExternalLink className="w-4 h-4" />
                      打开 itch.io API Key 页面
                    </button>
                    <p className="text-xs text-gray-500 mt-2">
                      在 itch.io 登录后，创建一个新的 API Key（权限选择：下载）
                    </p>
                  </div>

                  <div>
                    <label className="block text-sm text-gray-400 mb-2">
                      步骤 2: 粘贴 API Key
                    </label>
                    <input
                      type="password"
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      placeholder="输入你的 itch.io API Key"
                      className="w-full px-4 py-3 bg-[#0f0f0f] border border-[#333] rounded-lg text-white focus:outline-none focus:border-blue-500"
                    />
                  </div>

                  <button
                    onClick={handleValidateApiKey}
                    disabled={isValidating || !apiKey.trim()}
                    className="w-full py-3 bg-blue-500 hover:bg-blue-600 disabled:bg-blue-500/50 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                  >
                    {isValidating ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        验证中...
                      </>
                    ) : (
                      "验证并保存"
                    )}
                  </button>

                  {validationResult && (
                    <div
                      className={`p-4 rounded-lg ${
                        validationResult.valid
                          ? "bg-green-500/10 border border-green-500/20"
                          : "bg-red-500/10 border border-red-500/20"
                      }`}
                    >
                      <div className="flex items-center gap-2">
                        {validationResult.valid ? (
                          <Check className="w-4 h-4 text-green-400" />
                        ) : (
                          <AlertCircle className="w-4 h-4 text-red-400" />
                        )}
                        <span
                          className={
                            validationResult.valid ? "text-green-400" : "text-red-400"
                          }
                        >
                          {validationResult.message}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setStep("welcome")}
                  className="px-6 py-3 bg-[#252525] hover:bg-[#333] rounded-lg transition-colors"
                >
                  返回
                </button>
                <button
                  onClick={handleSkip}
                  className="flex-1 py-3 bg-[#252525] hover:bg-[#333] rounded-lg transition-colors"
                >
                  跳过，稍后配置
                </button>
              </div>
            </div>
          )}

          {step === "complete" && (
            <div className="space-y-6 text-center">
              <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto">
                <Check className="w-8 h-8 text-green-400" />
              </div>
              
              <div>
                <h2 className="text-xl font-semibold mb-2">配置完成！</h2>
                <p className="text-gray-400">
                  {validationResult?.username
                    ? `已登录为: ${validationResult.username}`
                    : "API Key 已保存"}
                </p>
              </div>

              <div className="bg-[#0f0f0f] rounded-lg p-4 text-left">
                <h3 className="font-medium mb-2">接下来你可以：</h3>
                <ul className="text-sm text-gray-400 space-y-2">
                  <li>• 搜索并下载 itch.io 上的免费素材</li>
                  <li>• 下载你已购买的素材</li>
                  <li>• 从 OpenGameArt 和 Kenney 下载素材（无需登录）</li>
                </ul>
              </div>

              <button
                onClick={handleFinish}
                className="w-full py-3 bg-blue-500 hover:bg-blue-600 rounded-lg font-medium transition-colors"
              >
                开始使用
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
