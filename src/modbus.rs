//! Modbus RTU protocol layer.
//!
//! Protocol logic (CRC, frame building, response parsing) is implemented in
//! `lw3010ec-core`. This module adds serial-port I/O on top of that.

use serialport::SerialPort;
use std::time::Duration;

pub use lw3010ec_core::*;

/// Read exactly `count` bytes from a serial port into `buf`.
fn read_exact_bytes(port: &mut dyn SerialPort, buf: &mut [u8]) -> Result<usize, String> {
    let mut total_read = 0usize;

    while total_read < buf.len() {
        match port.read(&mut buf[total_read..]) {
            Ok(n) => {
                if n == 0 {
                    return Ok(total_read);
                }
                total_read += n;
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // A timeout terminates the current RTU frame. Preserve any
                // partial response so the parser can report the real issue.
                return Ok(total_read);
            }
            Err(e) => {
                return Err(format!("Serial read error: {} (kind: {:?})", e, e.kind()));
            }
        }
    }

    Ok(total_read)
}

/// Read bytes that arrive immediately after a request.
///
/// Some PSU/adapter combinations echo the eight-byte request and others return
/// the device response directly. The caller compares these bytes with the
/// request instead of blindly discarding a valid response.
fn read_initial_frame(
    port: &mut dyn SerialPort,
    request_len: usize,
    timeout: Duration,
) -> Result<Vec<u8>, String> {
    let echo_timeout = Duration::from_millis(100);
    port.set_timeout(echo_timeout)
        .map_err(|e| format!("set_timeout error: {}", e))?;

    let mut buf = vec![0u8; request_len];
    let read_result = read_exact_bytes(port, &mut buf);

    port.set_timeout(timeout)
        .map_err(|e| format!("set_timeout error: {}", e))?;

    let bytes_read = read_result?;
    buf.truncate(bytes_read);
    Ok(buf)
}

/// Read holding registers from a serial port
pub fn read_holding_registers(
    port: &mut dyn SerialPort,
    unit_id: u8,
    address: u16,
    quantity: u16,
    timeout: Duration,
) -> Result<Vec<u16>, String> {
    let frame = build_frame(unit_id, MODBUS_READ_HOLDING, address, quantity);
    let expected_response_size = 3 + (quantity as usize) * 2 + 2;

    port.write_all(&frame).map_err(|e| {
        format!(
            "Failed to write modbus request: {} (kind: {:?})",
            e,
            e.kind()
        )
    })?;
    port.flush()
        .map_err(|e| format!("Failed to flush serial port: {} (kind: {:?})", e, e.kind()))?;

    let initial = read_initial_frame(port, frame.len(), timeout)?;
    let response_prefix = if initial == frame {
        // This was an actual request echo.
        &[][..]
    } else {
        // No echo: these bytes are the beginning (usually all) of the reply.
        initial.as_slice()
    };

    let mut buf = vec![0u8; expected_response_size];
    if response_prefix.len() > buf.len() {
        return Err(format!(
            "Unexpected response prefix: got {} bytes, expected at most {}",
            response_prefix.len(),
            buf.len()
        ));
    }
    buf[..response_prefix.len()].copy_from_slice(response_prefix);
    let additional = read_exact_bytes(port, &mut buf[response_prefix.len()..])?;
    let bytes_read = response_prefix.len() + additional;

    if bytes_read == 0 {
        return Err("Timeout: no response from device".to_string());
    }

    let response = parse_response(&buf[..bytes_read], unit_id, MODBUS_READ_HOLDING, quantity)?;
    Ok(response.registers)
}

/// Write a single holding register
pub fn write_single_register(
    port: &mut dyn SerialPort,
    unit_id: u8,
    address: u16,
    value: u16,
    timeout: Duration,
) -> Result<(), String> {
    let frame = build_frame(unit_id, MODBUS_WRITE_SINGLE, address, value);

    port.write_all(&frame).map_err(|e| {
        format!(
            "Failed to write modbus request: {} (kind: {:?})",
            e,
            e.kind()
        )
    })?;
    port.flush()
        .map_err(|e| format!("Failed to flush serial port: {} (kind: {:?})", e, e.kind()))?;

    let mut response = [0u8; 8];
    let initial = read_initial_frame(port, frame.len(), timeout)?;
    let bytes_read = if initial.is_empty() {
        read_exact_bytes(port, &mut response)?
    } else {
        if initial.len() > response.len() {
            return Err(format!(
                "Unexpected write response length: {} bytes",
                initial.len()
            ));
        }
        response[..initial.len()].copy_from_slice(&initial);
        initial.len() + read_exact_bytes(port, &mut response[initial.len()..])?
    };
    if bytes_read == 0 {
        return Err("Timeout: no write acknowledgement from device".to_string());
    }
    parse_write_single_response(&response[..bytes_read], unit_id, address, value)?;

    Ok(())
}
