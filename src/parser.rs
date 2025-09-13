use crate::Error;

pub enum Instruction {
    /// 00E0 - clear screen
    ClearScreen,
    /// 00EE - return from subroutine
    ReturnFromSubroutine,
    /// 1NNN - jump to NNN
    Jump(u16),
    /// 2NNN - call subroutine at NNN
    Call(u16),
    /// 3XNN - skip next if VX equals NN
    SkipIfEqualByte(usize, u8),
    /// 4XNN - skip next if VX does not equal NN
    SkipIfNotEqualByte(usize, u8),
    /// 5XY0 - skip next if VX equals VY
    SkipIfRegistersEqual(usize, usize),
    /// 6XNN - set VX to NN
    SetRegisterToValue(usize, u8),
    /// 7XNN - add NN to VX
    AddToRegister(usize, u8),
    /// 8XY0 - set VX to value of VY
    SetRegisterToRegisterValue(usize, usize),
    /// 8XY1 - set VX | VY
    RegistersBitwiseOr(usize, usize),
    /// 8XY2 - set VX & VY
    RegistersBitwiseAnd(usize, usize),
    /// 8XY3 - set VX ^ VY
    RegistersBitwiseXor(usize, usize),
    /// 8XY4 - add VY to VX (with VF as overflow control)
    RegistersSumWithOverflow(usize, usize),
    /// 8XY5 - VX = VX - VY (with VF as overflow control)
    SubstractRegisterFromRegisterValue(usize, usize),
    /// 8XY6 - VX >>= 1, LSB stored in VF
    ShiftRegisterBitsRight(usize),
    /// 8XY7 - VX = VY - VX (with VF as overflow control)
    SubstractRegisterValueFromRegister(usize, usize),
    /// 8XYE - VX <<= 1 (with VF as overflow control)
    ShiftRegisterBitsLeft(usize),
    /// 9XY0 - skip next if VX does not equal VY
    SkipIfRegistersNotEqual(usize, usize),
    /// ANNN - set I to NNN
    SetIndexRegisterToValue(u16),
    /// BNNN - jump to V0 + NNN
    JumpByValue(u16),
    /// CXNN - set VX to rand(0, 255) & NN
    SetRegisterToRandAndValue(usize, u8),
    /// DXYN - draw a sprite
    DrawSprite(usize, usize, u8),
    /// EX9E - skip next if key == VX
    SkipIfKeyEqualsRegister(usize),
    /// EXA1 - skip next if key != VX
    SkipIfKeyNotEqualsRegister(usize),
    /// FX07 - set VX to delay timer value
    SetRegisterToDelayTimerValue(usize),
    /// FX15 - set delay timer to VX
    SetDelayTimerToRegisterValue(usize),
    /// FX18 - set sound timer to VX
    SetSoundTimerToRegisterValue(usize),
    /// FX29 - set I to location of sprite for character in VX
    SetIndexRegisterToSpriteForRegister(usize),
    /// FX33 - store binary coded decimal at memory under I(I+1)(I+2)
    StoreBinaryCodedDecimalAtIndexRegisterValue(usize),
    /// FX1E - add VX to I (don't consider overflow)
    AddRegisterToIndexRegister(usize),
    /// FX55 - dump registers V0 to VX in memory, starting from I
    DumpRegistersToMemoryAtIndexRegister(usize),
    /// FX65 - load memory starting from I into V0 to VX
    LoadMemoryToRegistersAtIndexRegister(usize),
}

