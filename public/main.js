document.addEventListener('DOMContentLoaded', () => {
    const cpuUsage = document.getElementById('cpu-usage');
    const cpuUsageBar = document.getElementById('cpu-usage-bar');
    const cpuTemp = document.getElementById('cpu-temp');
    const cpuProcesses = document.getElementById('cpu-processes');

    const gpuUsage = document.getElementById('gpu-usage');
    const gpuUsageBar = document.getElementById('gpu-usage-bar');
    const gpuVram = document.getElementById('gpu-vram');
    const gpuVramBar = document.getElementById('gpu-vram-bar');
    // const gpuProcesses = document.getElementById('gpu-processes');

    const ramUsage = document.getElementById('ram-usage');
    const ramUsageBar = document.getElementById('ram-usage-bar');
    const ramProcesses = document.getElementById('ram-processes');

    const diskUsage = document.getElementById('disk-usage');
    const diskUsageBar = document.getElementById('disk-usage-bar');
    // const diskProcesses = document.getElementById('disk-processes');

    function updateProcessList(element, processes) {
        element.innerHTML = '';
        if (processes && processes.length > 0) {
            processes.forEach(p => {
                const item = document.createElement('div');
                item.className = 'process-item';
                item.innerHTML = `
                    <span class="process-name">${p.name}</span>
                    <span class="process-usage">${p.usage}%</span>
                `;
                element.appendChild(item);
            });
        }
    }

    window.api.onSystemData((data) => {
        // CPU
        cpuUsage.textContent = `${data.cpu.usage}%`;
        cpuUsageBar.style.width = `${data.cpu.usage}%`;
        cpuTemp.textContent = `${data.cpu.temp}°C`;
        updateProcessList(cpuProcesses, data.cpu.top);

        // GPU
        gpuUsage.textContent = `${data.gpu.usage}%`;
        gpuUsageBar.style.width = `${data.gpu.usage}%`;
        gpuVram.textContent = `${data.gpu.vram}%`;
        gpuVramBar.style.width = `${data.gpu.vram}%`;
        // updateProcessList(gpuProcesses, data.gpu.top);

        // RAM
        ramUsage.textContent = `${data.ram.usage}%`;
        ramUsageBar.style.width = `${data.ram.usage}%`;
        updateProcessList(ramProcesses, data.ram.top);

        // Disk
        diskUsage.textContent = `${data.disk.usage}%`;
        diskUsageBar.style.width = `${data.disk.usage}%`;
        // updateProcessList(diskProcesses, data.disk.top);
    });
}); 