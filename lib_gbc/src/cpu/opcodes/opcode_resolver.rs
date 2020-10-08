use super::opcodes_resolvers::*;
use crate::mmu::memory::Memory;


pub enum OpcodeFuncType{
    OpcodeFunc(OpcodeFunc),
    U8OpcodeFunc(U8OpcodeFunc),
    U16OpcodeFunc(U16OpcodeFunc),
    U32OpcodeFunc(U32OpcodeFunc),
    MemoryOpcodeFunc(MemoryOpcodeFunc),
    U8MemoryOpcodeFunc(U8MemoryOpcodeFunc),
    U16MemoryOpcodeFunc(U16MemoryOpcodeFunc),
    U32MemoryOpcodeFunc(U32MemoryOpcodeFunc)
}

pub struct  OpcodeResolver{
    opcode_func_resolver:fn(u8)->Option<OpcodeFunc>,
    u8_opcode_func_resolver:fn(u8)->Option<U8OpcodeFunc>,
    u16_opcode_func_resolver:fn(u8,u8)->Option<U16OpcodeFunc>,
    u32_opcode_func_resolver:fn(u8)->Option<U32OpcodeFunc>,
    memory_opcode_func_resolver:fn(u8)->Option<MemoryOpcodeFunc>,
    memory_opcode_func_2bytes_resolver:fn(u8,u8)->Option<MemoryOpcodeFunc>,
    u8_memory_opcode_func_resolver:fn(u8)->Option<U8MemoryOpcodeFunc>,
    u16_memory_opcode_func_resolver:fn(u8,u8)->Option<U16MemoryOpcodeFunc>,
    u32_memory_opcode_func_resolver:fn(u8)->Option<U32MemoryOpcodeFunc>
}


impl OpcodeResolver{
    pub fn get_opcode(&mut self, opcode:u8, memory:&dyn Memory, program_counter:&mut u16)->OpcodeFuncType{
        let opcode_func = (self.opcode_func_resolver)(opcode);
        match opcode_func{
            Some(func)=> return OpcodeFuncType::OpcodeFunc(func),
            None=>{}
        }
        let memory_opcode_func = (self.memory_opcode_func_resolver)(opcode);
        match memory_opcode_func{
            Some(func)=> return OpcodeFuncType::MemoryOpcodeFunc(func),
            None=>{}
        }
        let u8_opcode_func=(self.u8_opcode_func_resolver)(opcode);
        match u8_opcode_func{
            Some(func)=> return OpcodeFuncType::U8OpcodeFunc(func),
            None=>{}
        }
        let u8_memory_func=(self.u8_memory_opcode_func_resolver)(opcode);
        match u8_memory_func{
            Some(func)=> return OpcodeFuncType::U8MemoryOpcodeFunc(func),
            None=>{}
        }
        let postfix:u8 = memory.read(*program_counter);
        let u16_opcode_func=(self.u16_opcode_func_resolver)(opcode, postfix);
        match u16_opcode_func{
            Some(func)=>return OpcodeFuncType::U16OpcodeFunc(func),
            None=>{}
        }
        let u32_opcode_func = (self.u32_opcode_func_resolver)(opcode);
        match u32_opcode_func{
            Some(func)=> return OpcodeFuncType::U32OpcodeFunc(func),
            None=>{}
        }
        let u32_memory_opcode_func=(self.u32_memory_opcode_func_resolver)(opcode);
        match u32_memory_opcode_func{
            Some(func)=> return OpcodeFuncType::U32MemoryOpcodeFunc(func),
            None=>{}
        }
        let u16_memory_opcode_func = (self.u16_memory_opcode_func_resolver)(opcode, postfix);
        match u16_memory_opcode_func{
            Some(func)=>return OpcodeFuncType::U16MemoryOpcodeFunc(func),
            None=>{}
        }
        let memory_opcode_func = (self.memory_opcode_func_2bytes_resolver)(opcode, postfix);
        match memory_opcode_func{
            Some(func)=>{
                //this is the only opcodes type that does not uses the postfix byte and therfore does not increment the program counter 
                //so im incrementing is manually
                *program_counter+=1;
                return OpcodeFuncType::MemoryOpcodeFunc(func);
            },
            None=>{}
        }
        
        std::panic!("no opcode matching: {:#X?}, nextb{:#X?}, c_pc{:#X?}",opcode, postfix, program_counter);
    }
}

impl Default for OpcodeResolver{
    fn default()->OpcodeResolver{
        OpcodeResolver{
            opcode_func_resolver:get_opcode_func_resolver(),
            memory_opcode_func_resolver:get_memory_opcode_func_resolver(),
            memory_opcode_func_2bytes_resolver:get_memory_opcode_func_2bytes_resolver(),
            u8_opcode_func_resolver:get_u8_opcode_func_resolver(),
            u8_memory_opcode_func_resolver:get_u8_memory_opcode_func_resolver(),
            u16_memory_opcode_func_resolver:get_u16_memory_opcode_func_resolver(),
            u16_opcode_func_resolver:get_u16_opcode_func_resolver(),
            u32_opcode_func_resolver:get_u32_opcode_func_resolver(),
            u32_memory_opcode_func_resolver:get_u32_memory_opcode_func_resolver()
        }
    }
}


