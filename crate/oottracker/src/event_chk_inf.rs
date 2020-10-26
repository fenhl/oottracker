use {
    std::{
        convert::TryFrom,
        iter,
    },
    bitflags::bitflags,
    byteorder::{
        BigEndian,
        ByteOrder as _,
    },
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EventChkInf(
    EventChkInf0,
    EventChkInf1,
    EventChkInf2,
    EventChkInf3,
    EventChkInf4,
    EventChkInf5,
    EventChkInf6,
    EventChkInf7,
    EventChkInf8,
    pub EventChkInf9,
    EventChkInfA,
    EventChkInfB,
    EventChkInfC,
    EventChkInfD,
);

impl TryFrom<Vec<u8>> for EventChkInf {
    type Error = Vec<u8>;

    fn try_from(raw_data: Vec<u8>) -> Result<EventChkInf, Vec<u8>> {
        if raw_data.len() != 28 { return Err(raw_data) }
        Ok(EventChkInf(
            EventChkInf0::try_from(&raw_data[0x00..0x02]).map_err(|()| raw_data.clone())?,
            EventChkInf1::try_from(&raw_data[0x02..0x04]).map_err(|()| raw_data.clone())?,
            EventChkInf2::try_from(&raw_data[0x04..0x06]).map_err(|()| raw_data.clone())?,
            EventChkInf3::try_from(&raw_data[0x06..0x08]).map_err(|()| raw_data.clone())?,
            EventChkInf4::try_from(&raw_data[0x08..0x0a]).map_err(|()| raw_data.clone())?,
            EventChkInf5::try_from(&raw_data[0x0a..0x0c]).map_err(|()| raw_data.clone())?,
            EventChkInf6::try_from(&raw_data[0x0c..0x0e]).map_err(|()| raw_data.clone())?,
            EventChkInf7::try_from(&raw_data[0x0e..0x10]).map_err(|()| raw_data.clone())?,
            EventChkInf8::try_from(&raw_data[0x10..0x12]).map_err(|()| raw_data.clone())?,
            EventChkInf9::try_from(&raw_data[0x12..0x14]).map_err(|()| raw_data.clone())?,
            EventChkInfA::try_from(&raw_data[0x14..0x16]).map_err(|()| raw_data.clone())?,
            EventChkInfB::try_from(&raw_data[0x16..0x18]).map_err(|()| raw_data.clone())?,
            EventChkInfC::try_from(&raw_data[0x18..0x1a]).map_err(|()| raw_data.clone())?,
            EventChkInfD::try_from(&raw_data[0x1a..0x1c]).map_err(|()| raw_data.clone())?,
        ))
    }
}

impl<'a> From<&'a EventChkInf> for Vec<u8> {
    fn from(event_chk_inf: &EventChkInf) -> Vec<u8> {
        iter::empty()
            .chain(Vec::from(event_chk_inf.0))
            .chain(Vec::from(event_chk_inf.1))
            .chain(Vec::from(event_chk_inf.2))
            .chain(Vec::from(event_chk_inf.3))
            .chain(Vec::from(event_chk_inf.4))
            .chain(Vec::from(event_chk_inf.5))
            .chain(Vec::from(event_chk_inf.6))
            .chain(Vec::from(event_chk_inf.7))
            .chain(Vec::from(event_chk_inf.8))
            .chain(Vec::from(event_chk_inf.9))
            .chain(Vec::from(event_chk_inf.10))
            .chain(Vec::from(event_chk_inf.11))
            .chain(Vec::from(event_chk_inf.12))
            .chain(Vec::from(event_chk_inf.13))
            .collect()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf0;

impl<'a> TryFrom<&'a [u8]> for EventChkInf0 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf0, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf0)
    }
}

impl From<EventChkInf0> for Vec<u8> {
    fn from(_: EventChkInf0) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf1;

impl<'a> TryFrom<&'a [u8]> for EventChkInf1 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf1, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf1)
    }
}

impl From<EventChkInf1> for Vec<u8> {
    fn from(_: EventChkInf1) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf2;

impl<'a> TryFrom<&'a [u8]> for EventChkInf2 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf2, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf2)
    }
}

impl From<EventChkInf2> for Vec<u8> {
    fn from(_: EventChkInf2) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf3;

impl<'a> TryFrom<&'a [u8]> for EventChkInf3 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf3, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf3)
    }
}

impl From<EventChkInf3> for Vec<u8> {
    fn from(_: EventChkInf3) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf4;

impl<'a> TryFrom<&'a [u8]> for EventChkInf4 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf4, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf4)
    }
}

impl From<EventChkInf4> for Vec<u8> {
    fn from(_: EventChkInf4) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf5;

impl<'a> TryFrom<&'a [u8]> for EventChkInf5 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf5, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf5)
    }
}

impl From<EventChkInf5> for Vec<u8> {
    fn from(_: EventChkInf5) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf6;

impl<'a> TryFrom<&'a [u8]> for EventChkInf6 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf6, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf6)
    }
}

impl From<EventChkInf6> for Vec<u8> {
    fn from(_: EventChkInf6) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf7;

impl<'a> TryFrom<&'a [u8]> for EventChkInf7 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf7, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf7)
    }
}

impl From<EventChkInf7> for Vec<u8> {
    fn from(_: EventChkInf7) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInf8;

impl<'a> TryFrom<&'a [u8]> for EventChkInf8 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf8, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf8)
    }
}

impl From<EventChkInf8> for Vec<u8> {
    fn from(_: EventChkInf8) -> Vec<u8> {
        vec![0; 2]
    }
}

bitflags! {
    #[derive(Default)]
    pub struct EventChkInf9: u16 {
        const SCARECROW_SONG = 0x1000;
    }
}

impl<'a> TryFrom<&'a [u8]> for EventChkInf9 {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInf9, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInf9::from_bits_truncate(BigEndian::read_u16(&raw_data)))
    }
}

impl From<EventChkInf9> for Vec<u8> {
    fn from(eci9: EventChkInf9) -> Vec<u8> {
        eci9.bits().to_be_bytes().into()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInfA;

impl<'a> TryFrom<&'a [u8]> for EventChkInfA {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInfA, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInfA)
    }
}

impl From<EventChkInfA> for Vec<u8> {
    fn from(_: EventChkInfA) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInfB;

impl<'a> TryFrom<&'a [u8]> for EventChkInfB {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInfB, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInfB)
    }
}

impl From<EventChkInfB> for Vec<u8> {
    fn from(_: EventChkInfB) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInfC;

impl<'a> TryFrom<&'a [u8]> for EventChkInfC {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInfC, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInfC)
    }
}

impl From<EventChkInfC> for Vec<u8> {
    fn from(_: EventChkInfC) -> Vec<u8> {
        vec![0; 2]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventChkInfD;

impl<'a> TryFrom<&'a [u8]> for EventChkInfD {
    type Error = ();

    fn try_from(raw_data: &[u8]) -> Result<EventChkInfD, ()> {
        if raw_data.len() != 2 { return Err(()) }
        Ok(EventChkInfD)
    }
}

impl From<EventChkInfD> for Vec<u8> {
    fn from(_: EventChkInfD) -> Vec<u8> {
        vec![0; 2]
    }
}
