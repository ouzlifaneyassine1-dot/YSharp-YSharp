const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('ysharp', {
  window: {
    minimize: () => ipcRenderer.send('window-minimize'),
    maximize: () => ipcRenderer.send('window-maximize'),
    close: () => ipcRenderer.send('window-close'),
  },
  fs: {
    readdir: (dir) => ipcRenderer.invoke('fs-readdir', dir),
    readfile: (path) => ipcRenderer.invoke('fs-readfile', path),
    writefile: (path, content) => ipcRenderer.invoke('fs-writefile', path, content),
    exists: (path) => ipcRenderer.invoke('fs-exists', path),
    mkdir: (path) => ipcRenderer.invoke('fs-mkdir', path),
    rmdir: (path) => ipcRenderer.invoke('fs-rmdir', path),
  },
  dialog: {
    open: (opts) => ipcRenderer.invoke('dialog-open', opts),
    save: (opts) => ipcRenderer.invoke('dialog-save', opts),
  },
  shell: {
    exec: (cmd, cwd) => ipcRenderer.invoke('shell-exec', cmd, cwd || ''),
    spawn: (id, cwd) => ipcRenderer.invoke('shell-spawn', id, cwd || ''),
    write: (id, data) => ipcRenderer.invoke('shell-write', id, data),
    resize: (id, cols, rows) => ipcRenderer.invoke('shell-resize', id, cols, rows),
    kill: (id) => ipcRenderer.invoke('shell-kill', id),
    onOutput: (cb) => {
      ipcRenderer.on('terminal-output', (_, id, data) => cb(id, data));
      ipcRenderer.on('terminal-exit', (_, id, code) => cb(id, null, code));
    },
  },
  ai: {
    ask: (config, messages) => ipcRenderer.invoke('ai-ask', config, messages),
    askStream: (config, messages) => ipcRenderer.invoke('ai-ask-stream', config, messages),
    onChunk: (cb) => {
      ipcRenderer.on('ai-chunk', (_, text) => cb('chunk', text));
      ipcRenderer.on('ai-done', () => cb('done'));
      ipcRenderer.on('ai-error', (_, err) => cb('error', err));
    },
  },
  yo: {
    exec: (args) => ipcRenderer.invoke('yo-exec', args),
  },
  settings: {
    get: () => ipcRenderer.invoke('settings-get'),
    set: (s) => ipcRenderer.invoke('settings-set', s),
  },
  openExternal: (url) => ipcRenderer.invoke('open-external', url),
});
