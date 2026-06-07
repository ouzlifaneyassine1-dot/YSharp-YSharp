const { app, BrowserWindow, ipcMain, dialog, Menu, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn, execSync } = require('child_process');

let mainWindow;
const userDataPath = app.getPath('userData');

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 900,
    minHeight: 600,
    frame: false,
    transparent: false,
    backgroundColor: '#0a0e17',
    titleBarStyle: 'hidden',
    webPreferences: {
      preload: path.join(__dirname, 'src', 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.loadFile(path.join(__dirname, 'src', 'index.html'));
  mainWindow.on('closed', () => { mainWindow = null; });
}

app.whenReady().then(createWindow);
app.on('window-all-closed', () => { if (process.platform !== 'darwin') app.quit(); });
app.on('activate', () => { if (mainWindow === null) createWindow(); });

// Window controls
ipcMain.on('window-minimize', () => mainWindow?.minimize());
ipcMain.on('window-maximize', () => {
  if (mainWindow?.isMaximized()) mainWindow.unmaximize();
  else mainWindow?.maximize();
});
ipcMain.on('window-close', () => mainWindow?.close());

// File system operations
ipcMain.handle('fs-readdir', async (_, dir) => {
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    return entries.map(e => ({ name: e.name, isDir: e.isDirectory() }));
  } catch { return []; }
});

ipcMain.handle('fs-readfile', async (_, filePath) => {
  try { return fs.readFileSync(filePath, 'utf-8'); } catch { return null; }
});

ipcMain.handle('fs-writefile', async (_, filePath, content) => {
  try { fs.writeFileSync(filePath, content, 'utf-8'); return true; } catch { return false; }
});

ipcMain.handle('fs-exists', async (_, p) => fs.existsSync(p));
ipcMain.handle('fs-mkdir', async (_, p) => { try { fs.mkdirSync(p, { recursive: true }); return true; } catch { return false; } });
ipcMain.handle('fs-rmdir', async (_, p) => { try { fs.rmSync(p, { recursive: true }); return true; } catch { return false; } });

ipcMain.handle('dialog-open', async (_, opts) => {
  const r = await dialog.showOpenDialog(mainWindow, opts);
  return r.canceled ? null : r.filePaths[0];
});

ipcMain.handle('dialog-save', async (_, opts) => {
  const r = await dialog.showSaveDialog(mainWindow, opts);
  return r.canceled ? null : r.filePath;
});

// Shell command execution
ipcMain.handle('shell-exec', async (_, cmd, cwd) => {
  return new Promise(resolve => {
    try {
      const result = execSync(cmd, { cwd: cwd || process.cwd(), encoding: 'utf-8', timeout: 30000 });
      resolve({ stdout: result, stderr: '', code: 0 });
    } catch (e) {
      resolve({ stdout: e.stdout || '', stderr: e.stderr || e.message, code: e.status || -1 });
    }
  });
});

// Spawn shell process for terminal
ipcMain.handle('shell-spawn', async (_, id, cwd) => {
  const shell = process.platform === 'win32' ? 'cmd.exe' : '/bin/bash';
  const shellArgs = process.platform === 'win32' ? [] : [];
  const child = spawn(shell, shellArgs, {
    cwd: cwd || process.cwd(),
    env: { ...process.env, TERM: 'xterm-256color' },
    stdio: ['pipe', 'pipe', 'pipe'],
  });

  child.stdout.on('data', data => mainWindow?.webContents.send('terminal-output', id, data.toString()));
  child.stderr.on('data', data => mainWindow?.webContents.send('terminal-output', id, data.toString()));
  child.on('exit', code => mainWindow?.webContents.send('terminal-exit', id, code));

  terminals.set(id, child);
  return true;
});

const terminals = new Map();

ipcMain.handle('shell-write', async (_, id, data) => {
  const child = terminals.get(id);
  if (child) { child.stdin.write(data); return true; }
  return false;
});

ipcMain.handle('shell-resize', async (_, id, cols, rows) => {
  // no-op on Windows, useful for Linux PTY
  return true;
});

ipcMain.handle('shell-kill', async (_, id) => {
  const child = terminals.get(id);
  if (child) { child.kill(); terminals.delete(id); return true; }
  return false;
});

// AI Agent API
ipcMain.handle('ai-ask', async (_, config, messages) => {
  const { provider, endpoint, apiKey, model } = config;
  try {
    const url = endpoint || (provider === 'ollama'
      ? 'http://localhost:11434/v1/chat/completions'
      : 'https://api.bigpickle.ai/v1/chat/completions');

    const body = JSON.stringify({
      model: model || (provider === 'ollama' ? 'gemma4:4b' : 'big-pickle-1'),
      messages,
      stream: false,
    });

    const headers = { 'Content-Type': 'application/json' };
    if (apiKey) headers['Authorization'] = `Bearer ${apiKey}`;

    const res = await fetch(url, { method: 'POST', headers, body, signal: AbortSignal.timeout(60000) });
    const data = await res.json();
    return { ok: true, content: data.choices?.[0]?.message?.content || JSON.stringify(data) };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

// AI streaming
ipcMain.handle('ai-ask-stream', async (_, config, messages) => {
  const { provider, endpoint, apiKey, model } = config;
  try {
    const url = endpoint || (provider === 'ollama'
      ? 'http://localhost:11434/v1/chat/completions'
      : 'https://api.bigpickle.ai/v1/chat/completions');

    const body = JSON.stringify({
      model: model || (provider === 'ollama' ? 'gemma4:4b' : 'big-pickle-1'),
      messages,
      stream: true,
    });

    const headers = { 'Content-Type': 'application/json' };
    if (apiKey) headers['Authorization'] = `Bearer ${apiKey}`;

    const res = await fetch(url, { method: 'POST', headers, body });
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const json = line.slice(6).trim();
          if (json === '[DONE]') continue;
          try {
            const chunk = JSON.parse(json);
            const content = chunk.choices?.[0]?.delta?.content || '';
            if (content) mainWindow?.webContents.send('ai-chunk', content);
          } catch {}
        }
      }
    }
    mainWindow?.webContents.send('ai-done');
    return { ok: true };
  } catch (e) {
    mainWindow?.webContents.send('ai-error', e.message);
    return { ok: false, error: e.message };
  }
});

// Yo package manager
ipcMain.handle('yo-exec', async (_, args) => {
  const yoPaths = [
    path.join(process.env.LOCALAPPDATA || '', 'Programs', 'YSharp', 'bin', 'yo.exe'),
    path.join(__dirname, '..', 'dist', 'yo.exe'),
    path.join(__dirname, '..', '..', 'dist', 'yo.exe'),
  ];
  let yoBin = 'yo';
  for (const p of yoPaths) { if (fs.existsSync(p)) { yoBin = p; break; } }

  try {
    const result = execSync(`"${yoBin}" ${args}`, { encoding: 'utf-8', timeout: 15000 });
    return { ok: true, stdout: result, stderr: '' };
  } catch (e) {
    return { ok: true, stdout: e.stdout || '', stderr: e.stderr || e.message };
  }
});

// Settings
ipcMain.handle('settings-get', () => {
  const p = path.join(userDataPath, 'settings.json');
  try { return JSON.parse(fs.readFileSync(p, 'utf-8')); } catch { return {}; }
});

ipcMain.handle('settings-set', async (_, settings) => {
  const p = path.join(userDataPath, 'settings.json');
  try { fs.writeFileSync(p, JSON.stringify(settings, null, 2)); return true; } catch { return false; }
});

// Open external URLs
ipcMain.handle('open-external', async (_, url) => shell.openExternal(url));
