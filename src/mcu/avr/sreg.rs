use bitfield::bitfield;

bitfield!{
    pub struct StatusRegister(u8);
    impl Debug;
    pub c, set_c: 0;
    pub z, set_z: 1;
    pub n, set_n: 2;
    pub v, set_v: 3;
    pub s, set_s: 4;
    pub h, set_h: 5;
    pub t, set_t: 6;
    pub i, set_i: 7;
}