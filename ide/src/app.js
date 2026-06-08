// =====================================================
// Y# IDE — Main Application
// Features: Editor, AI Agent, Terminal, Explorer, Tasks,
//           Package Manager, Settings, Language Installer
// =====================================================

const IDE = {
  currentFile: null,
  openFiles: [],
  activeFile: null,
  rootDir: null,
  editor: null,
  terminal: null,
  termFit: null,
  termId: 'term-1',
  settings: {},
  tasks: [],
  currentDir: null,
};

// ---- Init ----
document.addEventListener('DOMContentLoaded', async () => {
  initSidebar();
  await loadSettings();
  initEditor();
  initTerminal();
  initTasks();
  initSettingsUI();
  updateAIStatus();

  // Welcome message
  addAIMessage('assistant', 'Hello! I am the **Y# AI Agent**.\n\nI can help you:\n- Write and debug Y#, Python, C, and other code\n- Install packages with `yo`\n- Manage tasks and projects\n- Explain errors\n\nConfigure me in Settings (⚙ sidebar). Try asking me to write a Y# program!');

  // Load tasks from settings
  if (IDE.settings.tasks) {
    IDE.tasks = IDE.settings.tasks;
    renderTasks();
  }
});

// ---- Sidebar ----
function initSidebar() {
  document.querySelectorAll('.sb-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const panel = btn.dataset.panel;
      if (!panel) return;

      // Toggle panel
      const container = document.getElementById('panel-container');
      const isActive = btn.classList.contains('active');

      document.querySelectorAll('.sb-btn').forEach(b => b.classList.remove('active'));
      document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));

      if (isActive && document.getElementById(`panel-${panel}`)?.classList.contains('active')) {
        container.classList.remove('has-panel');
        return;
      }

      btn.classList.add('active');
      const p = document.getElementById(`panel-${panel}`);
      if (p) {
        p.classList.add('active');
        container.classList.add('has-panel');
        if (panel === 'explorer' && IDE.rootDir) loadExplorer(IDE.rootDir);
      } else {
        container.classList.remove('has-panel');
      }
    });

    // Double-click on explorer button reopens
    btn.addEventListener('dblclick', () => {
      if (btn.dataset.panel === 'explorer' && IDE.rootDir) {
        openFolder();
      }
    });
  });
}

// ---- Status Bar AI Status ----
function updateAIStatus() {
  const provider = IDE.settings.aiProvider || 'ollama';
  const model = IDE.settings.aiModel || (provider === 'ollama' ? 'Gemma 4 4B' : 'BIG PICKLE');
  document.getElementById('sb-ai-status').textContent = `🤖 AI: ${model}`;
}

function switchAIProvider() {
  const sel = document.getElementById('ai-provider');
  IDE.settings.aiProvider = sel.value;
  saveSettings();
  updateAIStatus();
}

// ---- Save/Load Settings ----
async function loadSettings() {
  IDE.settings = (await ysharp.settings.get()) || {};
  if (!IDE.settings.tasks) IDE.settings.tasks = [];
  // Default AI config
  if (!IDE.settings.aiProvider) IDE.settings.aiProvider = 'ollama';
  if (!IDE.settings.aiModel) IDE.settings.aiModel = 'gemma4:4b';
  if (!IDE.settings.aiEndpoint) IDE.settings.aiEndpoint = '';
  if (!IDE.settings.aiKey) IDE.settings.aiKey = '';
  document.getElementById('ai-provider').value = IDE.settings.aiProvider;
}

async function saveSettings() {
  IDE.settings.tasks = IDE.tasks;
  await ysharp.settings.set(IDE.settings);
}

// ---- Open Folder ----
async function openFolder() {
  const dir = await ysharp.dialog.open({ properties: ['openDirectory'] });
  if (!dir) return;
  IDE.rootDir = dir;
  document.getElementById('sb-git').textContent = `📁 ${dir.split('\\').pop() || dir.split('/').pop()}`;
  loadExplorer(dir);
}

async function loadExplorer(dir) {
  const tree = document.getElementById('explorer-tree');
  tree.innerHTML = '<div class="explorer-item" style="opacity:0.6;padding:12px;font-size:12px;">Loading...</div>';
  const entries = await ysharp.fs.readdir(dir);
  tree.innerHTML = '';
  renderTreeItems(tree, entries, dir, 0);
}

