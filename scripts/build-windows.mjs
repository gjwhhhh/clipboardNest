import { spawnSync } from "node:child_process";
import path from "node:path";

const workspaceRoot = process.cwd();
const isWindowsHost = process.platform === "win32";
const command = isWindowsHost ? process.env.ComSpec || "cmd.exe" : "npx";
const tauriCmd = path.join(workspaceRoot, "node_modules", ".bin", "tauri.cmd");

const args = isWindowsHost
  ? ["/d", "/s", "/c", `"${tauriCmd}" build --bundles msi,nsis`]
  : [
      "tauri",
      "build",
      "--bundles",
      "nsis",
      "--runner",
      "cargo-xwin",
      "--target",
      "x86_64-pc-windows-msvc",
    ];

const env = { ...process.env };

if (!isWindowsHost) {
  const extraPaths = [
    path.join(workspaceRoot, ".bin"),
    "/opt/homebrew/opt/lld/bin",
    "/opt/homebrew/opt/llvm/bin",
  ];

  env.PATH = `${extraPaths.join(path.delimiter)}${path.delimiter}${env.PATH ?? ""}`;
  console.log("当前不是 Windows 主机，自动切换为 Windows NSIS 交叉打包。");
  console.log("说明：该路径只产出 NSIS .exe；MSI 仍需在 Windows 主机或 Windows CI 上构建。");
}

const result = spawnSync(command, args, {
  cwd: workspaceRoot,
  env,
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
