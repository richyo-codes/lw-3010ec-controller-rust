use anyhow::Result;
use serialport::{SerialPortInfo, SerialPortType};

/// CH340 USB VID/PID (from the Python implementation)
const CH340_VID: u16 = 0x1A86;
const CH340_PID: u16 = 0x7523;

/// Get a list of available serial ports
pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
    serialport::available_ports().map_err(|e| anyhow::anyhow!("Failed to list serial ports: {}", e))
}

/// Check if a port is a CH340 USB device
fn is_ch340(port: &SerialPortInfo) -> bool {
    if let SerialPortType::UsbPort(usb_info) = &port.port_type {
        usb_info.vid == CH340_VID && usb_info.pid == CH340_PID
    } else {
        false
    }
}

/// Find all CH340 USB ports (likely PSUs)
pub fn find_psu_ports() -> Result<Vec<SerialPortInfo>> {
    let all_ports = list_ports()?;
    let psu_ports: Vec<_> = all_ports.iter().filter(|p| is_ch340(p)).cloned().collect();
    Ok(psu_ports)
}

/// Open a port with specified settings
pub fn open_port(port_name: &str, baud_rate: u32) -> Result<Box<dyn serialport::SerialPort>> {
    let port = serialport::new(port_name, baud_rate)
        .timeout(std::time::Duration::from_millis(1000))
        .flow_control(serialport::FlowControl::None)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .open()
        .map_err(|e| anyhow::anyhow!("Failed to open port {}: {}", port_name, e))?;

    Ok(port)
}