function renderTreeItems(parent, entries, basePath, depth) {
  const sorted = [...entries].sort((a, b) => {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    return a.name.localeCompare(b.name);
  });

  for (const entry of sorted) {
    if (entry.name.startsWith('.') && entry.name !== '.gitignore') continue;
    if (entry.name === 'node_modules' || entry.name === 'target') continue;

    const item = document.createElement('div');
    item.className = 'explorer-item' + (entry.isDir ? ' dir' : '');
    item.innerHTML = `<span class="icon">${entry.isDir ? '📁' : extIcon(entry.name)}</span><span>${entry.name}</span>`;
    item.style.paddingLeft = `${12 + depth * 16}px`;

    if (entry.isDir) {
      let children = null;
      let loaded = false;
      item.addEventListener('click', async (e) => {
        e.stopPropagation();
        const fullPath = `${basePath}\\${entry.name}`;
        if (!children) {
          children = document.createElement('div');
          children.className = 'explorer-children';
          item.after(children);
          children.innerHTML = '<div class="explorer-item" style="opacity:0.6;font-size:12px;">Loading...</div>';
          const sub = await ysharp.fs.readdir(fullPath);
          children.innerHTML = '';
          renderTreeItems(children, sub, fullPath, depth + 1);
        } else {
          children.style.display = children.style.display === 'none' ? '' : 'none';
        }
      });
    } else {
      item.addEventListener('click', () => openFile(`${basePath}\\${entry.name}`));
    }
    parent.appendChild(item);
  }
}

function extIcon(name) {
  const ext = name.split('.').pop().toLowerCase();
  const icons = { ys: '🦪', yse: '🦪', c: '⚙', h: '⚙', py: '🐍', js: '⬡', ts: '⬡', rs: '🦀',
    json: '📋', md: '📝', toml: '⚙', yml: '⚙', yaml: '⚙', txt: '📄', html: '🌐', css: '🎨',
    exe: '▶', bat: '⊞', ps1: '⊞', vsix: '🧩', png: '🖼', jpg: '🖼', svg: '🖼', wasm: '⚡' };
  return icons[ext] || '📄';
}

// ---- New File ----
async function newFile() {
  if (!IDE.rootDir) { await openFolder(); return; }
  const name = prompt('File name:');
  if (!name) return;
  const path = `${IDE.rootDir}\\${name}`;
  await ysharp.fs.writefile(path, '');
  loadExplorer(IDE.rootDir);
  openFile(path);
}

// ---- File Operations ----
async function openFile(path) {
  const content = await ysharp.fs.readfile(path);
  if (content === null) return;

  // Check if already open
  const existing = IDE.openFiles.find(f => f.path === path);
  if (existing) {
    IDE.activeFile = existing;
    renderTabs();
    editorSetValue(existing.content);
    return;
  }

  const file = { path, name: path.split('\\').pop().split('/').pop(), content };
  IDE.openFiles.push(file);
  IDE.activeFile = file;
  IDE.currentFile = path;
  renderTabs();
  editorSetValue(content);
  updateCursor();
}

function renderTabs() {
  const container = document.getElementById('tabs-container');
  container.innerHTML = IDE.openFiles.map(f => `
    <button class="tab ${f === IDE.activeFile ? 'active' : ''}" onclick="switchTab('${f.path.replace(/\\/g, '\\\\')}')">
      ${f.name}
      <span class="close" onclick="event.stopPropagation();closeTab('${f.path.replace(/\\/g, '\\\\')}')">✕</span>
    </button>
  `).join('');
}

function switchTab(path) {
  const file = IDE.openFiles.find(f => f.path === path);
  if (!file) return;
  IDE.activeFile = file;
  IDE.currentFile = path;
  renderTabs();
  editorSetValue(file.content);
}

