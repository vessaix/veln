import { describe, it, expect } from 'vitest'
import type { VM } from './types'

describe('Types', () => {
  it('VM interface should accept valid data', () => {
    const vm: VM = {
      uuid: '550e8400-e29b-41d4-a716-446655440000',
      name: 'test-vm',
      state: 'running',
      cpus: 2,
      memory_mb: 2048,
      disk_gb: 20,
      network_backend: 'tap',
      console_type: 'serial',
    }

    expect(vm.name).toBe('test-vm')
    expect(vm.state).toBe('running')
    expect(vm.cpus).toBe(2)
  })
})
