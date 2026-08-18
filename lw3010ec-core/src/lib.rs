//! Core Modbus RTU protocol logic for the LW-3010EC power supply.
//!
//! This crate is pure Rust with no platform dependencies. It can be compiled
//! to native binaries or to WebAssembly for use in browsers.

use std::sync::OnceLock;

// ── Register constants ──────────────────────────────────────────────

/// Modbus function codes
pub const MODBUS_READ_HOLDING: u8 = 0x03;
pub const MODBUS_WRITE_SINGLE: u8 = 0x06;

/// PSU registers
pub const REG_VOLTAGE_WRITE: u16 = 0x1000;
pub const REG_CURRENT_WRITE: u16 = 0x1001;
pub const REG_VOLTAGE_READ: u16 = 0x1002;
pub const REG_CURRENT_READ: u16 = 0x1003;
pub const REG_OUTPUT_READ: u16 = 0x1004;
pub const REG_OUTPUT_WRITE: u16 = 0x1006;

// ── CRC-16/Modbus ──────────────────────────────────────────────────

static CRC_TABLE: OnceLock<[u16; 256]> = OnceLock::new();

fn get_crc_table() -> &'static [u16; 256] {
    CRC_TABLE.get_or_init(|| {
        let mut table = [0u16; 256];
        for i in 0u16..256 {
            let mut crc = 0u16;
            let mut data = i;
            for _ in 0..8 {
                if (crc ^ data) & 1 != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
                data >>= 1;
            }
            table[i as usize] = crc;
        }
        table
    })
}

/// Calculate CRC-16/Modbus checksum for a byte sequence.
pub fn crc16(data: &[u8]) -> u16 {
    let table = get_crc_table();
    let mut crc = 0xFFFFu16;
    for &byte in data {
        let index = ((crc ^ byte as u16) & 0xFF) as usize;
        crc = (crc >> 8) ^ table[index];
    }
    crc
}

// ── Frame building ─────────────────────────────────────────────────

/// Build a Modbus RTU frame.
///
/// `func_code` must be `MODBUS_READ_HOLDING` (0x03) or `MODBUS_WRITE_SINGLE` (0x06).
/// For read operations, `quantity_or_value` is the number of registers.
/// For write operations, `quantity_or_value` is the register value to write.
pub fn build_frame(unit_id: u8, func_code: u8, address: u16, quantity_or_value: u16) -> Vec<u8> {
    assert!(
        func_code == MODBUS_READ_HOLDING || func_code == MODBUS_WRITE_SINGLE,
        "Unsupported function code: 0x{:02X}",
        func_code
    );

    let mut frame = vec![
        unit_id,
        func_code,
        (address >> 8) as u8,
        (address & 0xFF) as u8,
        (quantity_or_value >> 8) as u8,
        (quantity_or_value & 0xFF) as u8,
    ];

    let crc = crc16(&frame);
    frame.push(crc as u8);
    frame.push((crc >> 8) as u8);
    frame
}

// ── Response parsing ───────────────────────────────────────────────

/// Parsed Modbus response data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModbusResponse {
    /// Register values read from the device.
    pub registers: Vec<u16>,
}