async function closeTab(path) {
  const idx = IDE.openFiles.findIndex(f => f.path === path);
  if (idx === -1) return;
  // Save if it was the active file
  if (IDE.activeFile?.path === path) {
    const content = editorGetValue();
    if (content !== null) {
      await ysharp.fs.writefile(path, content);
    }
  }
  IDE.openFiles.splice(idx, 1);
  if (IDE.openFiles.length > 0) {
    IDE.activeFile = IDE.openFiles[Math.min(idx, IDE.openFiles.length - 1)];
    IDE.currentFile = IDE.activeFile.path;
    editorSetValue(IDE.activeFile.content);
  } else {
    IDE.activeFile = null;
    IDE.currentFile = null;
    editorSetValue('');
  }
  renderTabs();
}

// ---- Editor (Monaco) ----
function initEditor() {
  require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs' } });
  require(['vs/editor/editor.main'], () => {
    IDE.editor = monaco.editor.create(document.getElementById('editor-container'), {
      value: '// Welcome to Y# IDE\n// Open a file or start coding!\n\nProgram Main {\n    PrintLine("Hello, Y#!");\n}\n',
      language: 'plaintext',
      theme: 'vs-dark',
      fontSize: 14,
      fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', monospace",
      lineNumbers: 'on',
      minimap: { enabled: true, scale: 1 },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      padding: { top: 8 },
      cursorBlinking: 'smooth',
      smoothScrolling: true,
      bracketPairColorization: { enabled: true },
      renderWhitespace: 'selection',
    });

    // Register Y# language
    monaco.languages.register({ id: 'ysharp' });
    monaco.languages.setMonarchTokensProvider('ysharp', {
      tokenizer: {
        root: [
          [/\b(Program|Function|Actor|Entity|System|Component|On|State|Return|If|Else|For|While|Loop|Var|Let|True|False|Import|Export|As|In)\b/, 'keyword'],
          [/\b(Int|Float|Bool|String|Void)\b/, 'type'],
          [/\/\/.*$/, 'comment'],
          [/".*?"/, 'string'],
          [/'[^']*'/, 'string'],
          [/\d+\.?\d*/, 'number'],
          [/[{}()\[\]]/, 'delimiter'],
          [/[=+\-*/<>!%]=?/, 'operator'],
        ]
      }
    });

    // Register C language
    monaco.languages.register({ id: 'ysharp-c' });
    monaco.languages.setMonarchTokensProvider('ysharp-c', {
      tokenizer: {
        root: [
          [/\b(auto|break|case|const|continue|default|do|else|enum|extern|for|goto|if|register|return|signed|sizeof|static|struct|switch|typedef|union|unsigned|volatile|while|int|char|float|double|void|long|short|int8_t|int16_t|int32_t|int64_t|uint8_t|uint16_t|uint32_t|uint64_t|bool|true|false|include|define)\b/, 'keyword'],
          [/\/\/.*$/, 'comment'], [/\/\*[\s\S]*?\*\//, 'comment'], [/".*?"/, 'string'], [/'[^']*'/, 'string'],
          [/\d+\.?\d*/, 'number'], [/[{}()\[\]]/, 'delimiter'], [/[=+\-*/<>!%&|^~]=?/, 'operator'],
        ]
      }
    });

    // Language detection
    IDE.editor.onDidChangeModelContent(() => {
      if (IDE.activeFile) {
        const lang = detectLanguage(IDE.activeFile.name);
        monaco.editor.setModelLanguage(IDE.editor.getModel(), lang);
      }
    });

    // Cursor position
    IDE.editor.onDidChangeCursorPosition((e) => {
      document.getElementById('sb-cursor').textContent = `Ln ${e.position.lineNumber}, Col ${e.position.column}`;
    });

    // Auto-save on blur
    IDE.editor.onDidBlurEditorText(async () => {
      await saveCurrentFile();
    });
  });
}

function detectLanguage(filename) {
  const ext = filename.split('.').pop().toLowerCase();
  const map = {
    ys: 'ysharp', yse: 'ysharp', c: 'ysharp-c', h: 'ysharp-c',
    py: 'python', js: 'javascript', ts: 'typescript',
    rs: 'rust', json: 'json', md: 'markdown',
    html: 'html', css: 'css', toml: 'ini', yml: 'yaml', yaml: 'yaml',
    ps1: 'powershell', bat: 'bat', sh: 'shell',
  };
  return map[ext] || 'plaintext';
}

function editorSetValue(text) {
  if (IDE.editor) {
    const model = IDE.editor.getModel();
    if (model) {
      model.setValue(text || '');
      if (IDE.activeFile) {
        const lang = detectLanguage(IDE.activeFile.name);
        monaco.editor.setModelLanguage(model, lang);
      }
    }
  }
}

function editorGetValue() {
  if (IDE.editor) {
    const model = IDE.editor.getModel();
    return model ? model.getValue() : null;
  }
  return null;
}

async function saveCurrentFile() {
  if (IDE.activeFile) {
    const content = editorGetValue();
    if (content !== null) {
      IDE.activeFile.content = content;
      await ysharp.fs.writefile(IDE.activeFile.path, content);
    }
  }
}

function updateCursor() {
  document.getElementById('sb-cursor').textContent = 'Ln 1, Col 1';
}

// ---- AI Agent ----
function addAIMessage(role, content) {
  const msgs = document.getElementById('ai-messages');
  const div = document.createElement('div');
  div.className = `ai-msg ${role}`;

  // Convert markdown-style formatting
  let html = content
    .replace(/```(\w+)?\n([\s\S]*?)```/g, '<div class="code-block">$2</div>')
    .replace(/`([^`]+)`/g, '<code style="background:var(--bg-deep);padding:1px 4px;border-radius:3px;font-family:var(--font-mono);font-size:12px;">$1</code>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\n/g, '<br>');

  div.innerHTML = html;
  msgs.appendChild(div);
  msgs.scrollTop = msgs.scrollHeight;
}

let aiHistory = [];
let isAIThinking = false;

async function sendAIMessage() {
  const input = document.getElementById('ai-input');
  const text = input.value.trim();
  if (!text || isAIThinking) return;
  input.value = '';
  isAIThinking = true;

  addAIMessage('user', text);
  document.getElementById('ai-send').disabled = true;

  // Show thinking indicator
  const msgs = document.getElementById('ai-messages');
  const typing = document.createElement('div');
  typing.className = 'typing-indicator';
  typing.innerHTML = '<div class="typing-dot"></div><div class="typing-dot"></div><div class="typing-dot"></div>';
  typing.id = 'ai-typing';
  msgs.appendChild(typing);
  msgs.scrollTop = msgs.scrollHeight;

  // Get AI config
  const provider = document.getElementById('ai-provider').value;
  const config = {
    provider,
    endpoint: IDE.settings.aiEndpoint || '',
    apiKey: IDE.settings.aiKey || '',
    model: provider === 'ollama' ? 'gemma4:4b' : (provider === 'bigpickle' ? 'big-pickle-1' : IDE.settings.aiModel),
  };

  // System prompt
  const systemMsg = {
    role: 'system',
    content: `You are the Y# AI Agent integrated in the Y# IDE. You help users write and debug code in Y#, Python, C, and other languages.

You can:
1. Write and explain code
2. Help install packages with the yo package manager
3. Create task checklists
4. Debug compilation errors
5. Explain language concepts

When asked to install something, use: yo install <package>
When asked to create tasks, use the task system.

Y# is a systems programming language. Basic syntax:
- Program Name { ... } — main program
- Function name() { ... } — function
- var x = 42 / var name: String = "Y#" — variables
- Print("text") / PrintLine("text") — output
- If(cond) { ... } Else { ... } — conditions
- While(cond) { ... } / Loop(i from 1 to 10) { ... } — loops
- Return value; — return

Be concise, helpful, and use code blocks for examples.`
  };

  aiHistory.push({ role: 'user', content: text });

  try {
    const messages = [systemMsg, ...aiHistory.slice(-20)];
    const result = await ysharp.ai.ask(config, messages);

    // Remove typing indicator
    const t = document.getElementById('ai-typing');
    if (t) t.remove();

    if (result.ok) {
      aiHistory.push({ role: 'assistant', content: result.content });
      addAIMessage('assistant', result.content);

      // Parse for task creation
      const taskMatch = result.content.match(/\[TASK\](.+?)\[\/TASK\]/g);
      if (taskMatch) {
        for (const m of taskMatch) {
          const taskText = m.replace('[TASK]', '').replace('[/TASK]', '').trim();
          if (taskText) IDE.tasks.push({ text: taskText, done: false });
        }
        renderTasks();
        saveSettings();
      }
    } else {
      addAIMessage('assistant', `⚠ Error: ${result.error}\n\nMake sure your AI provider is configured in Settings (⚙ sidebar). For local models, install Ollama and pull a model like gemma4:4b.`);
    }
  } catch (e) {
    const t = document.getElementById('ai-typing');
    if (t) t.remove();
    addAIMessage('assistant', `❌ Connection error: ${e.message}`);
  }

  isAIThinking = false;
  document.getElementById('ai-send').disabled = false;
}

// Allow Enter to send (Shift+Enter for newline)
document.addEventListener('DOMContentLoaded', () => {
  const input = document.getElementById('ai-input');
  if (input) {
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendAIMessage();
      }
    });
  }
});

// ---- Terminal ----
function initTerminal() {
  const termEl = document.getElementById('terminal');
  IDE.terminal = new Terminal({
    cursorBlink: true,
    cursorStyle: 'bar',
    fontSize: 13,
    fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'Consolas', monospace",
    theme: {
      background: '#0a0e17',
      foreground: '#e2e8f0',
      cursor: '#60a5fa',
      selection: 'rgba(96,165,250,0.3)',
      black: '#1e293b', red: '#f87171', green: '#4ade80', yellow: '#fbbf24',
      blue: '#60a5fa', magenta: '#a78bfa', cyan: '#22d3ee', white: '#e2e8f0',
    },
    allowProposedApi: true,
  });

  IDE.termFit = new FitAddon.FitAddon();
  IDE.terminal.loadAddon(IDE.termFit);

  IDE.terminal.open(termEl);
  IDE.termFit.fit();

  // Write welcome
  IDE.terminal.writeln('\x1b[36mY# IDE Terminal v9.0.1\x1b[0m');
  IDE.terminal.writeln('Type commands or use `yo` for package management.\r\n');

  // Handle input
  let currentLine = '';
  IDE.terminal.onKey(e => {
    const char = e.key;
    if (char === '\r') {
      IDE.terminal.writeln('');
      if (currentLine.trim()) {
        executeTerminalCommand(currentLine.trim());
      }
      currentLine = '';
      IDE.terminal.write('\x1b[36m$\x1b[0m ');
    } else if (char === '\x7f') { // Backspace
      if (currentLine.length > 0) {
        currentLine = currentLine.slice(0, -1);
        IDE.terminal.write('\b \b');
      }
    } else if (char.length === 1 && char.charCodeAt(0) >= 32) {
      currentLine += char;
      IDE.terminal.write(char);
    }
  });

  IDE.terminal.write('\x1b[36m$\x1b[0m ');

  // Resize
  window.addEventListener('resize', () => { try { IDE.termFit.fit(); } catch {} });
}

async function executeTerminalCommand(cmd) {
  try {
    if (cmd === 'clear' || cmd === 'cls') {
      IDE.terminal.clear();
      return;
    }
    if (cmd.startsWith('cd ')) {
      const dir = cmd.slice(3).trim();
      IDE.currentDir = dir;
      IDE.terminal.writeln(`\x1b[2mChanged directory\x1b[0m`);
      return;
    }
    const result = await ysharp.shell.exec(cmd, IDE.currentDir || IDE.rootDir || '');
    if (result.stdout) IDE.terminal.write(result.stdout.replace(/\n/g, '\r\n'));
    if (result.stderr) IDE.terminal.writeln(`\x1b[31m${result.stderr}\x1b[0m`);
  } catch (e) {
    IDE.terminal.writeln(`\x1b[31mError: ${e.message}\x1b[0m`);
  }
}

async function spawnTerminal() {
  IDE.terminal.clear();
  IDE.terminal.writeln('\x1b[36mTerminal ready.\x1b[0m');
  IDE.terminal.write('\x1b[36m$\x1b[0m ');
  await ysharp.shell.spawn(IDE.termId, IDE.rootDir || '');
  ysharp.shell.onOutput((id, data, exitCode) => {
    if (data) IDE.terminal.write(data);
    if (exitCode !== null && exitCode !== undefined) {
      IDE.terminal.writeln(`\r\n\x1b[2mProcess exited with code ${exitCode}\x1b[0m`);
      IDE.terminal.write('\x1b[36m$\x1b[0m ');
    }
  });
}

function clearTerminal() { IDE.terminal.clear(); }
function killTerminal() { ysharp.shell.kill(IDE.termId); }

// ---- Tasks ----
function initTasks() {
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && document.activeElement?.id === 'task-input') {
      addTask();
    }
  });
}

function addTask() {
  const input = document.getElementById('task-input');
  const text = input.value.trim();
  if (!text) return;
  IDE.tasks.push({ text, done: false });
  input.value = '';
  renderTasks();
  saveSettings();
}

function toggleTask(idx) {
  IDE.tasks[idx].done = !IDE.tasks[idx].done;
  renderTasks();
  saveSettings();
}

function deleteTask(idx) {
  IDE.tasks.splice(idx, 1);
  renderTasks();
  saveSettings();
}

function renderTasks() {
  const list = document.getElementById('task-list');
  list.innerHTML = IDE.tasks.map((t, i) => `
    <div class="task-item ${t.done ? 'done' : ''}">
      <div class="task-checkbox" onclick="toggleTask(${i})"></div>
      <span class="task-text">${escapeHtml(t.text)}</span>
      <button class="task-delete" onclick="deleteTask(${i})">✕</button>
    </div>
  `).join('');
}

// ---- Packages ----
async function runPkgCmd() {
  const input = document.getElementById('pkg-input');
  const output = document.getElementById('pkg-output');
  const cmd = input.value.trim();
  if (!cmd) return;
  output.textContent = 'Running...';
  const result = await ysharp.yo.exec(cmd);
  output.textContent = (result.stdout || result.stderr || 'Done (no output)').trim();
}

async function pkgList() {
  const output = document.getElementById('pkg-output');
  output.textContent = 'Fetching packages...';
  const result = await ysharp.yo.exec('list');
  output.textContent = (result.stdout || 'No packages installed').trim();
}

// ---- Settings UI ----
function initSettingsUI() {
  const body = document.getElementById('settings-body');
  body.innerHTML = `
    <div class="setting-group">
      <h3>🤖 AI Agent</h3>
      <div class="setting-row">
        <label>Provider</label>
        <select id="set-ai-provider" onchange="updateSetting('aiProvider', this.value)">
          <option value="ollama">Local (Ollama)</option>
          <option value="bigpickle">Cloud (BIG PICKLE)</option>
          <option value="openai">OpenAI Compatible</option>
        </select>
      </div>
      <div class="setting-row">
        <label>Model Name</label>
        <input id="set-ai-model" value="gemma4:4b" onchange="updateSetting('aiModel', this.value)" placeholder="gemma4:4b or big-pickle-1" />
      </div>
      <div class="setting-row">
        <label>API Endpoint</label>
        <input id="set-ai-endpoint" value="" onchange="updateSetting('aiEndpoint', this.value)" placeholder="http://localhost:11434/v1/chat/completions" />
      </div>
      <div class="setting-row">
        <label>API Key</label>
        <input id="set-ai-key" type="password" value="" onchange="updateSetting('aiKey', this.value)" placeholder="sk-..." />
      </div>
      <div class="setting-row">
        <label>Connection Test</label>
        <button class="btn" onclick="testAIConnection()">Test</button>
      </div>
    </div>
    <div class="setting-group">
      <h3>📦 Language Installer</h3>
      <div class="setting-row">
        <label>Install Python</label>
        <button class="btn" onclick="installLanguage('python')">Install</button>
      </div>
      <div class="setting-row">
        <label>Install C (MinGW GCC)</label>
        <button class="btn" onclick="installLanguage('c')">Install</button>
      </div>
      <div class="setting-row">
        <label>Install Y#</label>
        <button class="btn" onclick="installLanguage('ysharp')">Install</button>
      </div>
      <div class="setting-row">
        <label>Install Gemma 4 (Local AI)</label>
        <button class="btn" onclick="installLanguage('gemma')">Install via Ollama</button>
      </div>
    </div>
    <div class="setting-group">
      <h3>ℹ About Y# IDE</h3>
      <div style="padding:8px;font-size:12px;color:var(--text-muted);line-height:1.6;">
        <strong>Version:</strong> 9.0.1<br>
        <strong>Compiler:</strong> oys (Y# Compiler)<br>
        <strong>Package Manager:</strong> yo<br>
        <strong>AI:</strong> Local (Ollama/Gemma 4) + Cloud (BIG PICKLE)<br>
        <strong>Editor:</strong> Monaco Editor (VS Code engine)<br><br>
        <a href="#" onclick="ysharp.openExternal('https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp')" style="color:var(--accent);">GitHub Repo</a>
      </div>
    </div>
  `;
}

function updateSetting(key, value) {
  IDE.settings[key] = value;
  saveSettings();
  updateAIStatus();
}

async function testAIConnection() {
  const btn = event.target;
  btn.textContent = 'Testing...';
  btn.disabled = true;
  const provider = document.getElementById('set-ai-provider')?.value || IDE.settings.aiProvider;
  const config = {
    provider,
    endpoint: document.getElementById('set-ai-endpoint')?.value || IDE.settings.aiEndpoint || '',
    apiKey: document.getElementById('set-ai-key')?.value || IDE.settings.aiKey || '',
    model: document.getElementById('set-ai-model')?.value || IDE.settings.aiModel || '',
  };
  const result = await ysharp.ai.ask(config, [{ role: 'user', content: 'Say "OK" if you can hear me.' }]);
  btn.textContent = result.ok ? '✅ Connected' : `❌ ${result.error.slice(0, 40)}`;
  setTimeout(() => { btn.textContent = 'Test'; btn.disabled = false; }, 3000);
}

async function installLanguage(lang) {
  const names = { python: 'Python 3', c: 'MinGW GCC', ysharp: 'Y#', gemma: 'Ollama + Gemma 4' };
  const btn = event.target;
  btn.textContent = 'Installing...';
  btn.disabled = true;

  let cmd = '';
  if (lang === 'python') cmd = 'winget install Python.Python.3.12';
  else if (lang === 'c') cmd = 'choco install mingw -y';
  else if (lang === 'ysharp') cmd = 'powershell -ExecutionPolicy Bypass -File "' + process.env.INSTALLDIR + '\\scripts\\install.ps1" -Silent';
  else if (lang === 'gemma') cmd = 'ollama pull gemma4:4b';

  try {
    const result = await ysharp.shell.exec(cmd);
    btn.textContent = result.code === 0 || !result.stderr ? '✅ Installed' : `⚠ ${result.stderr.slice(0, 30)}`;
  } catch (e) {
    btn.textContent = `❌ Error`;
  }
  setTimeout(() => { btn.textContent = `Install ${names[lang]}`; btn.disabled = false; }, 5000);
}

// ---- Utility ----
function escapeHtml(text) {
  const d = document.createElement('div');
  d.textContent = text;
  return d.innerHTML;
}

// ---- Build & Run integration ----
async function buildAndRun() {
  await saveCurrentFile();
  if (!IDE.activeFile) return;
  const cmd = `oys build "${IDE.activeFile.path}"`;
  const result = await ysharp.shell.exec(cmd, IDE.rootDir);
  IDE.terminal.writeln(`\r\n\x1b[36m$ ${cmd}\x1b[0m`);
  if (result.stdout) IDE.terminal.write(result.stdout.replace(/\n/g, '\r\n'));
  if (result.stderr) IDE.terminal.writeln(`\x1b[31m${result.stderr}\x1b[0m`);

  if (result.code === 0) {
    // Try to run
    const runResult = await ysharp.shell.exec('output.exe', IDE.rootDir);
    if (runResult.stdout) IDE.terminal.write('\r\n' + runResult.stdout.replace(/\n/g, '\r\n'));
  }
}

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault();
    saveCurrentFile();
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
    e.preventDefault();
    buildAndRun();
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
    e.preventDefault();
    openFolder();
  }
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'p') {
    e.preventDefault();
    // Command palette placeholder
  }
});
