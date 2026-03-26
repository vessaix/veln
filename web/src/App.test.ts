import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import VMList from '../components/VMList.vue'
import type { VM } from '../types'

const mockVMs: VM[] = [
  {
    uuid: '550e8400-e29b-41d4-a716-446655440000',
    name: 'test-vm-1',
    state: 'running',
    cpus: 2,
    memory_mb: 2048,
    disk_gb: 20,
    network_backend: 'tap',
    console_type: 'serial',
  },
  {
    uuid: '550e8400-e29b-41d4-a716-446655440001',
    name: 'test-vm-2',
    state: 'stopped',
    cpus: 4,
    memory_mb: 4096,
    disk_gb: 50,
    network_backend: 'vale',
    console_type: 'vnc',
  },
]

describe('VMList', () => {
  it('renders VM list correctly', () => {
    const wrapper = mount(VMList, {
      props: {
        vms: mockVMs,
        loading: false,
      },
    })

    expect(wrapper.text()).toContain('test-vm-1')
    expect(wrapper.text()).toContain('test-vm-2')
    expect(wrapper.text()).toContain('2 CPU')
    expect(wrapper.text()).toContain('4 CPU')
  })

  it('shows empty state when no VMs', () => {
    const wrapper = mount(VMList, {
      props: {
        vms: [],
        loading: false,
      },
    })

    expect(wrapper.text()).toContain('No virtual machines configured')
  })

  it('shows loading state', () => {
    const wrapper = mount(VMList, {
      props: {
        vms: [],
        loading: true,
      },
    })

    expect(wrapper.text()).toContain('Loading VMs')
  })

  it('emits select event when VM is clicked', async () => {
    const wrapper = mount(VMList, {
      props: {
        vms: mockVMs,
        loading: false,
      },
    })

    await wrapper.find('.group').trigger('click')
    expect(wrapper.emitted('select')).toBeTruthy()
    expect(wrapper.emitted('select')![0]).toEqual([mockVMs[0]])
  })
})
