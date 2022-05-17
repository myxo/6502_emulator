use crate::c64::C64;

use serde::{Deserialize, Serialize};

pub enum Request {
    CpuState,
    VicState,
    Memory,
    Dissasembly,
    ScreenTexture,
}

#[derive(Serialize, Deserialize)]
struct CpuState {
    a: u8,
    y: u8,
    x: u8,
}

pub fn process_command(c64: &C64, req: Request) -> Result<String, String> {
    let responce = match req {
        Request::CpuState => {
            let state = CpuState {
                a: c64.cpu.reg.a,
                x: c64.cpu.reg.x,
                y: c64.cpu.reg.y,
            };
            serde_json::to_string(&state).unwrap()
        },
        _ => todo!(),
    };

    Ok(responce)
}


