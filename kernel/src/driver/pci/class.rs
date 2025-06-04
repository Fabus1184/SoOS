#[derive(Debug)]
pub enum ClassCode {
    Unclassified(UnclassifiedSubclass),
    MassStorageController(MassStorageControllerSubclass),
    NetworkController(NetworkControllerSubclass),
    DisplayController(DisplayControllerSubclass),
    MultimediaController(MultimediaControllerSubclass),
    MemoryController(MemoryControllerSubclass),
    BridgeDevice(BridgeDeviceSubclass),
    SimpleCommunicationController(SimpleCommunicationControllerSubclass),
    BaseSystemPeripheral(BaseSystemPeripheralSubclass),
    InputDeviceController(InputDeviceControllerSubclass),
    DockingStation(DockingStationSubclass),
    Processor(ProcessorSubclass),
    SerialBusController(SerialBusControllerSubclass),
    WirelessController(WirelessControllerSubclass),
    IntelligentController(IntelligentControllerSubclass),
    SatelliteCommunicationController(SatelliteCommunicationControllerSubclass),
    EncryptionController(EncryptionControllerSubclass),
    SignalProcessingController(SignalProcessingControllerSubclass),
    ProcessingAccelerator,
    NonEssentialInstrumentation,
    Reserved,
    Unknown(u8),
}

