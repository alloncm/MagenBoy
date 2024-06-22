use magenboy_core::{mmu::{vram_dma_controller::VramDmaController, external_memory_bus::*, carts::Mbc}, ppu::{gb_ppu::*, gfx_device::*, ppu_state::PpuState}, machine::Mode};

struct StubGfxDevice;
impl GfxDevice for StubGfxDevice{
    fn swap_buffer(&mut self, _:&[Pixel; SCREEN_HEIGHT * SCREEN_WIDTH]) {}
}

const MEMORY_SIZE:usize = 0x1000;

struct EmptyMbc{
    memory:[u8;MEMORY_SIZE]
}
impl Mbc for EmptyMbc{
    fn get_ram(&mut self)->&mut [u8] {unreachable!()}
    fn has_battery(&self)->bool {false}
    fn read_bank0(&self, address:u16)->u8 {self.memory[address as usize]}
    fn read_current_bank(&self, address:u16)->u8 {self.read_bank0(address)}
    fn write_rom(&mut self, address:u16, value:u8) {self.memory[address as usize] = value}
    fn read_external_ram(&self, _:u16)->u8 {unreachable!()}
    fn write_external_ram(&mut self, _:u16, _:u8) {unreachable!()}
}

#[test]
fn vram_dma_transfer_test(){
    let mut controller = VramDmaController::new();
    let mut ppu = GbPpu::new(StubGfxDevice, Mode::CGB);
    let mut mbc = EmptyMbc{memory:[22;MEMORY_SIZE]};
    let mut memory = ExternalMemoryBus::new(&mut mbc, Bootrom::None);
    let dma_len_reg = 100;

    ppu.vram.set_bank_reg(1);
    controller.set_mode_length(dma_len_reg);
    controller.cycle(1000, &mut memory, &mut ppu);

    let expected_dma_byte_len = (dma_len_reg + 1) as u16 * 0x10;
    for i in 0..expected_dma_byte_len{
        assert_eq!(ppu.vram.read_bank(i as u16, 1), 22);
    }
    for i in expected_dma_byte_len..MEMORY_SIZE as u16{
        assert_eq!(ppu.vram.read_bank(i as u16, 1), 0);
    }
}

#[test]
fn vram_hblank_dma_transfer_test<'a>(){
    let mut controller = VramDmaController::new();
    let mut ppu = GbPpu::new(StubGfxDevice, Mode::CGB);
    let mut mbc = EmptyMbc{memory:[22;MEMORY_SIZE]};
    let mut memory = ExternalMemoryBus::new(&mut mbc, Bootrom::None);
    let dma_len_reg = 100;

    ppu.vram.set_bank_reg(1);
    ppu.state = PpuState::Hblank;
    ppu.ly_register = 0;
    controller.set_mode_length(dma_len_reg);
    for _ in 0..dma_len_reg + 1{
        controller.cycle(100, &mut memory, &mut ppu);
        ppu.ly_register += 1; 
    }

    let expected_dma_byte_len = (dma_len_reg + 1) as u16 * 0x10;
    for i in 0..expected_dma_byte_len{
        assert_eq!(ppu.vram.read_bank(i as u16, 1), 22);
    }
    for i in expected_dma_byte_len..MEMORY_SIZE as u16{
        assert_eq!(ppu.vram.read_bank(i as u16, 1), 0);
    }
}