/// Parse a Modbus RTU response frame.
///
/// Returns `Ok(ModbusResponse)` with the parsed register values, or an `Err`
/// with a description if the response is invalid.
pub fn parse_response(
    data: &[u8],
    expected_unit_id: u8,
    expected_func_code: u8,
    expected_quantity: u16,
) -> Result<ModbusResponse, String> {
    if data.len() < 5 {
        return Err(format!(
            "Response too short: got {} bytes, need at least 5",
            data.len(),
        ));
    }

    // Validate unit ID
    if data[0] != expected_unit_id {
        return Err(format!(
            "Invalid unit ID: expected {}, got {}",
            expected_unit_id, data[0]
        ));
    }

    // A Modbus exception response contains the requested function code with
    // bit 7 set, followed by a one-byte exception code.
    if data[1] == expected_func_code | 0x80 {
        if data.len() != 5 {
            return Err(format!(
                "Invalid exception response length: got {} bytes, expected 5",
                data.len()
            ));
        }
        let received_crc = u16::from_le_bytes([data[3], data[4]]);
        let calculated_crc = crc16(&data[..3]);
        if received_crc != calculated_crc {
            return Err(format!(
                "Invalid CRC: expected 0x{:04X}, got 0x{:04X}",
                calculated_crc, received_crc
            ));
        }
        return Err(format!("Modbus exception response: code 0x{:02X}", data[2]));
    }

    // Normal response: unit(1) + func(1) + byte_count(1) + data(n*2) + crc(2)
    let expected_size = 3 + (expected_quantity as usize) * 2 + 2;
    if data.len() != expected_size {
        return Err(format!(
            "Invalid response length: got {} bytes, expected {}",
            data.len(),
            expected_size
        ));
    }

    // Validate function code
    if data[1] != expected_func_code {
        return Err(format!(
            "Invalid function code: expected 0x{:02X}, got 0x{:02X}",
            expected_func_code, data[1]
        ));
    }

    // Validate byte count
    let byte_count = data[2] as usize;
    let expected_data_size = (expected_quantity as usize) * 2;
    if byte_count != expected_data_size {
        return Err(format!(
            "Invalid byte count: expected {}, got {}",
            expected_data_size, byte_count
        ));
    }

    let received_crc = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
    let calculated_crc = crc16(&data[..data.len() - 2]);
    if received_crc != calculated_crc {
        return Err(format!(
            "Invalid CRC: expected 0x{:04X}, got 0x{:04X}",
            calculated_crc, received_crc
        ));
    }

    // Parse register values (big-endian)
    let mut registers = Vec::with_capacity(expected_quantity as usize);
    for i in 0..expected_quantity as usize {
        let offset = 3 + i * 2;
        let value = ((data[offset] as u16) << 8) | (data[offset + 1] as u16);
        registers.push(value);
    }

    Ok(ModbusResponse { registers })
}

/// Validate the response to a Modbus "write single register" request.
///
/// The normal response is an exact echo of the six-byte request payload plus
/// its CRC. A successfully written request therefore must be acknowledged by
/// the device rather than merely accepted by the local serial driver.
pub fn parse_write_single_response(
    data: &[u8],
    expected_unit_id: u8,
    expected_address: u16,
    expected_value: u16,
) -> Result<(), String> {
    const RESPONSE_SIZE: usize = 8;
    if data.len() != RESPONSE_SIZE {
        return Err(format!(
            "Invalid write response length: got {} bytes, expected {}",
            data.len(),
            RESPONSE_SIZE
        ));
    }
    if data[0] != expected_unit_id {
        return Err(format!(
            "Invalid unit ID: expected {}, got {}",
            expected_unit_id, data[0]
        ));
    }
    if data[1] == MODBUS_WRITE_SINGLE | 0x80 {
        return Err(format!("Modbus exception response: code 0x{:02X}", data[2]));
    }
    if data[1] != MODBUS_WRITE_SINGLE {
        return Err(format!(
            "Invalid function code: expected 0x{:02X}, got 0x{:02X}",
            MODBUS_WRITE_SINGLE, data[1]
        ));
    }

    let address = u16::from_be_bytes([data[2], data[3]]);
    let value = u16::from_be_bytes([data[4], data[5]]);
    if address != expected_address || value != expected_value {
        return Err(format!(
            "Unexpected write acknowledgement: address 0x{:04X}, value {}",
            address, value
        ));
    }

    let received_crc = u16::from_le_bytes([data[6], data[7]]);
    let calculated_crc = crc16(&data[..6]);
    if received_crc != calculated_crc {
        return Err(format!(
            "Invalid CRC: expected 0x{:04X}, got 0x{:04X}",
            calculated_crc, received_crc
        ));
    }
    Ok(())
}

