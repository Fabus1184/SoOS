use alloc::vec::Vec;

use self::class::ClassCode;
mod class;

fn pci_config_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let address = 0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xfc);

    unsafe {
        x86_64::instructions::port::PortWrite::write_to_port(0xcf8, address);
        x86_64::instructions::port::PortRead::read_from_port(0xcfc)
    }
}

pub fn scan() -> anyhow::Result<Vec<PCIDevice>> {
    Ok((0..=255)
        .flat_map(|bus| (0..=31).map(move |device| (bus, device)))
        .flat_map(|(bus, device)| (0..=7).map(move |function| (bus, device, function)))
        .map(|(bus, device, function)| PCIDevice::from_bus_device(bus, device, function))
        .collect::<anyhow::Result<Vec<Option<PCIDevice>>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<PCIDevice>>())
}

#[derive(Debug)]
pub struct PCIDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header: PCIHeader,
}

impl PCIDevice {
    fn from_bus_device(bus: u8, device: u8, function: u8) -> anyhow::Result<Option<Self>> {
        let header = PCIHeader::from_bus_device(bus, device, function)?;
        if let Some(header) = header {
            Ok(Some(Self {
                bus,
                device,
                function,
                header,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct PCIHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    command: u16,
    status: u16,
    revision_id: u8,
    pub prog_if: u8,
    pub class: ClassCode,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: HeaderType,
    bist: u8,
}

impl PCIHeader {
    fn from_bus_device(bus: u8, device: u8, function: u8) -> anyhow::Result<Option<Self>> {
        let vendor_id = pci_config_read_word(bus, device, function, 0);
        if vendor_id == 0xffff {
            return Ok(None);
        }

        let device_id = pci_config_read_word(bus, device, function, 2);
        let command = pci_config_read_word(bus, device, function, 4);
        let status = pci_config_read_word(bus, device, function, 6);
        let revision_id = pci_config_read_word(bus, device, function, 8) as u8;
        let prog_if = pci_config_read_word(bus, device, function, 9) as u8;
        let subclass = pci_config_read_word(bus, device, function, 10) as u8;

        let class = ClassCode::from_u8(
            pci_config_read_word(bus, device, function, 11) as u8,
            subclass,
        );

        let cache_line_size = pci_config_read_word(bus, device, function, 12) as u8;
        let latency_timer = pci_config_read_word(bus, device, function, 13) as u8;
        let header_type = HeaderType::from_bus_device(bus, function, device)?;
        let bist = pci_config_read_word(bus, device, function, 15) as u8;

        Ok(Some(Self {
            vendor_id,
            device_id,
            command,
            status,
            revision_id,
            prog_if,
            class,
            cache_line_size,
            latency_timer,
            header_type,
            bist,
        }))
    }
}

#[derive(Debug)]
enum HeaderType {
    Normal {
        bar0: u32,
        bar1: u32,
        bar2: u32,
        bar3: u32,
        bar4: u32,
        bar5: u32,
        cardbus_cis_pointer: u32,
        subsystem_vendor_id: u16,
        subsystem_id: u16,
        expansion_rom_base_address: u32,
        capabilities_pointer: u8,
        reserved: [u8; 7],
        interrupt_line: u8,
        interrupt_pin: u8,
        min_grant: u8,
        max_latency: u8,
    },
    Bridge {
        bar0: u32,
        bar1: u32,
        primary_bus_number: u8,
        secondary_bus_number: u8,
        subordinate_bus_number: u8,
        secondary_latency_timer: u8,
        io_base: u8,
        io_limit: u8,
        secondary_status: u16,
        memory_base: u16,
        memory_limit: u16,
        prefetchable_memory_base: u16,
        prefetchable_memory_limit: u16,
        prefetchable_base_upper_32_bits: u32,
        prefetchable_limit_upper_32_bits: u32,
        io_base_upper_16_bits: u16,
        io_limit_upper_16_bits: u16,
        capabilities_pointer: u8,
        reserved: [u8; 3],
        expansion_rom_base_address: u32,
        interrupt_line: u8,
        interrupt_pin: u8,
        bridge_control: u16,
    },
    Cardbus {
        cardbus_socket_exca_base_address: u32,
        capabilities_pointer: u8,
        reserved: [u8; 3],
        secondary_status: u16,
        pci_bus_number: u8,
        cardbus_bus_number: u8,
        subordinate_bus_number: u8,
        cardbus_latency_timer: u8,
        memory_base_0: u32,
        memory_limit_0: u32,
        memory_base_1: u32,
        memory_limit_1: u32,
        io_base_0: u32,
        io_limit_0: u32,
        io_base_1: u32,
        io_limit_1: u32,
        interrupt_line: u8,
        interrupt_pin: u8,
        bridge_control: u16,
        subsystem_device_id: u16,
        subsystem_vendor_id: u16,
        legacy_base_address: u32,
    },
}

impl HeaderType {
    fn from_bus_device(bus: u8, device: u8, function: u8) -> anyhow::Result<Self> {
        let header_type = pci_config_read_word(bus, device, 0, 14) as u8;
        match header_type & 0x7f {
            0 => {
                let bar0 = pci_config_read_word(bus, device, function, 16) as u32;
                let bar1 = pci_config_read_word(bus, device, function, 18) as u32;
                let bar2 = pci_config_read_word(bus, device, function, 20) as u32;
                let bar3 = pci_config_read_word(bus, device, function, 22) as u32;
                let bar4 = pci_config_read_word(bus, device, function, 24) as u32;
                let bar5 = pci_config_read_word(bus, device, function, 26) as u32;
                let cardbus_cis_pointer = pci_config_read_word(bus, device, function, 28) as u32;
                let subsystem_vendor_id = pci_config_read_word(bus, device, function, 44);
                let subsystem_id = pci_config_read_word(bus, device, function, 46);
                let expansion_rom_base_address =
                    pci_config_read_word(bus, device, function, 48) as u32;
                let capabilities_pointer = pci_config_read_word(bus, device, function, 52) as u8;
                let reserved = [
                    pci_config_read_word(bus, device, function, 53) as u8,
                    pci_config_read_word(bus, device, function, 54) as u8,
                    pci_config_read_word(bus, device, function, 55) as u8,
                    pci_config_read_word(bus, device, function, 56) as u8,
                    pci_config_read_word(bus, device, function, 57) as u8,
                    pci_config_read_word(bus, device, function, 58) as u8,
                    pci_config_read_word(bus, device, function, 59) as u8,
                ];
                let interrupt_line = pci_config_read_word(bus, device, function, 60) as u8;
                let interrupt_pin = pci_config_read_word(bus, device, function, 61) as u8;
                let min_grant = pci_config_read_word(bus, device, function, 62) as u8;
                let max_latency = pci_config_read_word(bus, device, function, 63) as u8;

                Ok(Self::Normal {
                    bar0,
                    bar1,
                    bar2,
                    bar3,
                    bar4,
                    bar5,
                    cardbus_cis_pointer,
                    subsystem_vendor_id,
                    subsystem_id,
                    expansion_rom_base_address,
                    capabilities_pointer,
                    reserved,
                    interrupt_line,
                    interrupt_pin,
                    min_grant,
                    max_latency,
                })
            }
            1 => {
                let bar0 = pci_config_read_word(bus, device, function, 16) as u32;
                let bar1 = pci_config_read_word(bus, device, function, 18) as u32;
                let primary_bus_number = pci_config_read_word(bus, device, function, 20) as u8;
                let secondary_bus_number = pci_config_read_word(bus, device, function, 21) as u8;
                let subordinate_bus_number = pci_config_read_word(bus, device, function, 22) as u8;
                let secondary_latency_timer = pci_config_read_word(bus, device, function, 23) as u8;
                let io_base = pci_config_read_word(bus, device, function, 24) as u8;
                let io_limit = pci_config_read_word(bus, device, function, 25) as u8;
                let secondary_status = pci_config_read_word(bus, device, function, 26);
                let memory_base = pci_config_read_word(bus, device, function, 28);
                let memory_limit = pci_config_read_word(bus, device, function, 30);
                let prefetchable_memory_base =
                    pci_config_read_word(bus, device, function, 32);
                let prefetchable_memory_limit =
                    pci_config_read_word(bus, device, function, 34);
                let prefetchable_base_upper_32_bits =
                    pci_config_read_word(bus, device, function, 36) as u32;
                let prefetchable_limit_upper_32_bits =
                    pci_config_read_word(bus, device, function, 40) as u32;
                let io_base_upper_16_bits = pci_config_read_word(bus, device, function, 48);
                let io_limit_upper_16_bits = pci_config_read_word(bus, device, function, 50);
                let capabilities_pointer = pci_config_read_word(bus, device, function, 52) as u8;
                let reserved = [
                    pci_config_read_word(bus, device, function, 53) as u8,
                    pci_config_read_word(bus, device, function, 54) as u8,
                    pci_config_read_word(bus, device, function, 55) as u8,
                ];
                let expansion_rom_base_address =
                    pci_config_read_word(bus, device, function, 56) as u32;
                let interrupt_line = pci_config_read_word(bus, device, function, 60) as u8;
                let interrupt_pin = pci_config_read_word(bus, device, function, 61) as u8;
                let bridge_control = pci_config_read_word(bus, device, function, 62);

                Ok(Self::Bridge {
                    bar0,
                    bar1,
                    primary_bus_number,
                    secondary_bus_number,
                    subordinate_bus_number,
                    secondary_latency_timer,
                    io_base,
                    io_limit,
                    secondary_status,
                    memory_base,
                    memory_limit,
                    prefetchable_memory_base,
                    prefetchable_memory_limit,
                    prefetchable_base_upper_32_bits,
                    prefetchable_limit_upper_32_bits,
                    io_base_upper_16_bits,
                    io_limit_upper_16_bits,
                    capabilities_pointer,
                    reserved,
                    expansion_rom_base_address,
                    interrupt_line,
                    interrupt_pin,
                    bridge_control,
                })
            }
            2 => {
                let cardbus_socket_exca_base_address =
                    pci_config_read_word(bus, device, function, 16) as u32;
                let capabilities_pointer = pci_config_read_word(bus, device, function, 52) as u8;
                let reserved = [
                    pci_config_read_word(bus, device, function, 53) as u8,
                    pci_config_read_word(bus, device, function, 54) as u8,
                    pci_config_read_word(bus, device, function, 55) as u8,
                ];
                let secondary_status = pci_config_read_word(bus, device, function, 56);
                let pci_bus_number = pci_config_read_word(bus, device, function, 58) as u8;
                let cardbus_bus_number = pci_config_read_word(bus, device, function, 59) as u8;
                let subordinate_bus_number = pci_config_read_word(bus, device, function, 60) as u8;
                let cardbus_latency_timer = pci_config_read_word(bus, device, function, 61) as u8;
                let memory_base_0 = pci_config_read_word(bus, device, function, 64) as u32;
                let memory_limit_0 = pci_config_read_word(bus, device, function, 68) as u32;
                let memory_base_1 = pci_config_read_word(bus, device, function, 72) as u32;
                let memory_limit_1 = pci_config_read_word(bus, device, function, 76) as u32;
                let io_base_0 = pci_config_read_word(bus, device, function, 80) as u32;
                let io_limit_0 = pci_config_read_word(bus, device, function, 84) as u32;
                let io_base_1 = pci_config_read_word(bus, device, function, 88) as u32;
                let io_limit_1 = pci_config_read_word(bus, device, function, 92) as u32;
                let interrupt_line = pci_config_read_word(bus, device, function, 96) as u8;
                let interrupt_pin = pci_config_read_word(bus, device, function, 97) as u8;
                let bridge_control = pci_config_read_word(bus, device, function, 98);
                let subsystem_device_id = pci_config_read_word(bus, device, function, 106);
                let subsystem_vendor_id = pci_config_read_word(bus, device, function, 108);
                let legacy_base_address = pci_config_read_word(bus, device, function, 112) as u32;

                Ok(Self::Cardbus {
                    cardbus_socket_exca_base_address,
                    capabilities_pointer,
                    reserved,
                    secondary_status,
                    pci_bus_number,
                    cardbus_bus_number,
                    subordinate_bus_number,
                    cardbus_latency_timer,
                    memory_base_0,
                    memory_limit_0,
                    memory_base_1,
                    memory_limit_1,
                    io_base_0,
                    io_limit_0,
                    io_base_1,
                    io_limit_1,
                    interrupt_line,
                    interrupt_pin,
                    bridge_control,
                    subsystem_device_id,
                    subsystem_vendor_id,
                    legacy_base_address,
                })
            }
            _ => Err(anyhow::anyhow!("Unknown header type: {}", header_type)),
        }
    }
}
