import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import process from "node:process";

const root = fileURLToPath(new URL("..", import.meta.url));
const buildDir = join(root, "native", "build");
const outputDir = join(root, "src-tauri", "resources");
const target = join(outputDir, "moustache_native.dll");

function isUsableCmake(executable) {
  try {
    execFileSync(executable, ["--version"], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function visualStudioCmakeCandidates() {
  const candidates = [];
  const visualStudioRoots = [
    process.env.ProgramFiles,
    process.env["ProgramFiles(x86)"],
  ]
    .filter(Boolean)
    .map((directory) => join(directory, "Microsoft Visual Studio"));

  for (const visualStudioRoot of visualStudioRoots) {
    if (!existsSync(visualStudioRoot)) continue;

    for (const year of readdirSync(visualStudioRoot, { withFileTypes: true })) {
      if (!year.isDirectory()) continue;
      const yearDirectory = join(visualStudioRoot, year.name);

      for (const edition of readdirSync(yearDirectory, { withFileTypes: true })) {
        if (!edition.isDirectory()) continue;
        candidates.push(
          join(
            yearDirectory,
            edition.name,
            "Common7",
            "IDE",
            "CommonExtensions",
            "Microsoft",
            "CMake",
            "CMake",
            "bin",
            "cmake.exe",
          ),
        );
      }
    }
  }

  return candidates;
}

function findCmake() {
  const candidates = [
    process.env.MP_CMAKE,
    "cmake",
    process.env.ProgramFiles &&
      join(process.env.ProgramFiles, "CMake", "bin", "cmake.exe"),
    process.env.LOCALAPPDATA &&
      join(process.env.LOCALAPPDATA, "Programs", "CMake", "bin", "cmake.exe"),
    ...visualStudioCmakeCandidates(),
  ].filter(Boolean);

  const cmake = candidates.find(isUsableCmake);
  if (cmake) return cmake;

  throw new Error(
    [
      "CMake was not found.",
      "Install 'CMake tools for Windows' in Visual Studio Installer,",
      "add cmake.exe to PATH, or set MP_CMAKE to its full path.",
    ].join(" "),
  );
}

if (process.argv.includes("--clean")) {
  rmSync(buildDir, { recursive: true, force: true });
  if (existsSync(target)) rmSync(target);
  console.log("Native build cleaned.");
  process.exit(0);
}

if (process.platform !== "win32") {
  console.log("Native C++ engine is Windows-only; skipping on this platform.");
  process.exit(0);
}

mkdirSync(buildDir, { recursive: true });
mkdirSync(outputDir, { recursive: true });

const cmake = findCmake();
console.log(`Using CMake: ${cmake}`);

execFileSync(
  cmake,
  [
    "-S",
    join(root, "native"),
    "-B",
    buildDir,
    "-A",
    process.env.MP_NATIVE_ARCH ?? "x64",
    "-DCMAKE_BUILD_TYPE=Release",
  ],
  { stdio: "inherit" },
);

execFileSync(
  cmake,
  ["--build", buildDir, "--config", "Release", "--parallel"],
  { stdio: "inherit" },
);

const candidates = [
  join(buildDir, "bin", "Release", "moustache_native.dll"),
  join(buildDir, "bin", "moustache_native.dll"),
  join(buildDir, "Release", "moustache_native.dll"),
];

const dll = candidates.find(existsSync);
if (!dll) {
  const listing = existsSync(buildDir) ? readdirSync(buildDir).join(", ") : "empty";
  throw new Error(`CMake completed but DLL was not found. Build directory: ${listing}`);
}

copyFileSync(dll, target);
console.log(`Native engine copied to ${target}`);