// ── High-level PSU helpers ─────────────────────────────────────────

/// Build a frame to set voltage. The raw register value is `volts * 100`.
pub fn build_set_voltage_frame(unit_id: u8, volts: f32) -> Vec<u8> {
    let raw = (volts * 100.0).round() as u16;
    build_frame(unit_id, MODBUS_WRITE_SINGLE, REG_VOLTAGE_WRITE, raw)
}

/// Build a frame to set current limit. The raw register value is `amps * 100`.
pub fn build_set_current_frame(unit_id: u8, amps: f32) -> Vec<u8> {
    let raw = (amps * 100.0).round() as u16;
    build_frame(unit_id, MODBUS_WRITE_SINGLE, REG_CURRENT_WRITE, raw)
}

/// Build a frame to set output on/off.
pub fn build_set_output_frame(unit_id: u8, on: bool) -> Vec<u8> {
    let value: u16 = if on { 1 } else { 0 };
    build_frame(unit_id, MODBUS_WRITE_SINGLE, REG_OUTPUT_WRITE, value)
}

/// Build a frame to read a single register.
pub fn build_read_register_frame(unit_id: u8, address: u16) -> Vec<u8> {
    build_frame(unit_id, MODBUS_READ_HOLDING, address, 1)
}

/// Decode voltage from a register value (divides by 100).
pub fn decode_voltage(raw: u16) -> f32 {
    raw as f32 / 100.0
}

/// Decode current from a register value (divides by 100).
pub fn decode_current(raw: u16) -> f32 {
    raw as f32 / 100.0
}

/// Decode output state from a register value.
pub fn decode_output(raw: u16) -> bool {
    raw != 0
}

