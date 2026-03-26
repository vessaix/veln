export interface VM {
  uuid: string
  name: string
  state: 'stopped' | 'running' | 'unknown'
  cpus: number
  memory_mb: number
  disk_gb: number
  network_backend: 'tap' | 'vale'
  console_type: 'serial' | 'vnc'
}

export interface SystemInfo {
  pool: string
  vm_root: string
  resources: {
    memory_mb: number
    cpu_cores: number
    disk_gb: number
  }
}

export interface VMDetails extends VM {
  config?: {
    iso?: string
    network_device?: string
    vnc_port?: number
  }
  runtime?: {
    pid?: number
    uptime?: number
  }
}
