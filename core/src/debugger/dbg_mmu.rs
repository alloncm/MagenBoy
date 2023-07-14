use crate::{mmu::{Memory, gb_mmu::GbMmu, interrupts_handler::InterruptRequest}, utils::fixed_size_set::FixedSizeSet, apu::audio_device::AudioDevice, ppu::gfx_device::GfxDevice, keypad::joypad_provider::JoypadProvider};

pub struct MemoryWatcher{
    watching_addrs:crate::utils::fixed_size_set::FixedSizeSet<u16, 0xFF>,
    pub hit_addr:Option<u16>,
}

impl MemoryWatcher{
    pub fn add_address(&mut self, address:u16){self.watching_addrs.add(address)}
    pub fn try_remove_address(&mut self, address:u16)->bool{self.watching_addrs.try_remove(address)}
}

pub struct DbgMmu<'a, AD:AudioDevice, GFX:GfxDevice, JYP:JoypadProvider> {
    inner:GbMmu<'a, AD, GFX, JYP>,
    pub mem_watch:MemoryWatcher,
}

impl<'a, AD:AudioDevice, GFX:GfxDevice, JYP:JoypadProvider> DbgMmu<'a, AD, GFX, JYP>{
    pub fn new(mmu:GbMmu<'a, AD, GFX, JYP>)->Self{
        Self { 
            inner: mmu, 
            mem_watch: MemoryWatcher { watching_addrs: FixedSizeSet::new(), hit_addr: None, } 
        }
    }

    // Implement all the GbMMu public interface to replace with compile flags
    pub fn is_frame_finished(&mut self)->bool{self.inner.is_frame_finished()}
    pub fn poll_joypad_state(&mut self){self.inner.poll_joypad_state()}
    pub fn dma_block_cpu(&self)->bool{self.inner.dma_block_cpu()}
    pub fn cycle(&mut self, m_cycles:u8){self.inner.cycle(m_cycles)}
    pub fn handle_interrupts(&mut self, cpu_mie:bool)->InterruptRequest{self.inner.handle_interrupts(cpu_mie)}
}

impl<'a, AD:AudioDevice, GFX:GfxDevice, JYP:JoypadProvider> Memory for DbgMmu<'a, AD, GFX, JYP>{
    fn read(&mut self, address:u16, m_cycles:u8)->u8 {
        if self.mem_watch.watching_addrs.as_slice().contains(&address){
            self.mem_watch.hit_addr = Some(address);
        }
        return self.inner.read(address, m_cycles);
    }

    fn write(&mut self, address:u16, value:u8, m_cycles:u8) {
        if self.mem_watch.watching_addrs.as_slice().contains(&address){
            self.mem_watch.hit_addr = Some(address);
        }
        self.inner.write(address, value, m_cycles);
    }

    fn set_double_speed_mode(&mut self, state:bool) {self.inner.set_double_speed_mode(state)}
}