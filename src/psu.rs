//! PSU controller operations.
//!
//! Uses `lw3010ec_core` for the Modbus protocol and `modbus` for serial I/O.

use crate::modbus;
use serialport::SerialPort;
use std::time::Duration;

/// PSU status snapshot
#[derive(Debug, Clone)]
pub struct PsuStatus {
    pub voltage: f32,
    pub current: f32,
    pub output: bool,
}

impl std::fmt::Display for PsuStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "V: {:.2}V | I: {:.3}A | Output: {}",
            self.voltage, self.current, self.output
        )
    }
}

/// Static methods for PSU operations
pub struct PsuController;

impl PsuController {
    fn validate_setpoint(value: f32, maximum: f32, unit: &str) -> Result<(), String> {
        if !value.is_finite() || !(0.0..=maximum).contains(&value) {
            return Err(format!(
                "{} must be a finite value between 0 and {}{}",
                unit, maximum, unit
            ));
        }
        Ok(())
    }

    /// Set voltage (0-30V). Value is multiplied by 100 for the register.
    pub fn set_voltage(
        unit_id: u8,
        port: &mut dyn SerialPort,
        volts: f32,
        timeout: Duration,
    ) -> Result<(), String> {
        Self::validate_setpoint(volts, 30.0, "V")?;
        let raw_value = (volts * 100.0).round() as u16;
        modbus::write_single_register(port, unit_id, modbus::REG_VOLTAGE_WRITE, raw_value, timeout)
            .map_err(|e| format!("Failed to set voltage to {:.2}V: {}", volts, e))
    }

    /// Get measured voltage from PSU
    pub fn get_voltage(
        unit_id: u8,
        port: &mut dyn SerialPort,
        timeout: Duration,
    ) -> Result<f32, String> {
        let registers =
            modbus::read_holding_registers(port, unit_id, modbus::REG_VOLTAGE_READ, 1, timeout)?;
        Ok(modbus::decode_voltage(registers[0]))
    }

    /// Set current limit (0-10A). Value is multiplied by 100 for the register.
    pub fn set_current(
        unit_id: u8,
        port: &mut dyn SerialPort,
        amps: f32,
        timeout: Duration,
    ) -> Result<(), String> {
        Self::validate_setpoint(amps, 10.0, "A")?;
        let raw_value = (amps * 100.0).round() as u16;
        modbus::write_single_register(port, unit_id, modbus::REG_CURRENT_WRITE, raw_value, timeout)
            .map_err(|e| format!("Failed to set current to {:.3}A: {}", amps, e))
    }

    /// Get measured current from PSU
    pub fn get_current(
        unit_id: u8,
        port: &mut dyn SerialPort,
        timeout: Duration,
    ) -> Result<f32, String> {
        let registers =
            modbus::read_holding_registers(port, unit_id, modbus::REG_CURRENT_READ, 1, timeout)?;
        Ok(modbus::decode_current(registers[0]))
    }

    /// Get output status
    pub fn get_output(
        unit_id: u8,
        port: &mut dyn SerialPort,
        timeout: Duration,
    ) -> Result<bool, String> {
        let registers =
            modbus::read_holding_registers(port, unit_id, modbus::REG_OUTPUT_READ, 1, timeout)?;
        Ok(modbus::decode_output(registers[0]))
    }

    /// Set output on/off
    pub fn set_output(
        unit_id: u8,
        port: &mut dyn SerialPort,
        on: bool,
        timeout: Duration,
    ) -> Result<(), String> {
        let value = if on { 1u16 } else { 0u16 };
        modbus::write_single_register(port, unit_id, modbus::REG_OUTPUT_WRITE, value, timeout)
            .map_err(|e| format!("Failed to set output to {}: {}", on, e))
    }

    /// Read full status from PSU
    pub fn get_status(
        unit_id: u8,
        port: &mut dyn SerialPort,
        timeout: Duration,
    ) -> Result<PsuStatus, String> {
        // The LW-3010EC only reliably responds to single-register reads even
        // though these registers are contiguous.
        let voltage = Self::get_voltage(unit_id, port, timeout)?;
        let current = Self::get_current(unit_id, port, timeout)?;
        let output = Self::get_output(unit_id, port, timeout)?;
        Ok(PsuStatus {
            voltage,
            current,
            output,
        })
    }
}
