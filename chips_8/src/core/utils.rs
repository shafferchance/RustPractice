pub fn get_kk(value: &u16) -> u16 {
    return value & 0x00FF;
}

pub fn get_nibble(value: &u16) -> u16 {
    return value & 0x000F;
}

pub fn get_nnn(value: &u16) -> u16 {
    return value & 0x0FFF;
}

pub fn get_x(value: &u16) -> u16 {
    return (value & 0x0F00) >> 8;
}

pub fn get_y(value: &u16) -> u16 {
    return (value & 0x00F0) >> 4;
}