// ── WebAssembly bindings ───────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn crc16(data: &[u8]) -> u16 {
        super::crc16(data)
    }

    #[wasm_bindgen]
    pub fn build_frame(
        unit_id: u8,
        func_code: u8,
        address: u16,
        quantity_or_value: u16,
    ) -> Vec<u8> {
        super::build_frame(unit_id, func_code, address, quantity_or_value)
    }

    #[wasm_bindgen]
    pub fn parse_response(
        data: &[u8],
        expected_unit_id: u8,
        expected_func_code: u8,
        expected_quantity: u16,
    ) -> JsValue {
        match super::parse_response(
            data,
            expected_unit_id,
            expected_func_code,
            expected_quantity,
        ) {
            Ok(resp) => {
                // Copy the registers into JavaScript-owned storage. A view
                // would point at `resp.registers`, which is dropped when this
                // function returns and can produce stale or corrupted values.
                let arr = js_sys::Uint16Array::new_with_length(resp.registers.len() as u32);
                arr.copy_from(&resp.registers);
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"ok".into(), &JsValue::TRUE).unwrap();
                js_sys::Reflect::set(&obj, &"registers".into(), &JsValue::from(arr)).unwrap();
                obj.into()
            }
            Err(err) => {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"ok".into(), &JsValue::FALSE).unwrap();
                js_sys::Reflect::set(&obj, &"error".into(), &JsValue::from_str(&err)).unwrap();
                obj.into()
            }
        }
    }

    // Register constants as JS-accessible values
    #[wasm_bindgen]
    pub fn reg_voltage_write() -> u16 {
        super::REG_VOLTAGE_WRITE
    }
    #[wasm_bindgen]
    pub fn reg_current_write() -> u16 {
        super::REG_CURRENT_WRITE
    }
    #[wasm_bindgen]
    pub fn reg_voltage_read() -> u16 {
        super::REG_VOLTAGE_READ
    }
    #[wasm_bindgen]
    pub fn reg_current_read() -> u16 {
        super::REG_CURRENT_READ
    }
    #[wasm_bindgen]
    pub fn reg_output_read() -> u16 {
        super::REG_OUTPUT_READ
    }
    #[wasm_bindgen]
    pub fn reg_output_write() -> u16 {
        super::REG_OUTPUT_WRITE
    }

    #[wasm_bindgen]
    pub fn modbus_read_holding() -> u8 {
        super::MODBUS_READ_HOLDING
    }
    #[wasm_bindgen]
    pub fn modbus_write_single() -> u8 {
        super::MODBUS_WRITE_SINGLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_known_values() {
        // The CRC bytes on the wire are 0x21, 0x0A (little-endian 0x0A21).
        let data = [0x01u8, 0x03, 0x10, 0x02, 0x00, 0x01];
        let crc = crc16(&data);
        assert_eq!(crc, 0x0A21);
    }

    #[test]
    fn test_build_read_frame() {
        let frame = build_frame(1, MODBUS_READ_HOLDING, REG_VOLTAGE_READ, 1);
        assert_eq!(frame.len(), 8); // 6 bytes + 2 CRC
        assert_eq!(frame[0], 0x01); // unit ID
        assert_eq!(frame[1], 0x03); // function code
    }

    #[test]
    fn test_parse_response() {
        // Simulate a response: unit=1, func=3, byte_count=2, data=0x04D0 0x007D 0x0001, crc
        let voltage_raw = 1234u16; // 12.34V
        let current_raw = 125u16; // 1.25A
        let output_raw = 1u16;

        let data = [
            0x01,
            0x03,
            0x06, // unit, func, byte_count
            (voltage_raw >> 8) as u8,
            (voltage_raw & 0xFF) as u8,
            (current_raw >> 8) as u8,
            (current_raw & 0xFF) as u8,
            (output_raw >> 8) as u8,
            (output_raw & 0xFF) as u8,
        ];
        let crc = crc16(&data);
        let mut frame = data.to_vec();
        frame.push(crc as u8);
        frame.push((crc >> 8) as u8);

        let resp = parse_response(&frame, 1, 0x03, 3).unwrap();
        assert_eq!(resp.registers.len(), 3);
        assert_eq!(resp.registers[0], voltage_raw);
        assert_eq!(resp.registers[1], current_raw);
        assert_eq!(resp.registers[2], output_raw);
    }

    #[test]
    fn parse_response_rejects_bad_crc_and_extra_data() {
        let mut frame = vec![1, MODBUS_READ_HOLDING, 2, 0, 1];
        let crc = crc16(&frame);
        frame.extend(crc.to_le_bytes());
        assert!(parse_response(&frame, 1, MODBUS_READ_HOLDING, 1).is_ok());

        frame[5] ^= 0xFF;
        assert!(parse_response(&frame, 1, MODBUS_READ_HOLDING, 1)
            .unwrap_err()
            .contains("Invalid CRC"));

        frame.push(0);
        assert!(parse_response(&frame, 1, MODBUS_READ_HOLDING, 1)
            .unwrap_err()
            .contains("length"));
    }

    #[test]
    fn parse_response_reports_valid_exception() {
        let mut frame = vec![1, MODBUS_READ_HOLDING | 0x80, 0x02];
        frame.extend(crc16(&frame).to_le_bytes());
        assert!(parse_response(&frame, 1, MODBUS_READ_HOLDING, 1)
            .unwrap_err()
            .contains("exception response: code 0x02"));
    }

    #[test]
    fn parse_write_response_validates_acknowledgement() {
        let frame = build_frame(1, MODBUS_WRITE_SINGLE, REG_VOLTAGE_WRITE, 1234);
        assert!(parse_write_single_response(&frame, 1, REG_VOLTAGE_WRITE, 1234).is_ok());
        assert!(parse_write_single_response(&frame, 1, REG_VOLTAGE_WRITE, 999).is_err());
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode_voltage(1234), 12.34);
        assert_eq!(decode_current(125), 1.25);
        assert!(decode_output(1));
        assert!(!decode_output(0));
    }
}
