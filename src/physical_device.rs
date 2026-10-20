use rusb::{
    devices, ConfigDescriptor, Device, DeviceHandle, Error as RusbError, GlobalContext,
    InterfaceDescriptor, TransferType,
};

use std::time::Duration;

pub struct PhysicalDevice {
    device: Device<GlobalContext>,
    device_handle: DeviceHandle<GlobalContext>,
    endpoint_address: u8,
}

impl PhysicalDevice {
    pub fn open(vid: u16, pid: u16) -> Result<Self, RusbError> {
        let device = Self::get_target_device(vid, pid)?;
        let device_handle = device.open()?;

        let mut physical = PhysicalDevice {
            endpoint_address: 0,
            device_handle,
            device,
        };

        physical.init()?;
        physical.set_full_mode()?;
        Ok(physical)
    }

    #[allow(dead_code)]
    pub fn new(vid: u16, pid: u16) -> Self {
        Self::open(vid, pid).expect("Error finding and initializing device.")
    }

    pub fn init(&mut self) -> Result<(), RusbError> {
        let _ = self.device_handle.set_auto_detach_kernel_driver(true);

        let configurations = Self::get_configurations(&self.device);
        let interface_descriptors = Self::get_hid_interface_descriptors(&configurations);

        let mut found_endpoint = false;
        for interface_descriptor in interface_descriptors {
            self.device_handle
                .claim_interface(interface_descriptor.interface_number())?;
            for endpoint_descriptor in interface_descriptor.endpoint_descriptors() {
                if endpoint_descriptor.transfer_type() == TransferType::Interrupt
                    && endpoint_descriptor.max_packet_size() == 64
                {
                    self.endpoint_address = endpoint_descriptor.address();
                    found_endpoint = true;
                }
            }
        }

        if !found_endpoint {
            return Err(RusbError::NotFound);
        }

        self.reset()?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), RusbError> {
        self.device_handle.reset()
    }

    pub fn read_device_responses(&self, buffer: &mut [u8]) -> Result<usize, RusbError> {
        self.device_handle
            .read_interrupt(self.endpoint_address, buffer, Duration::from_millis(1000))
    }

    pub fn set_full_mode(&mut self) -> Result<(), RusbError> {
        const REPORTS: [[u8; 8]; 1] = [[0x08, 0x03, 0x00, 0xff, 0xf0, 0x00, 0xff, 0xf0]];
        let reports_as_slices: Vec<&[u8]> = REPORTS.iter().map(|r| &r[..]).collect();
        self.set_report(&reports_as_slices)
    }

    pub fn set_report(&mut self, reports: &[&[u8]]) -> Result<(), RusbError> {
        for report in reports.iter() {
            self.device_handle.write_control(
                0x21,
                0x9,
                0x0308,
                2,
                report,
                Duration::from_millis(250),
            )?;
        }

        Ok(())
    }

    fn is_target_device(vid: u16, pid: u16, device: &Device<GlobalContext>) -> bool {
        match device.device_descriptor() {
            Ok(desc) => desc.vendor_id() == vid && desc.product_id() == pid,
            Err(_) => false,
        }
    }

    fn get_target_device(vid: u16, pid: u16) -> Result<Device<GlobalContext>, RusbError> {
        match devices()?
            .iter()
            .find(|device| Self::is_target_device(vid, pid, device))
        {
            Some(device) => Ok(device),
            None => Err(RusbError::NoDevice),
        }
    }

    fn get_hid_interface_descriptors<'a>(
        config_descriptors: &'a [ConfigDescriptor],
    ) -> Vec<InterfaceDescriptor<'a>> {
        config_descriptors
            .iter()
            .flat_map(|config_descriptor| config_descriptor.interfaces())
            .flat_map(|interface| interface.descriptors())
            .filter(|interface_descriptor| {
                interface_descriptor.class_code() == rusb::constants::LIBUSB_CLASS_HID
            })
            .collect()
    }

    fn get_configurations(device: &Device<GlobalContext>) -> Vec<ConfigDescriptor> {
        match device.device_descriptor() {
            Ok(desc) => (0..desc.num_configurations())
                .filter_map(|n| device.config_descriptor(n).ok())
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