impl ClassCode {
    pub(super) fn from_u8(class: u8, subclass: u8) -> Self {
        match class {
            0x00 => Self::Unclassified(UnclassifiedSubclass::from_u8(subclass)),
            0x01 => Self::MassStorageController(MassStorageControllerSubclass::from_u8(subclass)),
            0x02 => Self::NetworkController(NetworkControllerSubclass::from_u8(subclass)),
            0x03 => Self::DisplayController(DisplayControllerSubclass::from_u8(subclass)),
            0x04 => Self::MultimediaController(MultimediaControllerSubclass::from_u8(subclass)),
            0x05 => Self::MemoryController(MemoryControllerSubclass::from_u8(subclass)),
            0x06 => Self::BridgeDevice(BridgeDeviceSubclass::from_u8(subclass)),
            0x07 => Self::SimpleCommunicationController(
                SimpleCommunicationControllerSubclass::from_u8(subclass),
            ),
            0x08 => Self::BaseSystemPeripheral(BaseSystemPeripheralSubclass::from_u8(subclass)),
            0x09 => Self::InputDeviceController(InputDeviceControllerSubclass::from_u8(subclass)),
            0x0a => Self::DockingStation(DockingStationSubclass::from_u8(subclass)),
            0x0b => Self::Processor(ProcessorSubclass::from_u8(subclass)),
            0x0c => Self::SerialBusController(SerialBusControllerSubclass::from_u8(subclass)),
            0x0d => Self::WirelessController(WirelessControllerSubclass::from_u8(subclass)),
            0x0e => Self::IntelligentController(IntelligentControllerSubclass::from_u8(subclass)),
            0x0f => Self::SatelliteCommunicationController(
                SatelliteCommunicationControllerSubclass::from_u8(subclass),
            ),
            0x10 => Self::EncryptionController(EncryptionControllerSubclass::from_u8(subclass)),
            0x11 => Self::SignalProcessingController(SignalProcessingControllerSubclass::from_u8(
                subclass,
            )),
            0x12 => Self::ProcessingAccelerator,
            0x13 => Self::NonEssentialInstrumentation,
            0x14 => Self::Reserved,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum UnclassifiedSubclass {
    VGACompatible,
    NonVGACompatible,
    Unknown(u8),
}

impl UnclassifiedSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::VGACompatible,
            0x01 => Self::NonVGACompatible,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum MassStorageControllerSubclass {
    SCSI,
    IDE,
    Floppy,
    IPI,
    RAID,
    ATA,
    SATA,
    SAS,
    NVM,
    Other,
    Unknown(u8),
}

impl MassStorageControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::SCSI,
            0x01 => Self::IDE,
            0x02 => Self::Floppy,
            0x03 => Self::IPI,
            0x04 => Self::RAID,
            0x05 => Self::ATA,
            0x06 => Self::SATA,
            0x07 => Self::SAS,
            0x08 => Self::NVM,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum NetworkControllerSubclass {
    Ethernet,
    TokenRing,
    FDDI,
    ATM,
    ISDN,
    WorldFip,
    PICGMG,
    Infiniband,
    Fabric,
    Other,
    Unknown(u8),
}

impl NetworkControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Ethernet,
            0x01 => Self::TokenRing,
            0x02 => Self::FDDI,
            0x03 => Self::ATM,
            0x04 => Self::ISDN,
            0x05 => Self::WorldFip,
            0x06 => Self::PICGMG,
            0x07 => Self::Infiniband,
            0x08 => Self::Fabric,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum DisplayControllerSubclass {
    VGACompatible,
    XGACompatible,
    ThreeDController,
    Other,
    Unknown(u8),
}

impl DisplayControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::VGACompatible,
            0x01 => Self::XGACompatible,
            0x02 => Self::ThreeDController,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum MultimediaControllerSubclass {
    Video,
    Audio,
    Telephony,
    Other,
    Unknown(u8),
}

impl MultimediaControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Video,
            0x01 => Self::Audio,
            0x02 => Self::Telephony,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum MemoryControllerSubclass {
    RAM,
    Flash,
    Other,
    Unknown(u8),
}

impl MemoryControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::RAM,
            0x01 => Self::Flash,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum BridgeDeviceSubclass {
    Host,
    ISA,
    EISA,
    MCA,
    PCItoPCI,
    PCMCIA,
    NuBus,
    CardBus,
    RACEway,
    PCItoPCI2,
    InfinibandtoPCI,
    Other,
    Unknown(u8),
}

impl BridgeDeviceSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Host,
            0x01 => Self::ISA,
            0x02 => Self::EISA,
            0x03 => Self::MCA,
            0x04 => Self::PCItoPCI,
            0x05 => Self::PCMCIA,
            0x06 => Self::NuBus,
            0x07 => Self::CardBus,
            0x08 => Self::RACEway,
            0x09 => Self::PCItoPCI2,
            0x0a => Self::InfinibandtoPCI,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum SimpleCommunicationControllerSubclass {
    Serial,
    Parallel,
    MultiportSerial,
    Modem,
    GPIB,
    SmartCard,
    Other,
    Unknown(u8),
}

impl SimpleCommunicationControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Serial,
            0x01 => Self::Parallel,
            0x02 => Self::MultiportSerial,
            0x03 => Self::Modem,
            0x04 => Self::GPIB,
            0x05 => Self::SmartCard,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum BaseSystemPeripheralSubclass {
    PIC,
    DMA,
    Timer,
    RTC,
    PCIHotPlug,
    SDHost,
    IOMMU,
    Other,
    Unknown(u8),
}

impl BaseSystemPeripheralSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::PIC,
            0x01 => Self::DMA,
            0x02 => Self::Timer,
            0x03 => Self::RTC,
            0x04 => Self::PCIHotPlug,
            0x05 => Self::SDHost,
            0x06 => Self::IOMMU,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum InputDeviceControllerSubclass {
    Keyboard,
    DigitizerPen,
    Mouse,
    Scanner,
    Gameport,
    Other,
    Unknown(u8),
}

impl InputDeviceControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Keyboard,
            0x01 => Self::DigitizerPen,
            0x02 => Self::Mouse,
            0x03 => Self::Scanner,
            0x04 => Self::Gameport,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum DockingStationSubclass {
    Generic,
    Other,
    Unknown(u8),
}

impl DockingStationSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Generic,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum ProcessorSubclass {
    I386,
    I486,
    Pentium,
    PentiumPro,
    Alpha,
    PowerPC,
    MIPS,
    CoProcessor,
    Other,
    Unknown(u8),
}

impl ProcessorSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::I386,
            0x01 => Self::I486,
            0x02 => Self::Pentium,
            0x03 => Self::PentiumPro,
            0x10 => Self::Alpha,
            0x20 => Self::PowerPC,
            0x30 => Self::MIPS,
            0x40 => Self::CoProcessor,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum SerialBusControllerSubclass {
    Firewire,
    ACCESS,
    SSA,
    USB,
    FibreChannel,
    SMBus,
    Infiniband,
    IPMI,
    SERCOS,
    CANbus,
    Other,
    Unknown(u8),
}

impl SerialBusControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Firewire,
            0x01 => Self::ACCESS,
            0x02 => Self::SSA,
            0x03 => Self::USB,
            0x04 => Self::FibreChannel,
            0x05 => Self::SMBus,
            0x06 => Self::Infiniband,
            0x07 => Self::IPMI,
            0x08 => Self::SERCOS,
            0x09 => Self::CANbus,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum WirelessControllerSubclass {
    IRDA,
    ConsumerIR,
    RF,
    Bluetooth,
    Broadband,
    Ethernet,
    Other,
    Unknown(u8),
}

impl WirelessControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::IRDA,
            0x01 => Self::ConsumerIR,
            0x10 => Self::RF,
            0x11 => Self::Bluetooth,
            0x12 => Self::Broadband,
            0x20 => Self::Ethernet,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum IntelligentControllerSubclass {
    I2O,
    Other,
    Unknown(u8),
}

impl IntelligentControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::I2O,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum SatelliteCommunicationControllerSubclass {
    TV,
    Audio,
    Voice,
    Data,
    Other,
    Unknown(u8),
}

impl SatelliteCommunicationControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x01 => Self::TV,
            0x02 => Self::Audio,
            0x03 => Self::Voice,
            0x04 => Self::Data,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum EncryptionControllerSubclass {
    Network,
    Entertainment,
    Other,
    Unknown(u8),
}

impl EncryptionControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::Network,
            0x10 => Self::Entertainment,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}

#[derive(Debug)]
pub enum SignalProcessingControllerSubclass {
    DPIO,
    PerformanceCounter,
    CommunicationSynchronizer,
    SignalProcessingManagement,
    Other,
    Unknown(u8),
}

impl SignalProcessingControllerSubclass {
    pub(super) fn from_u8(subclass: u8) -> Self {
        match subclass {
            0x00 => Self::DPIO,
            0x01 => Self::PerformanceCounter,
            0x10 => Self::CommunicationSynchronizer,
            0x20 => Self::SignalProcessingManagement,
            0x80 => Self::Other,
            x => Self::Unknown(x),
        }
    }
}
