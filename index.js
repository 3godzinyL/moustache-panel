const { app, BrowserWindow, ipcMain, globalShortcut, screen } = require('electron');
const path = require('path');
const si = require('systeminformation');

let mainWindow;

function createWindow() {
  const primaryDisplay = screen.getPrimaryDisplay();
  const { width: screenWidth } = primaryDisplay.workAreaSize;

  const panelWidth = 300; 

  mainWindow = new BrowserWindow({
    width: panelWidth,
    height: 600,
    x: screenWidth - panelWidth,
    y: 0,
    frame: false,
    transparent: true,
    alwaysOnTop: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
    },
  });

  mainWindow.loadFile('public/index.html');

  // mainWindow.webContents.openDevTools({ mode: 'detach' });
}

app.whenReady().then(() => {
  createWindow();

  globalShortcut.register('Alt+P', () => {
    if (mainWindow) {
      if (mainWindow.isVisible()) {
        mainWindow.hide();
      } else {
        mainWindow.show();
      }
    }
  });

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });

  // Start sending data
  setInterval(async () => {
    try {
        const [cpu, mem, graphics, fs, network, processes] = await Promise.all([
            si.currentLoad(),
            si.mem(),
            si.graphics(),
            si.fsSize(),
            si.networkStats(),
            si.processes()
        ]);

        const cpuTemp = await si.cpuTemperature();

        const topCpu = processes.list.sort((a, b) => b.cpu - a.cpu).slice(0, 3);
        const topMem = processes.list.sort((a, b) => b.mem - a.mem).slice(0, 3);
        
        // Per-process disk and network usage is harder to get reliably.
        // We will stick to top cpu and memory for now.
        // We will get overall disk usage from fsSize.
        const disk = fs[0];

        const data = {
            cpu: {
                usage: cpu.currentLoad.toFixed(2),
                temp: cpuTemp.main || 'N/A',
                top: topCpu.map(p => ({ name: p.name, usage: p.cpu.toFixed(2) }))
            },
            gpu: {
                usage: graphics.controllers[0]?.utilizationGpu?.toFixed(2) || 'N/A',
                vram: ((graphics.controllers[0]?.vramDynamic ? graphics.controllers[0]?.vram - graphics.controllers[0]?.vramDynamic : 0) / graphics.controllers[0]?.vram * 100).toFixed(2) || 'N/A',
                temp: graphics.controllers[0]?.temperatureGpu || 'N/A',
                top: [] // GPU process info is very complex, skip for now
            },
            ram: {
                usage: ((mem.active / mem.total) * 100).toFixed(2),
                top: topMem.map(p => ({ name: p.name, usage: p.mem.toFixed(2) }))
            },
            disk: {
                usage: disk.use.toFixed(2),
                top: [] // Disk process info is complex, skip for now
            },
            network: {
                top: [] // Network process info is complex, skip for now
            }
        };

        if(mainWindow) {
            mainWindow.webContents.send('system-data', data);
        }

    } catch (e) {
        console.error('Error fetching system data:', e);
    }
  }, 1500);
});

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