impl Instruction {
    pub fn from_opcode(opcode: u16) -> Result<Self, Error> {
        let n1 = ((opcode >> 12) & 0xF) as usize;
        let n2 = ((opcode >> 8) & 0xF) as usize;
        let n3 = ((opcode >> 4) & 0xF) as usize;
        let n4 = (opcode & 0xF) as usize;

        match (n1, n2, n3, n4) {
            (0, 0, 0xE, 0) => Ok(Instruction::ClearScreen),
            (0, 0, 0xE, 0xE) => Ok(Instruction::ReturnFromSubroutine),
            (1, n2, n3, n4) => {
                let address = opcode & 0x0FFF;
                Ok(Instruction::Jump(address))
            }
            (2, n2, n3, n4) => {
                let address = opcode & 0x0FFF;
                Ok(Instruction::Call(address))
            }
            (3, n2, n3, n4) => {
                let byte_value = (opcode & 0x00FF) as u8;
                Ok(Instruction::SkipIfEqualByte(n2, byte_value))
            }
            (4, n2, n3, n4) => {
                let byte_value = (opcode & 0x00FF) as u8;
                Ok(Instruction::SkipIfNotEqualByte(n2, byte_value))
            }
            (5, n2, n3, 0) => Ok(Instruction::SkipIfRegistersEqual(n2, n3)),
            (6, n2, n3, n4) => {
                let byte_value = (opcode & 0x00FF) as u8;
                Ok(Instruction::SetRegisterToValue(n2, byte_value))
            }
            (7, n2, n3, n4) => {
                let byte_value = (opcode & 0x00FF) as u8;
                Ok(Instruction::AddToRegister(n2, byte_value))
            }
            (8, n2, n3, 0) => Ok(Instruction::SetRegisterToRegisterValue(n2, n3)),
            (8, n2, n3, 1) => Ok(Instruction::RegistersBitwiseOr(n2, n3)),
            (8, n2, n3, 2) => Ok(Instruction::RegistersBitwiseAnd(n2, n3)),
            (8, n2, n3, 3) => Ok(Instruction::RegistersBitwiseXor(n2, n3)),
            (8, n2, n3, 4) => Ok(Instruction::RegistersSumWithOverflow(n2, n3)),
            (8, n2, n3, 5) => Ok(Instruction::SubstractRegisterFromRegisterValue(n2, n3)),
            (8, n2, n3, 6) => Ok(Instruction::ShiftRegisterBitsRight(n2)),
            (8, n2, n3, 7) => Ok(Instruction::SubstractRegisterValueFromRegister(n2, n3)),
            (8, n2, n3, 0xE) => Ok(Instruction::ShiftRegisterBitsLeft(n2)),
            (9, n2, n3, 0) => Ok(Instruction::SkipIfRegistersNotEqual(n2, n3)),
            (0xA, n2, n3, n4) => {
                let address = opcode & 0x0FFF;
                Ok(Instruction::SetIndexRegisterToValue(address))
            }
            (0xB, n2, n3, n4) => {
                let address = opcode & 0x0FFF;
                Ok(Instruction::JumpByValue(address))
            }
            (0xC, n2, n3, n4) => {
                let byte_value = (opcode & 0x00FF) as u8;
                Ok(Instruction::SetRegisterToRandAndValue(n2, byte_value))
            }
            (0xD, vx, vy, n) => Ok(Instruction::DrawSprite(vx, vy, n as u8)),
            (0xE, n2, 9, 0xE) => Ok(Instruction::SkipIfKeyEqualsRegister(n2)),
            (0xE, n2, 0xA, 1) => Ok(Instruction::SkipIfKeyNotEqualsRegister(n2)),
            (0xF, n2, 0, 7) => Ok(Instruction::SetRegisterToDelayTimerValue(n2)),
            (0xF, n2, 1, 5) => Ok(Instruction::SetDelayTimerToRegisterValue(n2)),
            (0xF, n2, 1, 8) => Ok(Instruction::SetSoundTimerToRegisterValue(n2)),
            (0xF, n2, 2, 9) => Ok(Instruction::SetIndexRegisterToSpriteForRegister(n2)),
            (0xF, n2, 3, 3) => Ok(Instruction::StoreBinaryCodedDecimalAtIndexRegisterValue(n2)),
            (0xF, n2, 1, 0xE) => Ok(Instruction::AddRegisterToIndexRegister(n2)),
            (0xF, n2, 5, 5) => Ok(Instruction::DumpRegistersToMemoryAtIndexRegister(n2)),
            (0xF, n2, 6, 5) => Ok(Instruction::LoadMemoryToRegistersAtIndexRegister(n2)),
            (_, _, _, _) => Err(Error::UnknownOpcode(opcode)),
        }
    }
}
