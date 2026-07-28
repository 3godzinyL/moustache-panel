use crate::models::{
    CpuMetric, GpuMetric, MemoryMetric, NetworkMetric, ProcessMetric, SnapshotSource,
    StorageMetric, SystemSnapshot,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

pub struct FallbackCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    last_network_sample: Instant,
}

impl FallbackCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            last_network_sample: Instant::now(),
        }
    }

    pub fn sample(&mut self) -> SystemSnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system
            .refresh_processes(ProcessesToUpdate::All, true);
        self.networks.refresh(true);
        self.disks.refresh(true);

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let ram_usage = percentage(used_memory, total_memory);

        let mut total_disk = 0_u64;
        let mut available_disk = 0_u64;
        for disk in self.disks.list() {
            total_disk = total_disk.saturating_add(disk.total_space());
            available_disk = available_disk.saturating_add(disk.available_space());
        }
        let used_disk = total_disk.saturating_sub(available_disk);

        let elapsed = self.last_network_sample.elapsed().max(Duration::from_millis(1));
        self.last_network_sample = Instant::now();
        let elapsed_secs = elapsed.as_secs_f64();
        let (rx_delta, tx_delta) = self
            .networks
            .iter()
            .fold((0_u64, 0_u64), |(rx, tx), (_, data)| {
                (
                    rx.saturating_add(data.received()),
                    tx.saturating_add(data.transmitted()),
                )
            });
        let rx = (rx_delta as f64 / elapsed_secs).round() as u64;
        let tx = (tx_delta as f64 / elapsed_secs).round() as u64;
        let logical_cpus = self.system.cpus().len().max(1) as f32;

        let mut processes = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessMetric {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                cpu: (process.cpu_usage() / logical_cpus).clamp(0.0, 100.0),
                memory_bytes: process.memory(),
            })
            .collect::<Vec<_>>();

        processes.sort_by(|a, b| {
            b.cpu
                .total_cmp(&a.cpu)
                .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
        });
        processes.truncate(8);

        SystemSnapshot {
            timestamp: now_millis(),
            source: SnapshotSource::RustFallback,
            cpu: CpuMetric {
                usage: self.system.global_cpu_usage().clamp(0.0, 100.0),
                temp_c: None,
            },
            gpu: GpuMetric {
                usage: None,
                vram_used_bytes: None,
                vram_total_bytes: None,
                temp_c: None,
            },
            ram: MemoryMetric {
                usage: ram_usage,
                used_bytes: used_memory,
                total_bytes: total_memory,
            },
            disk: StorageMetric {
                usage: percentage(used_disk, total_disk),
                used_bytes: used_disk,
                total_bytes: total_disk,
            },
            network: NetworkMetric {
                rx_bytes_per_sec: rx,
                tx_bytes_per_sec: tx,
            },
            processes,
        }
    }
}

fn percentage(value: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        ((value as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as f32
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
