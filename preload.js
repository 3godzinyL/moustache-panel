const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
  onSystemData: (callback) => ipcRenderer.on('system-data', (event, ...args) => callback(...args)),
  removeSystemDataListener: () => ipcRenderer.removeAllListeners('system-data'),
}); 